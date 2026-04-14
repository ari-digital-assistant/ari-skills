use ari_skill_sdk as ari;

#[no_mangle]
pub extern "C" fn score(ptr: i32, len: i32) -> f32 {
    // For most skills, leave this at 0.0 and let the manifest's keyword
    // patterns handle scoring (set custom_score: false in SKILL.md).
    // Only implement custom scoring if your skill needs input-dependent
    // relevance beyond what keywords/regex can express.
    let _input = unsafe { ari::input(ptr, len) };
    0.95
}

#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    ari::log(ari::LogLevel::Info, "skill executed");
    // Use `respond_text` for plain text, or `respond_action` to return a
    // JSON envelope the frontend can interpret as a rich UI action (timer
    // card, app launch, search, etc). See docs/action-responses.md.
    ari::respond_text(&format!("You said: {input}"))
}
