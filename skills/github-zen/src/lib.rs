#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use ari_skill_sdk as ari;

const ZEN_URL: &str = "https://api.github.com/zen";

/// Ceremonial — the manifest's `matching.patterns` score this skill
/// (`custom_score: false`), so the host never calls this export.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    0.95
}

/// Speaks the zen line. `http_fetch` hands back a `{"status":…,"body":…}`
/// envelope; what the user asked for is the body inside it.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(_ptr: i32, _len: i32) -> i64 {
    let response = ari::http_fetch(ZEN_URL);
    let zen = response.body.as_deref().unwrap_or("").trim();
    if (200..300).contains(&response.status) && !zen.is_empty() {
        return ari::respond_text(zen);
    }

    ari::log(
        ari::LogLevel::Warn,
        &alloc::format!(
            "github zen fetch failed: status={} error={}",
            response.status,
            response.error.as_deref().unwrap_or("(none)"),
        ),
    );
    ari::respond_text(
        ari::t("unavailable", &[]).unwrap_or("I couldn't reach GitHub for a piece of wisdom."),
    )
}
