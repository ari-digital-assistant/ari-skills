#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::string::ToString;
use ari_skill_sdk as ari;
use serde_json::json;

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    0.95
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    let envelope = json!({
        "action": "open",
        "speak": alloc::format!("Opening {input}."),
        "target": input,
    });
    ari::respond_action(&envelope.to_string())
}
