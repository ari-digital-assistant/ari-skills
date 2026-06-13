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

// TEMPORARY stub so the crate compiles in this scaffold task. Replaced by the
// real orchestration in a later task.
#[cfg(target_arch = "wasm32")]
fn dispatch_wasm(_input: &str) -> alloc::string::String {
    alloc::string::String::from("{\"v\":1}")
}
