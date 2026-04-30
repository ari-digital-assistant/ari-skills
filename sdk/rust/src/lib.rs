#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "presentation")]
pub mod presentation;

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
//
// Only compiled for wasm32, so unit tests running on the host linker don't
// have to resolve `__heap_base` (which is a wasm-only linker-provided symbol).
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
mod bump {
    use core::alloc::{GlobalAlloc, Layout};

    extern "C" {
        static __heap_base: u8;
    }

    /// One WASM linear-memory page = 64 KiB.
    const PAGE_BYTES: u32 = 65_536;

    static mut BUMP: u32 = 0;

    /// Allocate `size` bytes aligned to `align` from the bump arena.
    /// Grows the WASM linear memory via `memory.grow` when the next
    /// allocation would run past the current end-of-memory; without
    /// this growth a long-running skill (or any skill that produces
    /// big format! / String buffers in a single call) would silently
    /// return a pointer outside addressable memory and the next write
    /// would trap.
    ///
    /// Never frees — bump-only by design. The whole arena is reset
    /// implicitly when the host re-instantiates the skill module.
    pub fn bump_alloc(size: u32, align: u32) -> *mut u8 {
        unsafe {
            if BUMP == 0 {
                BUMP = &__heap_base as *const u8 as u32;
            }
            let aligned = (BUMP + align - 1) & !(align - 1);
            let new_bump = aligned + size;

            // memory.size returns the current linear memory size in
            // pages (64 KiB each). If our allocation would land past
            // the end, grow by enough pages to cover it.
            #[cfg(target_arch = "wasm32")]
            {
                let current_bytes = (core::arch::wasm32::memory_size(0) as u32) * PAGE_BYTES;
                if new_bump > current_bytes {
                    let extra_bytes = new_bump - current_bytes;
                    let extra_pages = (extra_bytes + PAGE_BYTES - 1) / PAGE_BYTES;
                    // memory.grow returns -1 on failure. If it fails
                    // there's nothing reasonable we can do from a
                    // panic-handler-less no_std skill, so trap by
                    // returning a null pointer; the caller's write
                    // will trap with a clear out-of-bounds rather
                    // than silently corrupting memory.
                    let prev = core::arch::wasm32::memory_grow(0, extra_pages as usize);
                    if prev == usize::MAX {
                        return core::ptr::null_mut();
                    }
                }
            }

            BUMP = new_bump;
            aligned as *mut u8
        }
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
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn ari_alloc(size: i32) -> i32 {
    bump::bump_alloc(size as u32, 1) as i32
}


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

pub const RESPONSE_TAG_TEXT: u8 = 0x00;
pub const RESPONSE_TAG_ACTION: u8 = 0x01;

/// Pack a text response for return from execute().
/// Copies the bytes into bump-allocated memory and returns the packed
/// `tag|ptr|len` value the host expects. The tag byte is 0x00 (implicit
/// via the zero top byte) so this is the same wire format the old
/// text-only ABI used — any existing skill that called the previous
/// `respond` continues to compile and behave identically.
///
/// Only defined for `wasm32` because the allocator and the "linear memory"
/// concept the host decodes against only exist there.
#[cfg(target_arch = "wasm32")]
pub fn respond_text(s: &str) -> i64 {
    pack_response(RESPONSE_TAG_TEXT, s.as_bytes())
}

/// Pack an action response (UTF-8 JSON) for return from execute().
/// The host decodes the payload into a `serde_json::Value` and wraps
/// it in `Response::Action`. See the host ABI docs for the expected
/// envelope shape (`{"action": "...", "speak": "...", ...}`).
#[cfg(target_arch = "wasm32")]
pub fn respond_action(json: &str) -> i64 {
    pack_response(RESPONSE_TAG_ACTION, json.as_bytes())
}

/// Deprecated: prefer `respond_text`. Kept so examples written before the
/// tagged ABI still compile. Will be removed after the first tagged-ABI
/// release cut.
#[cfg(target_arch = "wasm32")]
#[deprecated(note = "use respond_text")]
pub fn respond(s: &str) -> i64 {
    respond_text(s)
}

#[cfg(target_arch = "wasm32")]
fn pack_response(tag: u8, bytes: &[u8]) -> i64 {
    let ptr = ari_alloc(bytes.len() as i32);
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
    }
    // Layout must match the host-side decoder in `ari-skill-loader`:
    //   bits 63..56 = tag, 55..32 = ptr (24-bit), 31..0 = len (32-bit)
    let tag = (tag as i64) << 56;
    let ptr = ((ptr as i64) & 0x00FF_FFFF) << 32;
    let len = (bytes.len() as i64) & 0xFFFF_FFFF;
    tag | ptr | len
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

    #[link_name = "now_ms"]
    fn host_now_ms() -> i64;

    #[link_name = "rand_u64"]
    fn host_rand_u64() -> i64;

    #[link_name = "setting_get"]
    fn host_setting_get(key_ptr: i32, key_len: i32) -> i64;

    #[link_name = "args"]
    fn host_args() -> i64;

    #[link_name = "get_locale"]
    fn host_get_locale() -> i64;

    #[link_name = "t"]
    fn host_t(key_ptr: i32, key_len: i32, args_ptr: i32, args_len: i32) -> i64;
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

/// Current Unix time in milliseconds, as seen by the host.
///
/// This is wall-clock time (not a monotonic clock) — the host reads
/// `SystemTime::now()` every call, so it reflects clock changes, DST, etc.
/// Good enough for timers, timestamps, and "when did the user last ask me
/// something"; not good enough for performance measurement.
pub fn now_ms() -> i64 {
    unsafe { host_now_ms() }
}

/// 64 bits of cryptographically-random entropy from the host.
///
/// Use for ids, tokens, and anything where predictability matters. The
/// caller does not need to seed or reset anything — each call is independent.
pub fn rand_u64() -> u64 {
    unsafe { host_rand_u64() as u64 }
}

/// Read one of this skill's user-configurable settings (as declared
/// in SKILL.md under `metadata.ari.settings`). Returns `None` if the
/// key hasn't been set by the user yet — skills should treat that as
/// "use your documented default".
///
/// Scoped to the calling skill's id — you can't read another skill's
/// settings. Always available (no capability declaration required).
pub fn setting_get(key: &str) -> Option<&'static str> {
    let bytes = key.as_bytes();
    let packed = unsafe { host_setting_get(bytes.as_ptr() as i32, bytes.len() as i32) };
    unsafe { unpack(packed) }
}

/// Typed JSON args extracted from the user's utterance by the
/// FunctionGemma skill router. Returns `Some(json)` when the skill
/// was invoked via the router's typed-args path; `None` when invoked
/// via the keyword scorer or with no extracted slots.
///
/// The JSON object's shape matches whatever the skill declared in
/// `parameters_schema()` (built-in skills) or inferred from
/// `metadata.ari.examples[].args` (community skills). For example, a
/// reminder might receive `{"title":"call mum","when":"tomorrow at 3pm"}`.
///
/// Skills using this should still keep their own input parser as a
/// fallback for keyword-scorer dispatches and for cases where the
/// model's extraction is missing fields. Self-report parse-confidence
/// `low` on the response envelope when the args look dodgy and Layer
/// C will consult the assistant for a better extraction.
///
/// Always available (no capability declaration required). Only
/// meaningful inside `execute()` — `score()` is invoked without args.
pub fn args() -> Option<&'static str> {
    let packed = unsafe { host_args() };
    unsafe { unpack(packed) }
}

// ---------------------------------------------------------------------------
// Locale + i18n
// ---------------------------------------------------------------------------

/// The user's currently-active language as an ISO 639-1 lowercase
/// code (`"en"`, `"it"`, …). Read fresh on every call from the host's
/// locale provider — skills that fork their behaviour on language
/// (different parsers, different default settings) should call this
/// once at the top of `execute()` and branch from there.
///
/// Always available, no capability declaration required.
pub fn get_locale() -> &'static str {
    let packed = unsafe { host_get_locale() };
    // The host always writes *something* (defaulting to "en" when no
    // provider is wired). A 0-packed return would mean the wasm
    // memory export is broken, which the SDK can't recover from
    // — fall through to "en" as a last-resort sentinel rather than
    // panicking.
    unsafe { unpack(packed) }.unwrap_or("en")
}

/// Look up a translation key in the skill's `strings/{locale}.json`
/// table for the user's active locale, falling back to English when
/// the requested locale doesn't have the key. Substitutes
/// `{placeholder}` slots from the `args` slice.
///
/// On a full miss (key absent in both the active locale and English),
/// the host returns the bare key as the resolved string — visible-
/// failure UX so a typo stays visible rather than rendering empty.
/// `None` only on the degenerate case where the host can't read this
/// skill's WASM memory at all (effectively unreachable from a
/// running skill); pair with `.unwrap_or(...)` to a fallback string
/// at the call site.
///
/// `args` is a slice of `(name, value)` string pairs. The SDK
/// serialises it to a flat JSON object before crossing the WASM
/// boundary; numeric placeholders should be stringified by the
/// caller (`{"count": "3"}`). Empty args is fine — many keys have no
/// placeholders.
///
/// Always available, no capability declaration required.
///
/// ```ignore
/// let greeting = ari::t("greet.hello", &[("name", "Keith")])
///     .unwrap_or("Hello!");
/// // → "Hi Keith!" in English, "Ciao Keith!" in Italian, …
/// ```
pub fn t(key: &str, args: &[(&str, &str)]) -> Option<&'static str> {
    let json = encode_args_json(args);
    let key_bytes = key.as_bytes();
    let json_bytes = json.as_bytes();
    let packed = unsafe {
        host_t(
            key_bytes.as_ptr() as i32,
            key_bytes.len() as i32,
            json_bytes.as_ptr() as i32,
            json_bytes.len() as i32,
        )
    };
    unsafe { unpack(packed) }
}

/// Encode a slice of (name, value) pairs as a flat JSON object.
/// Hand-rolled to keep `t()` available without the `serde` features
/// — it's a core capability every skill might want, and forcing the
/// `serde_json` dep on every translation call would bloat lean text-
/// only skills.
fn encode_args_json(args: &[(&str, &str)]) -> String {
    if args.is_empty() {
        return String::new();
    }
    let mut s = String::with_capacity(args.len() * 32);
    s.push('{');
    for (i, (k, v)) in args.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        json_escape_into(&mut s, k);
        s.push(':');
        json_escape_into(&mut s, v);
    }
    s.push('}');
    s
}

fn json_escape_into(out: &mut String, val: &str) {
    out.push('"');
    for c in val.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // ASCII control chars get \uXXXX-escaped; everything
            // else (including all printable Unicode) goes through
            // verbatim — JSON allows raw UTF-8 for non-control chars.
            c if (c as u32) < 0x20 => {
                out.push_str("\\u00");
                let n = c as u32;
                out.push(hex_nibble((n >> 4) & 0xf));
                out.push(hex_nibble(n & 0xf));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn hex_nibble(n: u32) -> char {
    match n {
        0..=9 => (b'0' + n as u8) as char,
        10..=15 => (b'a' + (n - 10) as u8) as char,
        _ => '0',
    }
}

#[cfg(test)]
mod i18n_tests {
    use super::{encode_args_json, hex_nibble, json_escape_into};

    #[test]
    fn encode_args_empty_yields_empty_string() {
        // Convention: empty args → empty JSON, host treats this as
        // "no args" without a parse attempt.
        assert_eq!(encode_args_json(&[]), "");
    }

    #[test]
    fn encode_args_single_pair() {
        assert_eq!(
            encode_args_json(&[("name", "Keith")]),
            r#"{"name":"Keith"}"#
        );
    }

    #[test]
    fn encode_args_multiple_pairs() {
        assert_eq!(
            encode_args_json(&[("name", "Keith"), ("count", "3")]),
            r#"{"name":"Keith","count":"3"}"#
        );
    }

    #[test]
    fn encode_args_escapes_quotes_and_backslashes() {
        // Carefully constructed values — we want both quote and
        // backslash to be escaped properly so the host's
        // serde_json::from_str round-trips.
        let escaped = encode_args_json(&[("k", "she said \"hi\\there\"")]);
        assert_eq!(escaped, r#"{"k":"she said \"hi\\there\""}"#);
    }

    #[test]
    fn encode_args_escapes_control_characters() {
        let mut s = String::new();
        json_escape_into(&mut s, "a\x01b");
        assert_eq!(s, r#""a\u0001b""#);
    }

    #[test]
    fn encode_args_passes_unicode_verbatim() {
        // Italian "Ciao!" with non-ASCII characters should NOT be
        // \uXXXX-escaped — JSON allows raw UTF-8 in strings.
        let mut s = String::new();
        json_escape_into(&mut s, "Ciaò");
        assert_eq!(s, r#""Ciaò""#);
    }

    #[test]
    fn hex_nibble_covers_full_range() {
        assert_eq!(hex_nibble(0), '0');
        assert_eq!(hex_nibble(9), '9');
        assert_eq!(hex_nibble(10), 'a');
        assert_eq!(hex_nibble(15), 'f');
    }
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

// ---------------------------------------------------------------------------
// Platform tasks (feature = "tasks")
// ---------------------------------------------------------------------------

#[cfg(feature = "tasks")]
mod tasks_impl {
    #[cfg(not(feature = "std"))]
    use alloc::{string::String, vec::Vec};

    #[link(wasm_import_module = "ari")]
    extern "C" {
        #[link_name = "tasks_provider_installed"]
        fn host_tasks_provider_installed() -> i32;
        #[link_name = "tasks_list_lists"]
        fn host_tasks_list_lists() -> i64;
        #[link_name = "tasks_insert"]
        fn host_tasks_insert(params_ptr: i32, params_len: i32) -> i64;
        #[link_name = "tasks_delete"]
        fn host_tasks_delete(id: i64) -> i32;
        #[link_name = "tasks_query_in_range"]
        fn host_tasks_query_in_range(start_ms: i64, end_ms: i64, limit: i32) -> i64;
    }

    /// One writable task list the user can target. `id` is the stable
    /// provider-supplied identifier; `account_name` disambiguates when
    /// two lists share a display name (e.g. "Personal" across two
    /// CalDAV accounts). Empty string if the host has no such concept.
    #[derive(Debug, Clone, serde::Deserialize)]
    pub struct TaskList {
        pub id: u64,
        pub display_name: String,
        #[serde(default)]
        pub account_name: String,
    }

    /// Parameters for [`tasks_insert`]. Separate struct so adding
    /// fields later doesn't break the ABI.
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct InsertTaskParams<'a> {
        pub list_id: u64,
        pub title: &'a str,
        /// UTC epoch ms. `None` for untimed tasks (shopping list etc).
        #[serde(skip_serializing_if = "Option::is_none")]
        pub due_ms: Option<i64>,
        /// When true, `due_ms` is interpreted as a wall-clock date;
        /// the time portion is ignored by the provider.
        pub due_all_day: bool,
        /// IANA timezone id (`"Europe/London"`). Required by some
        /// providers when `due_ms` is a precise instant; ignored for
        /// all-day tasks. `None` if the skill doesn't know.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tz_id: Option<&'a str>,
    }

    /// Is any task provider available on this host right now? Returns
    /// false on hosts that don't implement the capability at all, and
    /// on hosts whose backing provider isn't installed (e.g. Android
    /// without Tasks.org / OpenTasks). The skill should call this
    /// before list/insert/delete and degrade gracefully if false.
    pub fn tasks_provider_installed() -> bool {
        unsafe { host_tasks_provider_installed() == 1 }
    }

    /// All writable task lists the skill can target. Empty when the
    /// provider isn't installed or has no lists configured yet.
    pub fn tasks_list_lists() -> Vec<TaskList> {
        let packed = unsafe { host_tasks_list_lists() };
        let Some(json) = (unsafe { super::unpack(packed) }) else {
            return Vec::new();
        };
        serde_json::from_str(json).unwrap_or_default()
    }

    /// Insert a task. Returns the provider's row id on success;
    /// `None` on permission failure / invalid list / IO error. The
    /// host logs details via the log sink.
    pub fn tasks_insert(params: &InsertTaskParams<'_>) -> Option<u64> {
        let json = serde_json::to_string(params).ok()?;
        let bytes = json.as_bytes();
        let packed = unsafe { host_tasks_insert(bytes.as_ptr() as i32, bytes.len() as i32) };
        if packed == 0 {
            None
        } else {
            Some(packed as u64)
        }
    }

    /// Hard-delete a task by its provider row id. Returns true if the
    /// row existed and was removed; false on permission / IO failure
    /// or if the id doesn't exist.
    pub fn tasks_delete(id: u64) -> bool {
        unsafe { host_tasks_delete(id as i64) == 1 }
    }

    /// One row from [`tasks_query_in_range`]. Always-timed: untimed
    /// tasks (no `due` value) don't appear in range queries.
    #[derive(Debug, Clone, serde::Deserialize)]
    pub struct TaskRow {
        pub id: u64,
        pub title: String,
        /// UTC epoch ms for the task's due time.
        pub due_ms: i64,
        /// True when only the date portion of `due_ms` is meaningful
        /// (the provider stored it as an all-day task).
        pub due_all_day: bool,
        pub list_id: u64,
    }

    /// Tasks with due time in `[start_ms, end_ms)`, ordered by due
    /// ascending and capped at `limit`. Empty Vec when no provider
    /// is installed, the read permission is missing, or the range
    /// is empty.
    pub fn tasks_query_in_range(start_ms: i64, end_ms: i64, limit: u32) -> Vec<TaskRow> {
        let packed =
            unsafe { host_tasks_query_in_range(start_ms, end_ms, limit as i32) };
        let Some(json) = (unsafe { super::unpack(packed) }) else {
            return Vec::new();
        };
        serde_json::from_str(json).unwrap_or_default()
    }
}

#[cfg(feature = "tasks")]
pub use tasks_impl::{
    tasks_delete, tasks_insert, tasks_list_lists, tasks_provider_installed,
    tasks_query_in_range, InsertTaskParams, TaskList, TaskRow,
};

// ---------------------------------------------------------------------------
// Platform calendar (feature = "calendar")
// ---------------------------------------------------------------------------

#[cfg(feature = "calendar")]
mod calendar_impl {
    #[cfg(not(feature = "std"))]
    use alloc::{string::String, vec::Vec};

    #[link(wasm_import_module = "ari")]
    extern "C" {
        #[link_name = "calendar_has_write_permission"]
        fn host_calendar_has_write_permission() -> i32;
        #[link_name = "calendar_list_calendars"]
        fn host_calendar_list_calendars() -> i64;
        #[link_name = "calendar_insert"]
        fn host_calendar_insert(params_ptr: i32, params_len: i32) -> i64;
        #[link_name = "calendar_delete"]
        fn host_calendar_delete(id: i64) -> i32;
        #[link_name = "calendar_query_in_range"]
        fn host_calendar_query_in_range(start_ms: i64, end_ms: i64, limit: i32) -> i64;
    }

    /// One writable calendar the user can target.
    #[derive(Debug, Clone, serde::Deserialize)]
    pub struct Calendar {
        pub id: u64,
        pub display_name: String,
        #[serde(default)]
        pub account_name: String,
        /// ARGB colour (Android's native format); `None` if the host
        /// doesn't expose one.
        #[serde(default)]
        pub color_argb: Option<i32>,
    }

    /// Parameters for [`calendar_insert`].
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct InsertCalendarEventParams<'a> {
        pub calendar_id: u64,
        pub title: &'a str,
        /// UTC epoch ms the event starts at.
        pub start_ms: i64,
        /// Event length. Most providers require non-zero.
        pub duration_minutes: u32,
        /// Reminder offset in minutes. 0 = no reminder.
        pub reminder_minutes_before: u32,
        /// IANA timezone id. Provider stores this in `EVENT_TIMEZONE`.
        pub tz_id: &'a str,
    }

    /// Does the host have write permission to at least one calendar?
    /// On Android this reflects the runtime `WRITE_CALENDAR` grant.
    pub fn calendar_has_write_permission() -> bool {
        unsafe { host_calendar_has_write_permission() == 1 }
    }

    pub fn calendar_list_calendars() -> Vec<Calendar> {
        let packed = unsafe { host_calendar_list_calendars() };
        let Some(json) = (unsafe { super::unpack(packed) }) else {
            return Vec::new();
        };
        serde_json::from_str(json).unwrap_or_default()
    }

    pub fn calendar_insert(params: &InsertCalendarEventParams<'_>) -> Option<u64> {
        let json = serde_json::to_string(params).ok()?;
        let bytes = json.as_bytes();
        let packed = unsafe { host_calendar_insert(bytes.as_ptr() as i32, bytes.len() as i32) };
        if packed == 0 {
            None
        } else {
            Some(packed as u64)
        }
    }

    pub fn calendar_delete(id: u64) -> bool {
        unsafe { host_calendar_delete(id as i64) == 1 }
    }

    /// One row from [`calendar_query_in_range`]. Recurring events
    /// expand to one row per concrete instance whose start lands in
    /// the queried window.
    #[derive(Debug, Clone, serde::Deserialize)]
    pub struct CalendarEventRow {
        pub id: u64,
        pub title: String,
        /// UTC epoch ms the instance starts at.
        pub start_ms: i64,
        /// UTC epoch ms the instance ends at. May equal `start_ms`
        /// when the provider doesn't store a duration.
        pub end_ms: i64,
        pub all_day: bool,
        pub calendar_id: u64,
    }

    /// Event instances starting in `[start_ms, end_ms)`, ordered by
    /// start ascending and capped at `limit`.
    pub fn calendar_query_in_range(
        start_ms: i64,
        end_ms: i64,
        limit: u32,
    ) -> Vec<CalendarEventRow> {
        let packed =
            unsafe { host_calendar_query_in_range(start_ms, end_ms, limit as i32) };
        let Some(json) = (unsafe { super::unpack(packed) }) else {
            return Vec::new();
        };
        serde_json::from_str(json).unwrap_or_default()
    }
}

#[cfg(feature = "calendar")]
pub use calendar_impl::{
    calendar_delete, calendar_has_write_permission, calendar_insert, calendar_list_calendars,
    calendar_query_in_range, CalendarEventRow,
    Calendar, InsertCalendarEventParams,
};

// ---------------------------------------------------------------------------
// Local clock (feature = "clock")
// ---------------------------------------------------------------------------

#[cfg(feature = "clock")]
mod clock_impl {
    #[cfg(not(feature = "std"))]
    use alloc::string::{String, ToString};

    #[link(wasm_import_module = "ari")]
    extern "C" {
        #[link_name = "local_now_components"]
        fn host_local_now_components() -> i64;
        #[link_name = "local_timezone_id"]
        fn host_local_timezone_id() -> i64;
    }

    /// Current datetime broken into local-timezone components. Returned
    /// by [`local_now_components`]. A skill uses these to interpret
    /// "today", "next Friday", "on the 27th" relative to the user's
    /// timezone — which the skill can't compute itself from
    /// [`now_ms`] alone because WASM has no TZ database.
    #[derive(Debug, Clone, serde::Deserialize)]
    pub struct LocalTimeComponents {
        pub year: i32,
        /// 1..=12
        pub month: u8,
        /// 1..=31
        pub day: u8,
        /// 0..=23
        pub hour: u8,
        /// 0..=59
        pub minute: u8,
        /// 0..=59
        pub second: u8,
        /// ISO weekday: 0=Monday..6=Sunday.
        pub weekday: u8,
        /// IANA timezone id, or `"UTC"` on hosts with no TZ database.
        pub tz_id: String,
    }

    /// Read the current datetime in the host's local timezone.
    /// Always available (no capability required). Returns an all-zero
    /// epoch-style fallback if the marshalling fails, which shouldn't
    /// happen on a correctly-implemented host.
    pub fn local_now_components() -> LocalTimeComponents {
        let packed = unsafe { host_local_now_components() };
        let Some(json) = (unsafe { super::unpack(packed) }) else {
            return LocalTimeComponents {
                year: 1970,
                month: 1,
                day: 1,
                hour: 0,
                minute: 0,
                second: 0,
                weekday: 3, // 1970-01-01 was a Thursday
                tz_id: "UTC".into(),
            };
        };
        serde_json::from_str(json).unwrap_or(LocalTimeComponents {
            year: 1970,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            weekday: 3,
            tz_id: "UTC".into(),
        })
    }

    /// IANA timezone id for the host's current locale, or `"UTC"` on
    /// hosts without a TZ database.
    pub fn local_timezone_id() -> String {
        let packed = unsafe { host_local_timezone_id() };
        unsafe { super::unpack(packed) }
            .map(|s| s.to_string())
            .unwrap_or_else(|| "UTC".into())
    }
}

#[cfg(feature = "clock")]
pub use clock_impl::{local_now_components, local_timezone_id, LocalTimeComponents};

// ---------------------------------------------------------------------------
// Unit tests (host-side only — these cover pure pack/decode logic, not the
// wasm32 ABI, so they run under the std default feature).
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    fn decode_packed(packed: i64) -> (u8, i32, i32) {
        let tag = ((packed as u64) >> 56) as u8;
        let ptr = (((packed as u64) >> 32) & 0x00FF_FFFF) as i32;
        let len = (packed as u64 & 0xFFFF_FFFF) as i32;
        (tag, ptr, len)
    }

    #[test]
    fn pack_response_text_sets_tag_zero() {
        // Simulate what respond_text would produce if ari_alloc returned, say,
        // ptr 2048 for a 10-byte payload. The tag byte MUST be zero so older
        // skills (and host fixtures) keep round-tripping as Response::Text.
        let tag = (RESPONSE_TAG_TEXT as i64) << 56;
        let ptr = (2048_i64 & 0x00FF_FFFF) << 32;
        let len = 10_i64;
        let packed = tag | ptr | len;
        assert_eq!(decode_packed(packed), (0x00, 2048, 10));
    }

    #[test]
    fn pack_response_action_sets_tag_one() {
        let tag = (RESPONSE_TAG_ACTION as i64) << 56;
        let ptr = (4096_i64 & 0x00FF_FFFF) << 32;
        let len = 42_i64;
        let packed = tag | ptr | len;
        assert_eq!(decode_packed(packed), (0x01, 4096, 42));
    }
}
