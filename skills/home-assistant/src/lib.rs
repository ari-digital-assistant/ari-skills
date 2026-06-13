#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

mod logic;

#[cfg(target_arch = "wasm32")]
use ari_skill_sdk as ari;

/// Ceremonial — the manifest's `matching.patterns` score this skill
/// (`custom_score: false`), so the host never calls this export.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    0.85
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    let envelope = dispatch_wasm(input);
    ari::respond_action(&envelope)
}

#[cfg(target_arch = "wasm32")]
use alloc::string::{String, ToString};

#[cfg(target_arch = "wasm32")]
fn t_or(key: &str, args: &[(&str, &str)], fallback: &str) -> String {
    ari::t(key, args).unwrap_or(fallback).to_string()
}

#[cfg(target_arch = "wasm32")]
fn dispatch_wasm(input: &str) -> String {
    // 1. Settings.
    let base_url = match ari::setting_get("base_url") {
        Some(s) if !s.trim().is_empty() => s.to_string(),
        _ => return logic::error_envelope(&t_or("not_configured", &[], "Home Assistant isn't set up yet.")),
    };
    let token = match ari::setting_get("token") {
        Some(s) if !s.trim().is_empty() => s.to_string(),
        _ => return logic::error_envelope(&t_or("not_configured", &[], "Home Assistant isn't set up yet.")),
    };
    let language = ari::setting_get("language")
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| ari::get_locale().to_string());
    let private = logic::is_private_base_url(&base_url);

    match logic::classify(input) {
        logic::Intent::PersonLocation { name } => person_flow(&base_url, &token, &name, private),
        logic::Intent::Forward => forward_flow(&base_url, &token, input, &language, private),
    }
}

#[cfg(target_arch = "wasm32")]
fn forward_flow(base_url: &str, token: &str, input: &str, language: &str, private: bool) -> String {
    let req = logic::build_conversation_request(base_url, token, input, language);
    let (auth_k, auth_v) = req.auth_header();
    let resp = ari::http_request(
        req.method,
        &req.url,
        &[(&auth_k, &auth_v), ("Content-Type", "application/json")],
        Some(&req.body),
    );
    if let Some(kind) = logic::http_error_kind(resp.status, private) {
        return logic::error_envelope(&render_error(kind));
    }
    match resp.body.as_deref().and_then(logic::parse_conversation_response) {
        Some(result) => {
            let title = t_or("card.done", &[], "Done");
            logic::build_conversation_envelope(&result, &title)
        }
        None => logic::error_envelope(&t_or("no_match", &[], "I couldn't find that in Home Assistant.")),
    }
}

#[cfg(target_arch = "wasm32")]
fn person_flow(base_url: &str, token: &str, name: &str, private: bool) -> String {
    let req = logic::build_person_template_request(base_url, token);
    let (auth_k, auth_v) = req.auth_header();
    let resp = ari::http_request(
        req.method,
        &req.url,
        &[(&auth_k, &auth_v), ("Content-Type", "application/json")],
        Some(&req.body),
    );
    if let Some(kind) = logic::http_error_kind(resp.status, private) {
        return logic::error_envelope(&render_error(kind));
    }
    let people = resp.body.as_deref().map(logic::parse_people).unwrap_or_default();
    match logic::match_person(&people, name) {
        Some(p) => logic::build_person_envelope(
            p,
            &t_or("person_at", &[], "{name} is at {place}."),
            &t_or("person_home", &[], "{name} is home."),
            &t_or("person_away", &[], "{name} is away."),
            &p.name,
            &t_or("card.person.home", &[], "Home"),
            &t_or("card.person.away", &[], "Away"),
        ),
        None => logic::error_envelope(&t_or("person_unknown", &[("name", name)], "I don't know who that is.")),
    }
}

#[cfg(target_arch = "wasm32")]
fn render_error(kind: logic::ErrorKind) -> String {
    match kind {
        logic::ErrorKind::Unreachable => t_or("unreachable", &[], "I couldn't reach Home Assistant."),
        logic::ErrorKind::UnreachableLan => t_or("unreachable_lan_hint", &[], "I couldn't reach Home Assistant — are you on your home network?"),
        logic::ErrorKind::Unauthorized => t_or("unauthorized", &[], "Home Assistant rejected the access token."),
    }
}
