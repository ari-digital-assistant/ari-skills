import { ari_alloc, input, respondText, log, INFO } from "ari-skill-sdk-as/assembly";

// Re-export the allocator so the host can write your input into memory.
// Forget this and the skill compiles but never receives anything.
export { ari_alloc };

/**
 * Required export. Never called while `matching.custom_score` is false —
 * the engine scores this skill from the manifest's keyword patterns.
 */
export function score(ptr: i32, len: i32): f32 {
  return 0.0;
}

export function execute(ptr: i32, len: i32): i64 {
  // The host hands you the NORMALISED utterance: lowercased, contractions
  // expanded, punctuation stripped, English number words turned into digits.
  const text = input(ptr, len);
  log(INFO, "echo-as executed");

  // `respondAction(json)` is the other option — it returns an action
  // envelope for the frontend to render. The AssemblyScript SDK has no
  // typed builder for it, so you would hand-write the JSON.
  // See ../../docs/reference-actions.md.
  return respondText("You said: " + text);
}
