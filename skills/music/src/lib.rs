#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

mod parse;
mod action;
mod resolve;
mod transport;
mod spotify;

use ari_skill_sdk as ari;

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug)]
pub enum ReplyOutcome {
    Play { query: String, service: String },
    Unrecognized,
}

/// Resolve a spoken reply to the "which service?" question. `context_json` is
/// the blob the picker stored: `{"query":…,"installed":[…]}`. We scan the
/// reply's tokens for the first that canonicalises to an installed service.
pub fn resolve_reply(context_json: &str, text: &str) -> ReplyOutcome {
    use alloc::string::ToString;
    let v: serde_json::Value = match serde_json::from_str(context_json) {
        Ok(v) => v,
        Err(_) => return ReplyOutcome::Unrecognized,
    };
    let query = v.get("query").and_then(|q| q.as_str()).unwrap_or("").to_string();
    let installed: Vec<String> = v
        .get("installed")
        .and_then(|i| i.as_array())
        .map(|a| a.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    // The reply is either a bare label ("apple music") or — for BOTH the tap
    // and the voice-intercept paths — the card action's utterance
    // ("play <query> on <label>"). Service aliases are up to two words
    // ("apple music", "amazon music"), so scan each token AND each adjacent
    // pair through the canonicaliser. Longest window wins at a given position
    // (so "apple music" beats "apple"); we keep the LAST installed match so a
    // service named after "on" wins over any service-like word in the query.
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let mut chosen: Option<String> = None;
    for i in 0..tokens.len() {
        for span in [2usize, 1] {
            if i + span > tokens.len() {
                continue;
            }
            let cand = tokens[i..i + span].join(" ");
            if let Some(canon) = parse::canonical_service(&cand) {
                if installed.iter().any(|s| s == &canon) {
                    chosen = Some(canon);
                    break; // longest match at this position wins
                }
            }
        }
    }
    match chosen {
        Some(service) => ReplyOutcome::Play { query, service },
        None => ReplyOutcome::Unrecognized,
    }
}

#[cfg(test)]
mod reply_tests {
    use super::*;

    #[test]
    fn reply_matching_installed_service_yields_play() {
        let ctx = r#"{"query":"hotel california","installed":["spotify","ytmusic"]}"#;
        let outcome = resolve_reply(ctx, "spotify");
        match outcome {
            ReplyOutcome::Play { query, service } => {
                assert_eq!(query, "hotel california");
                assert_eq!(service, "spotify");
            }
            other => panic!("expected Play, got {other:?}"),
        }
    }

    #[test]
    fn reply_with_alias_and_filler_still_matches() {
        let ctx = r#"{"query":"hotel california","installed":["spotify"]}"#;
        // canonical_service handles case/aliases; filler words are tolerated
        // by scanning tokens.
        assert!(matches!(resolve_reply(ctx, "spotify please"),
            ReplyOutcome::Play { ref service, .. } if service == "spotify"));
    }

    #[test]
    fn reply_not_matching_any_installed_service_fails() {
        let ctx = r#"{"query":"hotel california","installed":["spotify"]}"#;
        assert!(matches!(resolve_reply(ctx, "pandora"), ReplyOutcome::Unrecognized));
        assert!(matches!(resolve_reply(ctx, "blah blah"), ReplyOutcome::Unrecognized));
    }

    // Both tap and voice-intercept submit the card action's utterance
    // ("play <query> on <label>") as the reply — NOT the bare label — so a
    // MULTI-WORD service label must resolve out of that full sentence.
    #[test]
    fn reply_from_card_utterance_multiword_service_matches() {
        let ctx = r#"{"query":"hotel california","installed":["spotify","apple_music"]}"#;
        let outcome = resolve_reply(ctx, "play hotel california on apple music");
        match outcome {
            ReplyOutcome::Play { query, service } => {
                assert_eq!(query, "hotel california");
                assert_eq!(service, "apple_music");
            }
            other => panic!("expected Play(apple_music), got {other:?}"),
        }
    }

    #[test]
    fn reply_from_card_utterance_singleword_service_matches() {
        let ctx = r#"{"query":"x","installed":["spotify","apple_music"]}"#;
        assert!(matches!(resolve_reply(ctx, "play x on spotify"),
            ReplyOutcome::Play { ref service, .. } if service == "spotify"));
    }

    // The service after "on" must win over a service-like word buried in the
    // query (the picked service is always at the end of the utterance).
    #[test]
    fn reply_service_after_on_wins_over_query_token() {
        let ctx = r#"{"query":"spotify anthem","installed":["spotify","apple_music"]}"#;
        assert!(matches!(resolve_reply(ctx, "play spotify anthem on apple music"),
            ReplyOutcome::Play { ref service, .. } if service == "apple_music"));
    }

    #[test]
    fn bare_multiword_label_still_matches() {
        let ctx = r#"{"query":"x","installed":["spotify","apple_music"]}"#;
        assert!(matches!(resolve_reply(ctx, "apple music"),
            ReplyOutcome::Play { ref service, .. } if service == "apple_music"));
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    0.9
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    use alloc::string::ToString;
    let input = unsafe { ari::input(ptr, len) };

    // Multi-turn: if this is the user's reply to our "which service?" picker,
    // handle it directly — it must NOT be re-parsed as a fresh "play" request.
    if let Some(reply) = ari::parse_reply(input) {
        return match resolve_reply(&reply.context, &reply.text) {
            ReplyOutcome::Play { query, service } => play(&query, &service),
            ReplyOutcome::Unrecognized => {
                ari::respond_text(
                    ari::t("reply_unrecognized", &[])
                        .unwrap_or("Sorry, I didn't catch which service."),
                )
            }
        };
    }

    // Transport/volume verbs are unambiguous (no query) and must be handled
    // before the play path — a bare "pause"/"next"/"stop" is a command, not a
    // song title. Parsed from the raw utterance, independent of router args.
    if let Some(t) = transport::parse(input) {
        return ari::respond_action(&action::transport_action_json(&t));
    }

    // Prefer router typed args, else keyword parse.
    let parsed = match ari::args().and_then(parse_args) {
        Some(p) => p,
        None => parse::parse(input),
    };

    let default_setting = ari::setting_get("default_service").unwrap_or("last_used").to_string();
    let last_used = ari::storage_get("last_service").map(|s| s.to_string());
    let installed = ari::media_services();

    match resolve::decide(parsed, &default_setting, last_used, &installed) {
        resolve::Decision::Play { query, service } => play(&query, &service),
        resolve::Decision::Ask { query, installed } => {
            ari::respond_action(&build_picker(&query, &installed))
        }
        resolve::Decision::Clarify => {
            ari::respond_text(ari::t("clarify_no_query", &[]).unwrap_or("What would you like me to play?"))
        }
        resolve::Decision::NoApp => {
            ari::respond_text(ari::t("no_music_app", &[]).unwrap_or("I couldn't find a music app installed."))
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn play(query: &str, service: &str) -> i64 {
    let _ = ari::storage_set("last_service", service);
    if service == "spotify" {
        return play_on_spotify(query);
    }
    ari::respond_action(&action::play_action_json(query, service))
}

#[cfg(target_arch = "wasm32")]
fn play_on_spotify(query: &str) -> i64 {
    use ari_skill_sdk::presentation as p;
    use spotify::LookupError;

    let client_id = ari::setting_get("spotify_client_id").map(str::trim).unwrap_or("");
    if client_id.is_empty() {
        return ari::respond_text(ari::t("spotify_needs_key", &[]).unwrap_or(
            "To play songs on Spotify I need your Spotify Client ID. Add it in the Music skill's settings, then sign in.",
        ));
    }
    match spotify::find_track(client_id, query) {
        Ok(track) => {
            let speak = ari::t("spotify_playing", &[("title", &track.name), ("artist", &track.artist)])
                .unwrap_or("Playing it on Spotify.");
            ari::respond_action(&p::Envelope::new().speak(speak).open_url(track.uri).to_json())
        }
        Err(LookupError::NotSignedIn) => ari::respond_text(ari::t("spotify_not_signed_in", &[]).unwrap_or(
            "Sign in to Spotify in the Music skill's settings first.",
        )),
        Err(LookupError::SignInExpired) => ari::respond_text(ari::t("spotify_sign_in_expired", &[]).unwrap_or(
            "Your Spotify sign-in has stopped working. Sign in again in the Music skill's settings.",
        )),
        Err(LookupError::NotFound) => ari::respond_text(
            ari::t("spotify_not_found", &[("query", query)]).unwrap_or("I couldn't find that on Spotify."),
        ),
        Err(LookupError::NoConnection) => ari::respond_text(
            ari::t("spotify_no_connection", &[]).unwrap_or("I couldn't reach Spotify. Check your internet connection."),
        ),
        Err(LookupError::Unavailable) => ari::respond_text(
            ari::t("spotify_unavailable", &[]).unwrap_or("Spotify isn't answering right now. Try again in a minute."),
        ),
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn settings_action(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    ari::respond_action(&handle_settings_action(input))
}

#[cfg(target_arch = "wasm32")]
fn handle_settings_action(input: &str) -> alloc::string::String {
    use alloc::string::ToString;
    use ari::settings::{parse_action_input, SettingsResult};
    let t_or = |key: &str, args: &[(&str, &str)], fallback: &str| ari::t(key, args).unwrap_or(fallback).to_string();

    let a = match parse_action_input(input) {
        Some(a) => a,
        None => return SettingsResult::error("bad action input").to_json(),
    };
    if a.action != "spotify_sign_in" {
        return SettingsResult::error("unknown action").to_json();
    }
    let client_id = match a.value("spotify_client_id").map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => s.to_string(),
        None => {
            return SettingsResult::error(&t_or("spotify_sign_in_needs_id", &[], "Paste your Spotify Client ID first."))
                .to_json()
        }
    };

    let verifier = random_urlsafe::<32>();
    let state = random_urlsafe::<16>();
    let redirect_uri = ari::oauth_redirect_uri();
    let auth_url = spotify::authorize_url(&client_id, &redirect_uri, &state, &spotify::pkce_challenge(&verifier));

    // Spotify reports a wrong Client ID or an unregistered redirect on its own
    // page and never redirects back, so both surface here as an incomplete
    // sign-in. Name the two things to check.
    let res = ari::authorize(&auth_url, &redirect_uri, spotify::SIGN_IN_TIMEOUT_MS);
    if !res.ok || res.get("error").is_some() {
        let msg = if res.error.as_deref() == Some("no_browser") {
            t_or("spotify_sign_in_no_browser", &[], "I couldn't open a browser to sign in.")
        } else {
            t_or(
                "spotify_sign_in_incomplete",
                &[("redirect", &redirect_uri)],
                "Sign-in didn't complete. Check your Client ID, and that your Spotify app lists Ari's Redirect URI.",
            )
        };
        return SettingsResult::error(&msg).to_json();
    }
    let unverified = || {
        SettingsResult::error(&t_or("spotify_sign_in_unverified", &[], "Spotify sign-in couldn't be verified. Please try again."))
            .to_json()
    };
    if res.get("state") != Some(state.as_str()) {
        return unverified();
    }
    let code = match res.get("code") {
        Some(c) if !c.is_empty() => c.to_string(),
        _ => return unverified(),
    };

    let resp = ari::http_request(
        "POST",
        spotify::TOKEN_URL,
        &[("Content-Type", "application/x-www-form-urlencoded")],
        Some(&spotify::exchange_body(&code, &client_id, &verifier, &redirect_uri)),
    );
    if resp.status == 0 {
        return SettingsResult::error(&t_or(
            "spotify_no_connection",
            &[],
            "I couldn't reach Spotify. Check your internet connection.",
        ))
        .to_json();
    }
    let refresh = match resp.body.as_deref().filter(|_| resp.status == 200).and_then(spotify::parse_tokens) {
        Some(spotify::Tokens { refresh_token: Some(r), .. }) => r,
        _ => return unverified(),
    };
    if !spotify::store_refresh_token(&refresh) {
        return unverified();
    }
    SettingsResult::validated(&t_or("spotify_signed_in", &[], "Signed in to Spotify.")).with_refresh().to_json()
}

/// CSPRNG bytes as base64url: 32 bytes make a 43-char PKCE verifier, 16 a
/// 128-bit state.
#[cfg(target_arch = "wasm32")]
fn random_urlsafe<const N: usize>() -> alloc::string::String {
    let mut bytes = [0u8; N];
    for chunk in bytes.chunks_mut(8) {
        chunk.copy_from_slice(&ari::rand_u64().to_le_bytes()[..chunk.len()]);
    }
    ari::crypto::base64url_nopad(&bytes)
}

#[cfg(target_arch = "wasm32")]
fn parse_args(args_json: &str) -> Option<parse::Parsed> {
    use alloc::string::ToString;
    let v: serde_json::Value = serde_json::from_str(args_json).ok()?;
    let query = v.get("query").and_then(|q| q.as_str()).map(|s| s.trim())
        .filter(|s| !s.is_empty()).map(|s| s.to_string());
    let service = v.get("service").and_then(|s| s.as_str())
        .and_then(parse::canonical_service);
    if query.is_none() && service.is_none() { return None; }
    Some(parse::Parsed { query, service })
}

#[cfg(target_arch = "wasm32")]
fn build_picker(query: &str, installed: &[alloc::string::String]) -> alloc::string::String {
    use ari_skill_sdk::presentation as p;
    use alloc::format;
    use alloc::string::ToString;
    // The question doubles as the spoken prompt: on the voice path the mic
    // re-arms (await_reply) and the user must HEAR what's being asked — a card
    // title alone is silent. `speak` and `title` carry the same string.
    let prompt = ari::t("ask_which_service", &[])
        .unwrap_or("Which service would you like to use?")
        .to_string();
    let mut card = p::Card::new("music_pick").title(prompt.clone());
    for id in installed {
        let label = ari::t(&format!("service_{id}"), &[]).unwrap_or(id).to_string();
        // utterance re-uses the natural parse path; label drives voice-pick.
        let utt = format!("play {query} on {label}");
        card = card.action(p::Action::new(id, &label).utterance(&utt));
    }
    // Context the engine stores for our reply short-circuit: the pending query
    // and the services to match a spoken answer against. Buttons above remain
    // for tap + legacy voice-intercept; await_reply adds the multi-turn path.
    let context = serde_json::json!({ "query": query, "installed": installed }).to_string();
    p::Envelope::new().speak(prompt).card(card).await_reply(context).to_json()
}
