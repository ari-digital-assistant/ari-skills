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
#[no_mangle]
pub extern "C" fn settings_query(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    let result = handle_settings_query(input);
    ari::respond_action(&result)
}

#[cfg(target_arch = "wasm32")]
fn handle_settings_query(input: &str) -> String {
    use ari::settings::{parse_query_input, SelectOpt, SettingsResult};
    use alloc::vec::Vec;

    migrate_refresh_token();
    let q = match parse_query_input(input) {
        Some(q) => q,
        None => return SettingsResult::error("bad query input").to_json(),
    };
    let base_url = match q.value("base_url").filter(|s| !s.trim().is_empty()) {
        Some(s) => s,
        None => {
            return SettingsResult::error(&t_or(
                "not_configured",
                &[],
                "Home Assistant isn't set up yet.",
            ))
            .to_json()
        }
    };
    let oauth = matches!(ari::storage_get("auth_mode"), Some("oauth"));
    // The form's `token` value is only relevant in manual mode. In oauth
    // mode the manual token field is empty by design — auth is proven by
    // the stored refresh token, validated by the early-return below.
    let manual_token = q.value("token").filter(|s| !s.trim().is_empty());
    if !oauth && manual_token.is_none() {
        return SettingsResult::error(&t_or(
            "not_configured",
            &[],
            "Home Assistant isn't set up yet.",
        ))
        .to_json();
    }
    let private = logic::is_private_base_url(base_url);

    // Validate field in oauth mode: the stored `token` is a refresh token, not
    // a Bearer — don't round-trip it (it would 401). Sign-in already proved the
    // connection. The agent_id fetch still needs a live round-trip below.
    if q.field != "agent_id" && oauth {
        return SettingsResult::validated(&t_or(
            "connected",
            &[],
            "Connected to Home Assistant.",
        ))
        .to_json();
    }

    // Bearer for the round-trip. Token mode: validate/use the form's token value
    // as-is. OAuth mode (only reached here for agent_id): resolve a real access
    // token — the stored `token` is a refresh token, not a Bearer.
    let bearer: alloc::string::String = if oauth {
        match resolve_bearer(base_url) {
            Bearer::Token(t) => t,
            Bearer::Reauth => {
                return SettingsResult::error(&t_or("needs_reauth", &[],
                    "I've lost my connection to Home Assistant — please sign in again in settings.")).to_json();
            }
        }
    } else {
        manual_token.expect("manual_token checked non-None above").to_string()
    };
    let req = logic::build_agents_template_request(base_url, &bearer);
    let (auth_k, auth_v) = req.auth_header();
    let resp = ari::http_request(
        req.method,
        &req.url,
        &[(&auth_k, &auth_v), ("Content-Type", "application/json")],
        Some(&req.body),
    );
    if let Some(kind) = logic::http_error_kind(resp.status, private) {
        return SettingsResult::error(&render_error(kind)).to_json();
    }
    match q.field.as_str() {
        "agent_id" => {
            let agents = resp
                .body
                .as_deref()
                .map(logic::parse_conversation_agents)
                .unwrap_or_default();
            let opts: Vec<SelectOpt> = agents
                .into_iter()
                .map(|(value, label)| SelectOpt { value, label })
                .collect();
            SettingsResult::options(opts).to_json()
        }
        _ => {
            // token/manual mode succeeded — record it so the next validate and
            // runtime calls treat the stored `token` as a Bearer.
            ari::storage_set("auth_mode", "token");
            SettingsResult::validated(&t_or(
                "connected",
                &[],
                "Connected to Home Assistant.",
            ))
            .to_json()
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn settings_action(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    let result = handle_settings_action(input);
    ari::respond_action(&result)
}

#[cfg(target_arch = "wasm32")]
fn handle_settings_action(input: &str) -> String {
    use ari::settings::{parse_action_input, SettingsResult};

    migrate_refresh_token();
    let a = match parse_action_input(input) {
        Some(a) => a,
        None => return SettingsResult::error("bad action input").to_json(),
    };
    if a.action != "sign_in" {
        return SettingsResult::error("unknown action").to_json();
    }
    let base_url = match a.value("base_url").filter(|s| !s.trim().is_empty()) {
        Some(s) => s.to_string(),
        None => return SettingsResult::error(&t_or("not_configured", &[], "Home Assistant isn't set up yet.")).to_json(),
    };
    let private = logic::is_private_base_url(&base_url);

    let verifier = oauth_verifier();
    let challenge = logic::pkce_challenge(&verifier);
    let state = oauth_state();
    let redirect_uri = ari::oauth_redirect_uri();
    let auth_url = logic::build_authorize_url(
        &base_url, logic::OAUTH_CLIENT_ID, &redirect_uri, &state, &challenge,
    );

    let res = ari::authorize(&auth_url, &redirect_uri, logic::OAUTH_TIMEOUT_MS);
    if !res.ok {
        let (key, fallback) = match res.error.as_deref() {
            Some("no_browser") => ("sign_in_no_browser", "I couldn't open a browser to sign in."),
            _ => ("sign_in_incomplete", "Sign-in didn't complete. Please try again."),
        };
        return SettingsResult::error(&t_or(key, &[], fallback)).to_json();
    }
    if res.get("state") != Some(state.as_str()) || res.get("error").is_some() {
        return SettingsResult::error(&t_or("sign_in_unverified", &[], "Sign-in couldn't be verified.")).to_json();
    }
    let code = match res.get("code") {
        Some(c) if !c.is_empty() => c.to_string(),
        _ => return SettingsResult::error(&t_or("sign_in_unverified", &[], "Sign-in couldn't be verified.")).to_json(),
    };

    let body = logic::build_exchange_body(&code, logic::OAUTH_CLIENT_ID, &verifier, &redirect_uri);
    let resp = ari::http_request(
        "POST",
        &logic::token_endpoint(&base_url),
        &[("Content-Type", "application/x-www-form-urlencoded")],
        Some(&body),
    );
    if resp.status < 200 || resp.status >= 300 {
        let key = if private { "sign_in_no_internet" } else { "sign_in_unverified" };
        return SettingsResult::error(&t_or(key, &[], "Sign-in couldn't be verified.")).to_json();
    }
    let tokens = match resp.body.as_deref().and_then(logic::parse_token_response) {
        Some(t) => t,
        None => return SettingsResult::error(&t_or("sign_in_unverified", &[], "Sign-in couldn't be verified.")).to_json(),
    };
    let refresh = match tokens.refresh_token {
        Some(r) => r,
        None => return SettingsResult::error(&t_or("sign_in_unverified", &[], "Sign-in couldn't be verified.")).to_json(),
    };

    ari::storage_set("refresh_token", &refresh);
    ari::storage_set("auth_mode", "oauth");
    // The visible `token` setting is manual-entry-only. Clear any stale
    // manual token so the OAuth path doesn't leave a value in the
    // "Use token authentication instead" field the user never typed.
    ari::setting_set("token", "");
    ari::storage_set("access_token", &tokens.access_token);
    ari::storage_set("access_expires_at", &alloc::format!("{}", ari::now_ms() + (tokens.expires_in as i64) * 1000));
    ari::storage_set("needs_reauth", "0");

    SettingsResult::validated(&t_or("signed_in", &[], "Signed in to Home Assistant."))
        .with_refresh()
        .to_json()
}

/// CSPRNG base64url string for the PKCE verifier (32 bytes -> 43 chars).
#[cfg(target_arch = "wasm32")]
fn oauth_verifier() -> String {
    let mut bytes = [0u8; 32];
    for chunk in bytes.chunks_mut(8) {
        let r = ari::rand_u64().to_le_bytes();
        chunk.copy_from_slice(&r[..chunk.len()]);
    }
    ari::crypto::base64url_nopad(&bytes)
}

/// Random URL-safe state value (128-bit entropy).
#[cfg(target_arch = "wasm32")]
fn oauth_state() -> String {
    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&ari::rand_u64().to_le_bytes());
    bytes[8..].copy_from_slice(&ari::rand_u64().to_le_bytes());
    ari::crypto::base64url_nopad(&bytes)
}

#[cfg(target_arch = "wasm32")]
use alloc::string::{String, ToString};

#[cfg(target_arch = "wasm32")]
fn t_or(key: &str, args: &[(&str, &str)], fallback: &str) -> String {
    ari::t(key, args).unwrap_or(fallback).to_string()
}

#[cfg(target_arch = "wasm32")]
enum Bearer {
    Token(String),
    Reauth,
}

#[cfg(target_arch = "wasm32")]
fn resolve_bearer(base_url: &str) -> Bearer {
    let auth_mode = ari::storage_get("auth_mode");
    let cached_access = ari::storage_get("access_token");
    let cached_exp = ari::storage_get("access_expires_at").and_then(|s| s.parse::<i64>().ok());
    let manual_token = ari::setting_get("token");
    let refresh_token = ari::storage_get("refresh_token");
    match logic::plan_bearer(auth_mode, cached_access, cached_exp, ari::now_ms(), manual_token, refresh_token) {
        logic::BearerPlan::UseDirect(t) | logic::BearerPlan::UseCached(t) => Bearer::Token(t),
        logic::BearerPlan::NeedsReauth => Bearer::Reauth,
        logic::BearerPlan::Refresh(refresh) => match refresh_access_token(base_url, &refresh) {
            Some(access) => Bearer::Token(access),
            None => Bearer::Reauth,
        },
    }
}

/// POST the refresh grant; cache + return the new access token, or None on failure
/// (also flags needs_reauth + clears the cache).
#[cfg(target_arch = "wasm32")]
fn refresh_access_token(base_url: &str, refresh_token: &str) -> Option<String> {
    let body = logic::build_refresh_body(refresh_token, logic::OAUTH_CLIENT_ID);
    let resp = ari::http_request(
        "POST",
        &logic::token_endpoint(base_url),
        &[("Content-Type", "application/x-www-form-urlencoded")],
        Some(&body),
    );
    if resp.status < 200 || resp.status >= 300 {
        ari::storage_set("access_token", "");
        ari::storage_set("needs_reauth", "1");
        return None;
    }
    let tokens = match resp.body.as_deref().and_then(logic::parse_token_response) {
        Some(t) => t,
        None => {
            ari::storage_set("access_token", "");
            ari::storage_set("needs_reauth", "1");
            return None;
        }
    };
    ari::storage_set("access_token", &tokens.access_token);
    ari::storage_set(
        "access_expires_at",
        &alloc::format!("{}", ari::now_ms() + (tokens.expires_in as i64) * 1000),
    );
    // HA doesn't rotate refresh tokens by default, but persist a new one if sent
    // (the prior refresh token may be invalidated after rotation).
    if let Some(new_refresh) = &tokens.refresh_token {
        ari::storage_set("refresh_token", new_refresh);
    }
    Some(tokens.access_token)
}

#[cfg(target_arch = "wasm32")]
fn is_oauth_mode() -> bool {
    matches!(ari::storage_get("auth_mode"), Some("oauth"))
}

/// Send an HA request; in oauth mode, a 401/403 triggers ONE forced token
/// refresh and a single retry with the fresh access token.
#[cfg(target_arch = "wasm32")]
fn send_with_retry(base_url: &str, req: &logic::HaRequest) -> ari::HttpResponse {
    let (auth_k, auth_v) = req.auth_header();
    let resp = ari::http_request(
        req.method,
        &req.url,
        &[(&auth_k, &auth_v), ("Content-Type", "application/json")],
        Some(&req.body),
    );
    let unauthorized = resp.status == 401 || resp.status == 403;
    if !unauthorized || !is_oauth_mode() {
        return resp;
    }
    let refresh = match ari::storage_get("refresh_token") {
        Some(r) if !r.trim().is_empty() => r,
        _ => return resp,
    };
    let access = match refresh_access_token(base_url, refresh) {
        Some(a) => a,
        None => return resp,
    };
    let auth_v2 = alloc::format!("Bearer {}", access);
    ari::http_request(
        req.method,
        &req.url,
        &[("Authorization", &auth_v2), ("Content-Type", "application/json")],
        Some(&req.body),
    )
}

#[cfg(target_arch = "wasm32")]
fn migrate_refresh_token() {
    // Migration (one-time): pre-0.3.0 builds stored the OAuth refresh token
    // in the visible `token` setting. Move it into internal storage so the
    // user isn't forced to re-authenticate after upgrading, and clear the
    // setting so the manual-token field stops showing a value they never typed.
    if matches!(ari::storage_get("auth_mode"), Some("oauth"))
        && ari::storage_get("refresh_token").map(|s| s.trim().is_empty()).unwrap_or(true)
    {
        if let Some(legacy) = ari::setting_get("token").filter(|s| !s.trim().is_empty()) {
            ari::storage_set("refresh_token", legacy);
            ari::setting_set("token", "");
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn dispatch_wasm(input: &str) -> String {
    // 1. Settings.
    let base_url = match ari::setting_get("base_url") {
        Some(s) if !s.trim().is_empty() => s.to_string(),
        _ => return logic::error_envelope(&t_or("not_configured", &[], "Home Assistant isn't set up yet.")),
    };
    migrate_refresh_token();
    let token = match resolve_bearer(&base_url) {
        Bearer::Token(t) => t,
        Bearer::Reauth => {
            return logic::error_envelope(&t_or(
                "needs_reauth",
                &[],
                "I've lost my connection to Home Assistant — please sign in again in settings.",
            ))
        }
    };
    let language = ari::get_locale().to_string();
    let private = logic::is_private_base_url(&base_url);
    // Optional: a specific HA conversation agent entity (e.g.
    // `conversation.openai_conversation`). Blank → HA's built-in default agent.
    let agent_id = ari::setting_get("agent_id")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    match logic::classify(input) {
        logic::Intent::PersonLocation { name } => person_flow(&base_url, &token, &name, private),
        logic::Intent::Forward => forward_flow(&base_url, &token, input, &language, agent_id, private),
    }
}

#[cfg(target_arch = "wasm32")]
fn forward_flow(base_url: &str, token: &str, input: &str, language: &str, agent_id: Option<&str>, private: bool) -> String {
    let req = logic::build_conversation_request(base_url, token, input, language, agent_id);
    let resp = send_with_retry(base_url, &req);
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
    let resp = send_with_retry(base_url, &req);
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
