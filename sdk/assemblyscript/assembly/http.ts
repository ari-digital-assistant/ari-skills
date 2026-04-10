// Ari Skill SDK — HTTP capability
//
// Import this module ONLY if your skill declares capabilities: [http].
// The @external declaration causes the WASM module to import ari::http_fetch,
// which the host's sneak guard will reject if the capability isn't declared.

import { ari_alloc, unpack } from "./index";

// @ts-ignore: external
@external("ari", "http_fetch")
declare function host_http_fetch(url_ptr: i32, url_len: i32): i64;

/**
 * Perform an HTTP GET. The host enforces scheme restrictions (HTTPS only
 * by default) and body size limits.
 *
 * Returns the raw response string (JSON: {"status": N, "body": "..."} or
 * {"status": 0, "body": null, "error": "..."}). Returns null on failure.
 */
export function httpFetchRaw(url: string): string | null {
  const buf = String.UTF8.encode(url);
  const len = buf.byteLength;
  const ptr = ari_alloc(len as i32);
  memory.copy(ptr as usize, changetype<usize>(buf), len as usize);
  const packed = host_http_fetch(ptr, len as i32);
  return unpack(packed);
}
