// Ari Skill SDK — Storage capability
//
// Import this module ONLY if your skill declares capabilities: [storage_kv].
// The @external declarations cause the WASM module to import ari::storage_get
// and ari::storage_set, which the host's sneak guard will reject if the
// capability isn't declared.

import { ari_alloc, unpack } from "./index";

// @ts-ignore: external
@external("ari", "storage_get")
declare function host_storage_get(key_ptr: i32, key_len: i32): i64;

// @ts-ignore: external
@external("ari", "storage_set")
declare function host_storage_set(
  key_ptr: i32,
  key_len: i32,
  val_ptr: i32,
  val_len: i32
): i32;

/** Read a value from this skill's key-value store. Returns null if absent. */
export function storageGet(key: string): string | null {
  const buf = String.UTF8.encode(key);
  const len = buf.byteLength;
  const ptr = ari_alloc(len as i32);
  memory.copy(ptr as usize, changetype<usize>(buf), len as usize);
  const packed = host_storage_get(ptr, len as i32);
  return unpack(packed);
}

/**
 * Write a value to this skill's key-value store.
 * Returns true on success, false on failure (key/value too large,
 * total storage cap exceeded, I/O error).
 */
export function storageSet(key: string, value: string): bool {
  const kbuf = String.UTF8.encode(key);
  const klen = kbuf.byteLength;
  const kptr = ari_alloc(klen as i32);
  memory.copy(kptr as usize, changetype<usize>(kbuf), klen as usize);

  const vbuf = String.UTF8.encode(value);
  const vlen = vbuf.byteLength;
  const vptr = ari_alloc(vlen as i32);
  memory.copy(vptr as usize, changetype<usize>(vbuf), vlen as usize);

  return host_storage_set(kptr, klen as i32, vptr, vlen as i32) == 0;
}
