#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

mod conditions;
mod forecast;
mod openmeteo;

#[cfg(target_arch = "wasm32")]
use ari_skill_sdk as ari;

/// Ceremonial — the manifest's `matching.patterns` score this skill
/// (`custom_score: false`), so the host never calls this export.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    0.95
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let _input = unsafe { ari::input(ptr, len) };
    ari::respond_text("Weather skill not yet implemented.")
}
