//! Tally — a starter WASM skill.
//!
//! Demonstrates the four things a declarative skill can't do: persistent
//! state, reading user settings, branching on the utterance, and building a
//! rich response envelope.

use ari_skill_sdk as ari;
use ari_skill_sdk::presentation as p;

/// Key in this skill's private `storage_kv` namespace. Storage is scoped to
/// the skill id — no other skill can see or clobber it.
const KEY_COUNT: &str = "count";

/// Required export. The loader only calls this when the manifest sets
/// `matching.custom_score: true`; with the default (`false`) the engine
/// scores the skill from the manifest's keyword patterns and never enters
/// the WASM module. Returning 0.0 keeps the required export present without
/// claiming any relevance of its own.
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    0.0
}

#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    // The host hands `execute` the NORMALISED utterance: lowercased,
    // contractions expanded, punctuation stripped, English number words
    // turned into digits. Match against that, not against raw speech.
    let input = unsafe { ari::input(ptr, len) };

    let (count, spoken) = if input.contains("reset") {
        store(0);
        (0, translate("tally.reset", 0))
    } else if input.contains("add") || input.contains("another") {
        let next = load() + 1;
        store(next);
        (next, translate("tally.added", next))
    } else {
        let now = load();
        (now, translate("tally.current", now))
    };

    ari::respond_action(&envelope(&spoken, count))
}

// --- state -----------------------------------------------------------------

fn load() -> u32 {
    ari::storage_get(KEY_COUNT)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn store(value: u32) {
    if !ari::storage_set(KEY_COUNT, &value.to_string()) {
        ari::log(ari::LogLevel::Warn, "could not persist the tally");
    }
}

// --- presentation ----------------------------------------------------------

fn envelope(spoken: &str, count: u32) -> String {
    // Settings the user filled in on the skill's settings screen. Both are
    // optional here, so both have a sensible fallback.
    let label = ari::setting_get("label").unwrap_or("things");
    let goal: u32 = ari::setting_get("goal")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let mut stat = p::Stat::new(count.to_string()).caption(label);
    if goal > 0 {
        stat = stat.pill(p::IconText::new(format!("{count} of {goal}")));
    }

    let accent = if goal > 0 && count >= goal {
        p::Accent::Success
    } else {
        p::Accent::Default
    };

    p::Envelope::new()
        .speak(spoken)
        // A stable card id means re-emitting replaces the card in place
        // instead of stacking a new one under every utterance.
        .card(
            p::Card::new("tally")
                .title(translate_plain("tally.title"))
                .accent(accent)
                .stat(stat),
        )
        .to_json()
}

// --- i18n ------------------------------------------------------------------

/// Look up `key` in `strings/<locale>.json`, substituting `{count}`.
/// Falls back to English, then to the bare key, so a typo stays visible.
fn translate(key: &str, count: u32) -> String {
    let count = count.to_string();
    ari::t(key, &[("count", count.as_str())])
        .unwrap_or(key)
        .to_string()
}

fn translate_plain(key: &str) -> String {
    ari::t(key, &[]).unwrap_or(key).to_string()
}
