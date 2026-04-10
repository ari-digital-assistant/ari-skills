import { ari_alloc, input, respond, log, INFO } from "ari-skill-sdk-as/assembly";

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
  return respond("You said: " + text);
}
