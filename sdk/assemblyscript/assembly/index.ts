// Ari Skill SDK for AssemblyScript
//
// Core module: bump allocator, input/output marshalling, logging.
// Import http.ts or storage.ts separately if your skill needs those
// capabilities — keeping them in separate files prevents the WASM module
// from importing host functions it doesn't use (which would trip the
// host's sneak guard).

// ---------------------------------------------------------------------------
// Bump allocator
//
// Separate from AS's managed heap. The host calls ari_alloc to stage
// input strings and write back http_fetch / storage_get responses.
// We use heap.alloc (unmanaged, not GC'd) to hand out raw pointers.
// ---------------------------------------------------------------------------

export function ari_alloc(size: i32): i32 {
  return heap.alloc(size as usize) as i32;
}

// ---------------------------------------------------------------------------
// Input / output marshalling
// ---------------------------------------------------------------------------

/** Read the UTF-8 input the host wrote at (ptr, len). */
export function input(ptr: i32, len: i32): string {
  return String.UTF8.decodeUnsafe(ptr as usize, len as usize);
}

// Response tag bytes. Match the host-side decoder in `ari-skill-loader`.
export const RESPONSE_TAG_TEXT: u8 = 0x00;
export const RESPONSE_TAG_ACTION: u8 = 0x01;

/**
 * Pack a text response for return from execute().
 * Encodes to UTF-8 in bump-allocated memory and returns the tagged
 * `tag | ptr | len` value the host expects. Tag byte is 0x00 (implicit
 * via the zero top byte of the returned i64) so the wire format matches
 * the pre-tagged ABI and older skills continue to round-trip as text.
 */
export function respondText(s: string): i64 {
  return packResponse(RESPONSE_TAG_TEXT, s);
}

/**
 * Pack an action response (UTF-8 JSON) for return from execute().
 * The host parses the payload into a serde_json::Value and wraps it in
 * Response::Action. See docs/action-responses.md for the expected
 * envelope shape.
 */
export function respondAction(json: string): i64 {
  return packResponse(RESPONSE_TAG_ACTION, json);
}

/**
 * Prefer `respondText`. Kept so examples written before the tagged ABI
 * still compile.
 * @deprecated use respondText
 */
export function respond(s: string): i64 {
  return respondText(s);
}

function packResponse(tag: u8, s: string): i64 {
  const buf = String.UTF8.encode(s);
  const len = buf.byteLength;
  const dest = ari_alloc(len as i32);
  memory.copy(dest as usize, changetype<usize>(buf), len as usize);
  // Layout must match `decode_execute_return` in ari-skill-loader/src/wasm.rs:
  //   bits 63..56 = tag, 55..32 = ptr (24-bit), 31..0 = len (32-bit)
  const tagBits: i64 = (i64(tag) & 0xFF) << 56;
  const ptrBits: i64 = (i64(dest) & 0x00FFFFFF) << 32;
  const lenBits: i64 = i64(len) & 0xFFFFFFFF;
  return tagBits | ptrBits | lenBits;
}

/**
 * Unpack a host-returned packed value into a string. Used for import returns
 * (http_fetch, storage_get) — these carry plain (ptr, len) with no tag byte,
 * so the 32-bit-ptr layout is preserved here deliberately.
 * Returns null if the packed value is 0 (sentinel for "not found").
 */
export function unpack(packed: i64): string | null {
  if (packed == 0) return null;
  const ptr = i32(packed >> 32);
  const len = i32(packed & 0xFFFFFFFF);
  return String.UTF8.decodeUnsafe(ptr as usize, len as usize);
}

// ---------------------------------------------------------------------------
// Logging
// ---------------------------------------------------------------------------

// @ts-ignore: external
@external("ari", "log")
declare function host_log(level: i32, ptr: i32, len: i32): void;

// @ts-ignore: external
@external("ari", "get_capability")
declare function host_get_capability(name_ptr: i32, name_len: i32): i32;

// @ts-ignore: external
@external("ari", "now_ms")
declare function host_now_ms(): i64;

// @ts-ignore: external
@external("ari", "rand_u64")
declare function host_rand_u64(): i64;

export const TRACE: i32 = 0;
export const DEBUG: i32 = 1;
export const INFO: i32 = 2;
export const WARN: i32 = 3;
export const ERROR: i32 = 4;

export function log(level: i32, msg: string): void {
  const buf = String.UTF8.encode(msg);
  const len = buf.byteLength;
  const ptr = ari_alloc(len as i32);
  memory.copy(ptr as usize, changetype<usize>(buf), len as usize);
  host_log(level, ptr, len as i32);
}

/** Returns true if the named capability is both declared and granted. */
export function hasCapability(name: string): bool {
  const buf = String.UTF8.encode(name);
  const len = buf.byteLength;
  const ptr = ari_alloc(len as i32);
  memory.copy(ptr as usize, changetype<usize>(buf), len as usize);
  return host_get_capability(ptr, len as i32) == 1;
}

/**
 * Current Unix time in milliseconds, as seen by the host.
 *
 * Wall-clock, not monotonic — the host calls `SystemTime::now()` on each
 * invocation, so DST changes and clock adjustments are visible. Fine for
 * timers, timestamps, and "when did the user last ask me this"; not fine
 * for performance measurement.
 */
export function nowMs(): i64 {
  return host_now_ms();
}

/**
 * 64 bits of cryptographically-random entropy from the host.
 *
 * Use for ids, tokens, and anything where predictability matters. The
 * return is typed `i64` to match the import signature — cast to `u64`
 * with `u64(randU64())` if you want unsigned semantics.
 */
export function randU64(): i64 {
  return host_rand_u64();
}
