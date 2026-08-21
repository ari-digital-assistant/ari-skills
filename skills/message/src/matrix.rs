//! Matrix transport.
//!
//! The one service Ari sends entirely by itself: four ordinary HTTPS calls
//! against the user's own homeserver, no other app in the loop. That also
//! makes it the one where a wrong recipient is unrecallable, which is why the
//! skill reads the message back first.
//!
//! Matrix has no address-book presence — an MXID is not a phone number — so
//! the recipient is resolved through the homeserver's own user directory
//! rather than through `contacts`.
//!
//! **Unencrypted rooms only.** Modern Matrix DMs are end-to-end encrypted by
//! default, and doing that properly means Olm/Megolm — Curve25519, Ed25519,
//! AES, HKDF — where the SDK ships `sha2` and nothing else. Sending plaintext
//! into an encrypted room either gets refused by the server or shows up with a
//! warning shield in the recipient's client, so this reports the refusal
//! rather than pretending.

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::transport::Outcome;

#[cfg(target_arch = "wasm32")]
use ari_skill_sdk as ari;

/// A person the directory matched, and the id we can address them by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryUser {
    pub user_id: String,
    pub display_name: String,
}

/// Pick the user a spoken name meant.
///
/// Exact display-name matches win outright; otherwise a single candidate is
/// taken and anything ambiguous is refused. Guessing between two people is
/// the one mistake this whole design exists to avoid.
pub fn choose(query: &str, users: &[DirectoryUser]) -> Result<DirectoryUser, Ambiguity> {
    let q = query.trim().to_lowercase();
    if users.is_empty() {
        return Err(Ambiguity::None);
    }
    let exact: Vec<&DirectoryUser> = users
        .iter()
        .filter(|u| u.display_name.trim().to_lowercase() == q)
        .collect();
    if exact.len() == 1 {
        return Ok(exact[0].clone());
    }
    if exact.len() > 1 {
        return Err(Ambiguity::Several(exact.iter().map(|u| u.display_name.clone()).collect()));
    }
    if users.len() == 1 {
        return Ok(users[0].clone());
    }
    Err(Ambiguity::Several(users.iter().map(|u| u.display_name.clone()).collect()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ambiguity {
    None,
    Several(Vec<String>),
}

/// The room a `m.direct` account-data blob says to use for `user_id`.
///
/// The blob maps each MXID to every DM room shared with them; the last is the
/// most recently created, which is the one their client will be looking at.
pub fn direct_room_for(account_data: &str, user_id: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(account_data).ok()?;
    v.get(user_id)?
        .as_array()?
        .iter()
        .filter_map(|r| r.as_str())
        .next_back()
        .map(|s| s.to_string())
}

/// Users from a `/user_directory/search` response.
pub fn parse_directory(body: &str) -> Vec<DirectoryUser> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(body) else {
        return Vec::new();
    };
    v.get("results")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|u| {
                    let id = u.get("user_id")?.as_str()?.to_string();
                    let name = u
                        .get("display_name")
                        .and_then(|d| d.as_str())
                        .unwrap_or(&id)
                        .to_string();
                    Some(DirectoryUser { user_id: id, display_name: name })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Trim a trailing slash so `https://matrix.org/` and `https://matrix.org`
/// build the same URL.
pub fn base_url(homeserver: &str) -> String {
    homeserver.trim().trim_end_matches('/').to_string()
}

/// Matrix requires a transaction id that is unique per request, so a retried
/// send is de-duplicated rather than delivered twice.
pub fn txn_id(now_ms: u64, nonce: u64) -> String {
    format!("ari{now_ms}.{nonce}")
}

/// Does this error body mean "that room is encrypted"?
///
/// Servers don't agree on the code, so the check is deliberately loose — the
/// alternative is telling the user their message sent when it didn't.
pub fn is_encryption_refusal(body: &str) -> bool {
    let b = body.to_lowercase();
    b.contains("encrypt")
}

#[cfg(target_arch = "wasm32")]
pub fn send(homeserver: &str, token: &str, recipient: &str, body: &str) -> Outcome {
    let base = base_url(homeserver);
    if base.is_empty() || token.trim().is_empty() {
        return Outcome::NotConfigured;
    }
    let auth = format!("Bearer {}", token.trim());

    let users = match get(&format!(
        "{base}/_matrix/client/v3/user_directory/search"
    ), &auth, Some(&serde_json::json!({ "search_term": recipient, "limit": 10 }).to_string()))
    {
        Ok(b) => parse_directory(&b),
        Err(o) => return o,
    };
    let user = match choose(recipient, &users) {
        Ok(u) => u,
        Err(Ambiguity::None) => return Outcome::NoRecipient,
        Err(Ambiguity::Several(names)) => return Outcome::Ambiguous(names),
    };

    let whoami = match get_json(&format!("{base}/_matrix/client/v3/account/whoami"), &auth) {
        Ok(v) => v,
        Err(o) => return o,
    };
    let Some(me) = whoami.get("user_id").and_then(|v| v.as_str()) else {
        return Outcome::Failed("homeserver did not say who I am".to_string());
    };

    let room = match get_json(
        &format!("{base}/_matrix/client/v3/user/{me}/account_data/m.direct"),
        &auth,
    ) {
        Ok(v) => direct_room_for(&v.to_string(), &user.user_id),
        // No m.direct yet is normal on a fresh account, not a failure.
        Err(_) => None,
    };
    let room = match room {
        Some(r) => r,
        None => match create_dm(&base, &auth, &user.user_id) {
            Ok(r) => r,
            Err(o) => return o,
        },
    };

    let txn = txn_id(ari::now_ms() as u64, ari::rand_u64());
    let url = format!("{base}/_matrix/client/v3/rooms/{room}/send/m.room.message/{txn}");
    let payload = serde_json::json!({ "msgtype": "m.text", "body": body }).to_string();
    let resp = ari::http_request(
        "PUT",
        &url,
        &[("Authorization", &auth), ("Content-Type", "application/json")],
        Some(&payload),
    );
    classify(&resp).unwrap_or(Outcome::Sent)
}

#[cfg(target_arch = "wasm32")]
fn create_dm(base: &str, auth: &str, user_id: &str) -> Result<String, Outcome> {
    let payload =
        serde_json::json!({ "is_direct": true, "preset": "trusted_private_chat", "invite": [user_id] })
            .to_string();
    let resp = ari::http_request(
        "POST",
        &format!("{base}/_matrix/client/v3/createRoom"),
        &[("Authorization", auth), ("Content-Type", "application/json")],
        Some(&payload),
    );
    if let Some(o) = classify(&resp) {
        return Err(o);
    }
    serde_json::from_str::<serde_json::Value>(resp.body.as_deref().unwrap_or(""))
        .ok()
        .and_then(|v| v.get("room_id").and_then(|r| r.as_str()).map(|s| s.to_string()))
        .ok_or_else(|| Outcome::Failed("homeserver made no room".to_string()))
}

#[cfg(target_arch = "wasm32")]
fn get(url: &str, auth: &str, body: Option<&str>) -> Result<String, Outcome> {
    let method = if body.is_some() { "POST" } else { "GET" };
    let resp = ari::http_request(
        method,
        url,
        &[("Authorization", auth), ("Content-Type", "application/json")],
        body,
    );
    match classify(&resp) {
        Some(o) => Err(o),
        None => Ok(resp.body.unwrap_or_default()),
    }
}

#[cfg(target_arch = "wasm32")]
fn get_json(url: &str, auth: &str) -> Result<serde_json::Value, Outcome> {
    let body = get(url, auth, None)?;
    serde_json::from_str(&body).map_err(|_| Outcome::Failed("bad response".to_string()))
}

/// `None` means the call was fine. Anything else is why it wasn't.
#[cfg(target_arch = "wasm32")]
fn classify(resp: &ari::HttpResponse) -> Option<Outcome> {
    // status 0 is a transport failure, not an HTTP status — users on mobile
    // data hit it constantly, and it deserves its own sentence.
    if resp.status == 0 {
        return Some(Outcome::Offline);
    }
    if resp.status == 401 || resp.status == 403 {
        let body = resp.body.as_deref().unwrap_or("");
        if is_encryption_refusal(body) {
            return Some(Outcome::Encrypted);
        }
        return Some(Outcome::NotConfigured);
    }
    if resp.status >= 400 {
        let body = resp.body.as_deref().unwrap_or("");
        if is_encryption_refusal(body) {
            return Some(Outcome::Encrypted);
        }
        return Some(Outcome::Failed(format!("homeserver said {}", resp.status)));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(id: &str, name: &str) -> DirectoryUser {
        DirectoryUser { user_id: id.to_string(), display_name: name.to_string() }
    }

    #[test]
    fn a_single_result_is_taken() {
        let users = [u("@gail:example.org", "Gail Borg")];
        assert_eq!(choose("gail", &users).unwrap().user_id, "@gail:example.org");
    }

    #[test]
    fn an_exact_display_name_beats_other_candidates() {
        let users = [u("@g1:x.org", "Gail Borg"), u("@g2:x.org", "Gail")];
        assert_eq!(choose("Gail", &users).unwrap().user_id, "@g2:x.org");
    }

    #[test]
    fn two_plausible_people_are_refused_not_guessed() {
        // The whole point: sending to the wrong person can't be undone.
        let users = [u("@g1:x.org", "Gail Borg"), u("@g2:x.org", "Gail Marie")];
        match choose("gail", &users) {
            Err(Ambiguity::Several(names)) => assert_eq!(names.len(), 2),
            other => panic!("expected ambiguity, got {other:?}"),
        }
    }

    #[test]
    fn two_people_sharing_a_display_name_are_also_refused() {
        let users = [u("@g1:x.org", "Gail"), u("@g2:x.org", "Gail")];
        assert!(matches!(choose("gail", &users), Err(Ambiguity::Several(_))));
    }

    #[test]
    fn no_results_is_distinct_from_ambiguity() {
        assert_eq!(choose("gail", &[]), Err(Ambiguity::None));
    }

    #[test]
    fn directory_results_parse() {
        let body = r#"{"results":[{"user_id":"@gail:x.org","display_name":"Gail Borg"}]}"#;
        let users = parse_directory(body);
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].display_name, "Gail Borg");
    }

    #[test]
    fn a_result_without_a_display_name_falls_back_to_its_id() {
        let users = parse_directory(r#"{"results":[{"user_id":"@gail:x.org"}]}"#);
        assert_eq!(users[0].display_name, "@gail:x.org");
    }

    #[test]
    fn a_junk_directory_response_yields_nobody_rather_than_panicking() {
        assert!(parse_directory("not json").is_empty());
        assert!(parse_directory(r#"{"results":"nope"}"#).is_empty());
    }

    #[test]
    fn the_most_recent_direct_room_wins() {
        // The last entry is the newest, and the one their client has open.
        let data = r#"{"@gail:x.org":["!old:x.org","!new:x.org"]}"#;
        assert_eq!(direct_room_for(data, "@gail:x.org").as_deref(), Some("!new:x.org"));
    }

    #[test]
    fn no_direct_room_for_a_stranger() {
        let data = r#"{"@someone:x.org":["!r:x.org"]}"#;
        assert_eq!(direct_room_for(data, "@gail:x.org"), None);
    }

    #[test]
    fn base_url_tolerates_a_trailing_slash() {
        assert_eq!(base_url("https://matrix.org/"), "https://matrix.org");
        assert_eq!(base_url(" https://matrix.org "), "https://matrix.org");
    }

    #[test]
    fn transaction_ids_differ_between_sends() {
        // Matrix de-duplicates on this, so a repeat would silently vanish.
        assert_ne!(txn_id(1, 1), txn_id(1, 2));
        assert_ne!(txn_id(1, 1), txn_id(2, 1));
    }

    #[test]
    fn encryption_refusals_are_recognised() {
        assert!(is_encryption_refusal(r#"{"errcode":"M_FORBIDDEN","error":"Room is encrypted"}"#));
        assert!(!is_encryption_refusal(r#"{"errcode":"M_UNKNOWN_TOKEN"}"#));
    }
}
