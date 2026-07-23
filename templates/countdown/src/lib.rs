#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

#[cfg(target_arch = "wasm32")]
use ari_skill_sdk as ari;
use ari_skill_sdk::presentation as p;

/// Required export. Never called while `matching.custom_score` is false —
/// the engine scores this skill from the manifest's keyword patterns.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    0.0
}

/// Demo skill that emits a presentation card with a 30-second countdown +
/// an `on_complete` alert. The countdown ticks live in the chat; when it
/// hits zero the frontend fires the loud alert. Tap "Cancel" on the card
/// to send a "stop demo" utterance through the engine — your own skill
/// would handle that and dismiss the card.
///
/// Drop your own presentation primitives in here. See
/// `docs/reference-actions.md` for the full envelope shape.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let _input = unsafe { ari::input(ptr, len) };
    let now_ms = ari::now_ms();
    let end_ts_ms = now_ms + 30_000;

    let json = p::Envelope::new()
        .speak("Demo countdown started.")
        .card(
            p::Card::new("card_demo")
                .title("Demo countdown")
                .countdown_to(end_ts_ms)
                .started_at(now_ms)
                .action(
                    p::Action::new("cancel", "Cancel")
                        .utterance("stop demo")
                        .destructive(),
                )
                .on_complete(
                    p::OnComplete::new().alert(
                        p::Alert::new("alert_demo")
                            .title("Demo countdown done")
                            .urgency(p::Urgency::High)
                            .sound(p::Sound::SystemAlarm)
                            .speech_loop("Demo")
                            .action(
                                p::Action::new("stop_alert", "Stop").primary(),
                            ),
                    ),
                ),
        )
        .to_json();
    ari::respond_action(&json)
}

// The exports above are wasm32-only, so a host-target `cargo check` would
// otherwise see the SDK as an unused dependency. This keeps it referenced.
#[cfg(not(target_arch = "wasm32"))]
fn _ensure_deps_referenced() {
    let _ = p::Envelope::new().to_json();
}
