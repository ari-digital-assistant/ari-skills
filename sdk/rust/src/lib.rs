#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};

// ---------------------------------------------------------------------------
// Bump allocator
//
// Serves two masters:
// 1. The host calls `ari_alloc` to stage input strings and write back
//    http_fetch / storage_get responses into our linear memory.
// 2. Rust's `alloc` crate (String, Vec, format!) goes through GlobalAlloc.
//
// Bump-only, never frees. Safe because the WASM store is fresh per call —
// the entire linear memory is discarded after each score/execute invocation.
// ---------------------------------------------------------------------------

extern "C" {
    static __heap_base: u8;
}

static mut BUMP: u32 = 0;

fn bump_alloc(size: u32, align: u32) -> *mut u8 {
    unsafe {
        if BUMP == 0 {
            BUMP = &__heap_base as *const u8 as u32;
        }
        let aligned = (BUMP + align - 1) & !(align - 1);
        BUMP = aligned + size;
        aligned as *mut u8
    }
}

#[no_mangle]
pub extern "C" fn ari_alloc(size: i32) -> i32 {
    bump_alloc(size as u32, 1) as i32
}

struct BumpAlloc;

unsafe impl GlobalAlloc for BumpAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        bump_alloc(layout.size() as u32, layout.align() as u32)
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOC: BumpAlloc = BumpAlloc;

#[cfg(all(target_arch = "wasm32", not(feature = "std")))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// ---------------------------------------------------------------------------
// Input / output marshalling
// ---------------------------------------------------------------------------

/// Read the UTF-8 input the host wrote at `(ptr, len)`.
///
/// # Safety
/// The caller must pass the exact `(ptr, len)` pair received from score() or
/// execute(). The host guarantees these point to valid UTF-8 in linear memory.
pub unsafe fn input(ptr: i32, len: i32) -> &'static str {
    let slice = core::slice::from_raw_parts(ptr as *const u8, len as usize);
    core::str::from_utf8_unchecked(slice)
}

/// Pack a response string for return from execute().
/// Copies the bytes into bump-allocated memory and returns the packed
/// `(ptr << 32) | len` value the host expects.
pub fn respond(s: &str) -> i64 {
    let bytes = s.as_bytes();
    let ptr = ari_alloc(bytes.len() as i32);
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
    }
    ((ptr as i64) << 32) | (bytes.len() as i64)
}

/// Unpack a host-returned `(ptr << 32) | len` into a `&str`.
/// Returns `None` if the packed value is 0 (sentinel for "not found").
unsafe fn unpack(packed: i64) -> Option<&'static str> {
    if packed == 0 {
        return None;
    }
    let ptr = (packed >> 32) as i32;
    let len = (packed & 0xFFFF_FFFF) as i32;
    Some(input(ptr, len))
}

// ---------------------------------------------------------------------------
// Logging
// ---------------------------------------------------------------------------

#[repr(i32)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

#[link(wasm_import_module = "ari")]
extern "C" {
    #[link_name = "log"]
    fn host_log(level: i32, ptr: i32, len: i32);

    #[link_name = "get_capability"]
    fn host_get_capability(name_ptr: i32, name_len: i32) -> i32;
}

pub fn log(level: LogLevel, msg: &str) {
    let bytes = msg.as_bytes();
    unsafe { host_log(level as i32, bytes.as_ptr() as i32, bytes.len() as i32) }
}

/// Returns true if the named capability is both declared by this skill
/// and granted by the host.
pub fn has_capability(name: &str) -> bool {
    let bytes = name.as_bytes();
    unsafe { host_get_capability(bytes.as_ptr() as i32, bytes.len() as i32) == 1 }
}

// ---------------------------------------------------------------------------
// HTTP (feature = "http")
// ---------------------------------------------------------------------------

#[cfg(feature = "http")]
mod http_impl {
    #[link(wasm_import_module = "ari")]
    extern "C" {
        #[link_name = "http_fetch"]
        fn host_http_fetch(url_ptr: i32, url_len: i32) -> i64;
    }

    pub struct HttpResponse<'a> {
        pub status: u16,
        pub body: Option<&'a str>,
        pub error: Option<&'a str>,
    }

    /// Perform an HTTP GET. The host enforces scheme restrictions (default:
    /// HTTPS only) and body size limits.
    ///
    /// Returns an `HttpResponse` with the status code and body. On network
    /// errors, `status` is 0 and `error` contains the message.
    pub fn http_fetch(url: &str) -> HttpResponse<'static> {
        let bytes = url.as_bytes();
        let packed = unsafe { host_http_fetch(bytes.as_ptr() as i32, bytes.len() as i32) };
        let json = unsafe { super::unpack(packed) };
        match json {
            Some(s) => parse_http_response(s),
            None => HttpResponse { status: 0, body: None, error: None },
        }
    }

    // The host writes JSON: {"status":200,"body":"..."} or
    // {"status":0,"body":null,"error":"..."}
    // Hand-rolled because we're no_std with zero deps.
    fn parse_http_response(json: &str) -> HttpResponse<'_> {
        let status = parse_status(json);
        let body = extract_json_string(json, "\"body\":");
        let error = extract_json_string(json, "\"error\":");
        HttpResponse { status, body, error }
    }

    fn parse_status(json: &str) -> u16 {
        if let Some(pos) = json.find("\"status\":") {
            let rest = &json[pos + 9..];
            let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
            if let Ok(n) = u16::from_str(rest[..end].trim()) {
                return n;
            }
        }
        0
    }

    fn extract_json_string<'a>(json: &'a str, key: &str) -> Option<&'a str> {
        let pos = json.find(key)?;
        let rest = &json[pos + key.len()..];
        let trimmed = rest.trim_start();
        if trimmed.starts_with("null") {
            return None;
        }
        if !trimmed.starts_with('"') {
            return None;
        }
        let inner = &trimmed[1..];
        // Find the closing quote (not preceded by backslash)
        let mut i = 0;
        let bytes = inner.as_bytes();
        while i < bytes.len() {
            if bytes[i] == b'\\' {
                i += 2;
                continue;
            }
            if bytes[i] == b'"' {
                return Some(&inner[..i]);
            }
            i += 1;
        }
        None
    }

    // core doesn't have u16::from_str, so we use a minimal version
    trait FromStrMinimal: Sized {
        fn from_str(s: &str) -> Result<Self, ()>;
    }

    impl FromStrMinimal for u16 {
        fn from_str(s: &str) -> Result<Self, ()> {
            let mut n: u16 = 0;
            for b in s.bytes() {
                if !b.is_ascii_digit() {
                    return Err(());
                }
                n = n.checked_mul(10).ok_or(())?.checked_add((b - b'0') as u16).ok_or(())?;
            }
            Ok(n)
        }
    }
}

#[cfg(feature = "http")]
pub use http_impl::{http_fetch, HttpResponse};

// ---------------------------------------------------------------------------
// Storage (feature = "storage")
// ---------------------------------------------------------------------------

#[cfg(feature = "storage")]
mod storage_impl {
    #[link(wasm_import_module = "ari")]
    extern "C" {
        #[link_name = "storage_get"]
        fn host_storage_get(key_ptr: i32, key_len: i32) -> i64;

        #[link_name = "storage_set"]
        fn host_storage_set(
            key_ptr: i32,
            key_len: i32,
            val_ptr: i32,
            val_len: i32,
        ) -> i32;
    }

    /// Read a value from this skill's key-value store.
    /// Returns `None` if the key doesn't exist.
    pub fn storage_get(key: &str) -> Option<&'static str> {
        let kb = key.as_bytes();
        let packed = unsafe { host_storage_get(kb.as_ptr() as i32, kb.len() as i32) };
        unsafe { super::unpack(packed) }
    }

    /// Write a value to this skill's key-value store.
    /// Returns `true` on success, `false` on any failure (key/value too
    /// large, total storage cap exceeded, I/O error).
    pub fn storage_set(key: &str, value: &str) -> bool {
        let kb = key.as_bytes();
        let vb = value.as_bytes();
        let rc = unsafe {
            host_storage_set(
                kb.as_ptr() as i32,
                kb.len() as i32,
                vb.as_ptr() as i32,
                vb.len() as i32,
            )
        };
        rc == 0
    }
}

#[cfg(feature = "storage")]
pub use storage_impl::{storage_get, storage_set};
