#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

mod parse;
mod action;
mod resolve;
mod transport;

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

    // Try the whole reply, then each whitespace token, through the existing
    // canonicaliser; accept the first canonical id that is installed.
    let candidates = core::iter::once(text).chain(text.split_whitespace());
    for cand in candidates {
        if let Some(canon) = parse::canonical_service(cand) {
            if installed.iter().any(|s| s == &canon) {
                return ReplyOutcome::Play { query, service: canon };
            }
        }
    }
    ReplyOutcome::Unrecognized
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
            ReplyOutcome::Play { query, service } => {
                let _ = ari::storage_set("last_service", &service);
                ari::respond_action(&action::play_action_json(&query, &service))
            }
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
        resolve::Decision::Play { query, service } => {
            let _ = ari::storage_set("last_service", &service);
            ari::respond_action(&action::play_action_json(&query, &service))
        }
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
