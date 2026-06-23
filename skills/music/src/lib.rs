#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

mod parse;
mod action;
mod resolve;

use ari_skill_sdk as ari;

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
    let mut card = p::Card::new("music_pick").title(
        ari::t("ask_which_service", &[]).unwrap_or("Which service would you like to use?"),
    );
    for id in installed {
        let label = ari::t(&format!("service_{id}"), &[]).unwrap_or(id).to_string();
        // utterance re-uses the natural parse path; label drives voice-pick.
        let utt = format!("play {query} on {label}");
        card = card.action(p::Action::new(id, &label).utterance(&utt));
    }
    p::Envelope::new().card(card).to_json()
}
