#![cfg_attr(target_arch = "wasm32", no_std)]

use ari_skill_sdk as ari;

/// Ceremonial — the manifest's `matching.patterns` score this skill
/// (`custom_score: false`), so the host never calls this export.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    0.95
}

/// Returns the localized greeting from `strings/{locale}.json` (key
/// `greeting`), falling back to the English literal. Demonstrates the
/// canonical WASM-skill localization path via `ari::t`.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let _input = unsafe { ari::input(ptr, len) };
    let greeting = ari::t("greeting", &[]).unwrap_or("wasm hello");
    ari::respond_text(greeting)
}
