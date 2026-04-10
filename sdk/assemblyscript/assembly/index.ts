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

/**
 * Pack a response string for return from execute().
 * Encodes to UTF-8 in bump-allocated memory and returns the packed
 * (ptr << 32) | len value the host expects.
 */
export function respond(s: string): i64 {
  const buf = String.UTF8.encode(s);
  const len = buf.byteLength;
  const dest = ari_alloc(len as i32);
  memory.copy(dest as usize, changetype<usize>(buf), len as usize);
  return (i64(dest) << 32) | i64(len);
}

/**
 * Unpack a host-returned (ptr << 32) | len into a string.
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
