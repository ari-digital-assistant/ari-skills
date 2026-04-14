import { ari_alloc, input, respondText, log, INFO } from "ari-skill-sdk-as/assembly";

// Re-export the allocator so the host can find it.
export { ari_alloc };

export function score(ptr: i32, len: i32): f32 {
  // For most skills, leave this at 0.0 and let the manifest's keyword
  // patterns handle scoring (set custom_score: false in SKILL.md).
  return 0.95;
}

export function execute(ptr: i32, len: i32): i64 {
  const text = input(ptr, len);
  log(INFO, "skill executed");
  // Use `respondText` for plain text, or `respondAction` (same module) to
  // emit a JSON envelope the frontend interprets as a rich UI action — see
  // docs/action-responses.md. `nowMs()` and `randU64()` are also available.
  return respondText("You said: " + text);
}
