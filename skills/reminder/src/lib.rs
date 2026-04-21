//! Ari reminder skill.
//!
//! Parses utterances like "remind me to walk the dog at 5pm" and "add
//! milk to my shopping list" into a structured `create_reminder`
//! action envelope. The skill stays timezone-naive — the `when` field
//! is a descriptor (offset-from-now or local clock components) and the
//! Android frontend resolves to an absolute timestamp using the
//! device's local zone.
//!
//! See SKILL.md for the supported utterance shapes and the action
//! envelope schema.

#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;
use alloc::string::String;

#[cfg(target_arch = "wasm32")]
use ari_skill_sdk as ari;

mod parse;

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    // custom_score: false in the manifest — engine uses the regex
    // pattern scorer and never calls into here. Returned value is a
    // shrug for completeness.
    0.9
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    let envelope = dispatch(input);
    ari::respond_action(&envelope)
}

/// Plain-Rust entry point for unit tests and host-side reuse. Pure
/// function of the input — no host imports touched, no clock read.
pub fn dispatch(input: &str) -> String {
    let parsed = parse::parse(input);
    #[cfg(target_arch = "wasm32")]
    {
        // Surface the parse outcome on the host's log sink so skill
        // authors can eyeball confidence decisions in `adb logcat -s
        // AriSkill`. Low-volume, one line per utterance.
        ari::log(
            ari::LogLevel::Info,
            &alloc::format!(
                "parse confidence={} unparsed={:?} title={:?}",
                parsed.confidence.as_envelope_str(),
                parsed.unparsed.as_deref().unwrap_or(""),
                parsed.title,
            ),
        );
    }
    build_envelope(&parsed)
}

fn build_envelope(parsed: &parse::Parsed) -> String {
    // Top-level slot named `create_reminder`, matching the existing
    // envelope convention (`launch_app`, `search`, `clipboard`, ...).
    // Frontend's ActionHandler dispatches on the presence of this
    // slot the same way it dispatches the others. Hand-rolled JSON
    // here to keep field order stable across builds — easier to grep
    // for in logs than serde's HashMap ordering.
    let mut out = String::from("{\"v\":1,\"create_reminder\":{\"title\":");
    push_json_string(&mut out, &parsed.title);

    out.push_str(",\"when\":");
    match &parsed.when {
        parse::When::None => out.push_str("null"),
        parse::When::InSeconds(s) => {
            out.push_str(&format!("{{\"in_seconds\":{}}}", s));
        }
        parse::When::LocalClock {
            hour,
            minute,
            day_offset,
        } => {
            out.push_str(&format!(
                "{{\"local_time\":\"{:02}:{:02}\",\"day_offset\":{}}}",
                hour, minute, day_offset,
            ));
        }
        parse::When::LocalClockOnWeekday {
            hour,
            minute,
            weekday,
        } => {
            // Emit weekday as the English name rather than the ISO
            // index. It's a six-extra-chars price for JSON that a human
            // debugging logcat can read without an index cheat-sheet.
            out.push_str(&format!(
                "{{\"local_time\":\"{:02}:{:02}\",\"weekday\":\"{}\"}}",
                hour,
                minute,
                weekday_name(*weekday),
            ));
        }
        parse::When::LocalClockOnDate {
            hour,
            minute,
            month,
            day,
        } => {
            out.push_str(&format!(
                "{{\"local_time\":\"{:02}:{:02}\",\"month\":{},\"day\":{}}}",
                hour, minute, month, day,
            ));
        }
        parse::When::DateOnly { day_offset } => {
            out.push_str(&format!("{{\"day_offset\":{}}}", day_offset));
        }
        parse::When::DateOnlyWeekday { weekday } => {
            out.push_str(&format!("{{\"weekday\":\"{}\"}}", weekday_name(*weekday)));
        }
        parse::When::DateOnlyDate { month, day } => {
            out.push_str(&format!("{{\"month\":{},\"day\":{}}}", month, day));
        }
    }

    out.push_str(",\"list_hint\":");
    match &parsed.list_hint {
        Some(s) => push_json_string(&mut out, s),
        None => out.push_str("null"),
    }

    out.push_str(",\"speak_template\":");
    push_json_string(&mut out, &parsed.speak_template);

    // Close the `create_reminder` block, then drop `confidence` and
    // `unparsed` at envelope top-level — Layer A of the parse-confidence
    // signal (see wtf.md). Always emitted, even for High confidence,
    // so the Android side can treat a missing field as "old skill
    // build, assume high" without ambiguity from newer skills.
    out.push_str("}");
    out.push_str(",\"confidence\":\"");
    out.push_str(parsed.confidence.as_envelope_str());
    out.push_str("\"");
    if let Some(u) = &parsed.unparsed {
        out.push_str(",\"unparsed\":");
        push_json_string(&mut out, u);
    }
    out.push_str("}");
    out
}

/// ISO index (0=Monday..6=Sunday) back to its lowercase English name.
/// Used by the envelope emitter; the Android side parses the name back
/// into its platform-specific weekday enum.
fn weekday_name(idx: u8) -> &'static str {
    match idx {
        0 => "monday",
        1 => "tuesday",
        2 => "wednesday",
        3 => "thursday",
        4 => "friday",
        5 => "saturday",
        6 => "sunday",
        _ => "monday",
    }
}

/// Minimal JSON string escaper covering the characters that actually
/// appear in user-spoken text. We never see control bytes outside the
/// usual whitespace, so backslash + quote is enough.
fn push_json_string(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field<'a>(envelope: &'a str, key: &str) -> &'a str {
        // Tiny extractor — finds `"<key>":` and returns the literal
        // value up to the next top-level `,` or `}`. Good enough for
        // these flat envelopes; if the shape ever nests we'll switch
        // to serde_json in tests.
        let needle = format!("\"{}\":", key);
        let start = envelope.find(&needle).expect(&format!("key {key} missing")) + needle.len();
        let mut depth = 0usize;
        let mut end = start;
        for (i, c) in envelope[start..].char_indices() {
            match c {
                '{' | '[' => depth += 1,
                '}' | ']' if depth == 0 => {
                    end = start + i;
                    break;
                }
                '}' | ']' => depth -= 1,
                ',' if depth == 0 => {
                    end = start + i;
                    break;
                }
                _ => {}
            }
        }
        envelope[start..end].trim()
    }

    #[test]
    fn untimed_reminder_extracts_title_and_emits_null_when() {
        let json = dispatch("remind me to buy milk");
        assert_eq!(field(&json, "title"), "\"buy milk\"");
        assert_eq!(field(&json, "when"), "null");
        assert_eq!(field(&json, "list_hint"), "null");
    }

    #[test]
    fn relative_reminder_emits_in_seconds() {
        let json = dispatch("remind me in 30 minutes to check the oven");
        assert_eq!(field(&json, "title"), "\"check the oven\"");
        assert_eq!(field(&json, "when"), "{\"in_seconds\":1800}");
    }

    #[test]
    fn relative_hours_reminder_emits_in_seconds() {
        let json = dispatch("remind me in 2 hours to check the oven");
        assert_eq!(field(&json, "when"), "{\"in_seconds\":7200}");
    }

    #[test]
    fn at_time_reminder_emits_local_clock_today() {
        let json = dispatch("remind me to walk the dog at 5pm");
        assert_eq!(field(&json, "title"), "\"walk the dog\"");
        assert_eq!(field(&json, "when"), "{\"local_time\":\"17:00\",\"day_offset\":0}");
    }

    #[test]
    fn at_time_with_minutes_emits_local_clock() {
        let json = dispatch("remind me to walk the dog at 5:30pm");
        assert_eq!(field(&json, "when"), "{\"local_time\":\"17:30\",\"day_offset\":0}");
    }

    #[test]
    fn am_time_emits_morning_local_clock() {
        let json = dispatch("remind me to take pills at 9am");
        assert_eq!(field(&json, "when"), "{\"local_time\":\"09:00\",\"day_offset\":0}");
    }

    #[test]
    fn tomorrow_at_time_emits_day_offset_one() {
        let json = dispatch("remind me at 9am tomorrow to call the dentist");
        assert_eq!(field(&json, "when"), "{\"local_time\":\"09:00\",\"day_offset\":1}");
        assert_eq!(field(&json, "title"), "\"call the dentist\"");
    }

    #[test]
    fn tomorrow_with_no_time_emits_date_only() {
        let json = dispatch("remind me about laundry tomorrow");
        assert_eq!(field(&json, "when"), "{\"day_offset\":1}");
        assert_eq!(field(&json, "title"), "\"laundry\"");
    }

    #[test]
    fn noon_resolves_to_twelve_local_clock() {
        let json = dispatch("remind me to eat at noon");
        assert_eq!(field(&json, "when"), "{\"local_time\":\"12:00\",\"day_offset\":0}");
    }

    #[test]
    fn midnight_resolves_to_zero_local_clock() {
        let json = dispatch("remind me at midnight to set my alarm");
        assert_eq!(field(&json, "when"), "{\"local_time\":\"00:00\",\"day_offset\":0}");
    }

    #[test]
    fn add_to_named_list_extracts_list_hint_and_strips_phrase() {
        let json = dispatch("add milk to my shopping list");
        assert_eq!(field(&json, "title"), "\"milk\"");
        assert_eq!(field(&json, "list_hint"), "\"shopping\"");
        assert_eq!(field(&json, "when"), "null");
    }

    #[test]
    fn put_on_named_list_works_too() {
        let json = dispatch("put eggs on the shopping list");
        assert_eq!(field(&json, "title"), "\"eggs\"");
        assert_eq!(field(&json, "list_hint"), "\"shopping\"");
    }

    #[test]
    fn multi_word_list_name_is_captured() {
        let json = dispatch("add deadline review to my work projects list");
        assert_eq!(field(&json, "title"), "\"deadline review\"");
        assert_eq!(field(&json, "list_hint"), "\"work projects\"");
    }

    #[test]
    fn speak_template_uses_calendar_placeholder_for_calendar_intent() {
        // For named-list and default-list intents we use {list_name};
        // the frontend swaps for {calendar_name} when the destination
        // resolves to Calendar — same speak_template either way works
        // because the frontend only substitutes the placeholder it
        // recognises after resolution. Verify the raw template here.
        let json = dispatch("add milk to my shopping list");
        assert_eq!(
            field(&json, "speak_template"),
            "\"Added {title} to your {list_name} list\""
        );
    }

    #[test]
    fn empty_input_yields_blank_title_with_null_when() {
        let json = dispatch("");
        assert_eq!(field(&json, "title"), "\"\"");
        assert_eq!(field(&json, "when"), "null");
    }

    // ── Envelope-level confidence + unparsed emission ────────────────
    // Layer A of the parse-confidence work: the skill reports how
    // sure it is about its own output so the frontend can warn the
    // user when the parse is dodgy. Old frontends ignore the unknown
    // fields; new ones branch on them.

    #[test]
    fn high_confidence_emits_no_unparsed_field() {
        let json = dispatch("remind me to walk the dog at 5pm");
        assert_eq!(field(&json, "confidence"), "\"high\"");
        assert!(!json.contains("\"unparsed\""), "envelope = {json}");
    }

    #[test]
    fn partial_confidence_emits_unparsed_field() {
        // "next" is stranded after the weekday scanner consumes
        // "tuesday". Partial because a concrete weekday DID land.
        let json = dispatch("remind me next tuesday at 9am to see the dentist");
        assert_eq!(field(&json, "confidence"), "\"partial\"");
        assert_eq!(field(&json, "unparsed"), "\"next\"");
    }

    #[test]
    fn low_confidence_when_nothing_matched() {
        // "tonight" is in the reserved-residue list but isn't yet
        // recognised as an anchor, and no clock phrase is present.
        // When=None → fallback → Low.
        let json = dispatch("remind me to do a thing tonight");
        assert_eq!(field(&json, "confidence"), "\"low\"");
        assert_eq!(field(&json, "unparsed"), "\"tonight\"");
    }
}
