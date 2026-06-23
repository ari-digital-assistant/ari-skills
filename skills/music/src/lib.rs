#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

mod parse;

use ari_skill_sdk as ari;

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    0.9
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let _input = unsafe { ari::input(ptr, len) };
    ari::respond_text("music skill stub")
}
