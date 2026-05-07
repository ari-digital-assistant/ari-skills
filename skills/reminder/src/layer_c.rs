//! Layer C v2 — skill-authored, async, structured assistant round-trip.
//!
//! Runs when the first-pass parser's confidence is `Partial` or `Low`.
//! Instead of committing a reminder optimistically and hoping the
//! user's Cancel reflex catches the mistake, the skill punts to the
//! active cloud assistant with a tightly-framed prompt that asks for a
//! strict JSON response. The engine carries the round-trip
//! asynchronously and re-enters the skill with the response via
//! [`Skill::execute_continuation`]; the skill's continuation handler
//! owns all post-assistant logic (commit, ask for clarification, or
//! fall back to the old warn-and-commit card when the assistant was
//! unreachable or its answer was unhelpful).
//!
//! Nothing in this module calls host imports — it's pure string
//! munging and serde. The caller (`lib.rs`) does the actual
//! `ari::tasks_insert` / `ari::calendar_insert` work.
//!
//! Wire shapes (stable contract with the engine):
//!
//! - **Phase-1 envelope** (skill emits, engine intercepts):
//!   ```json
//!   { "v": 1,
//!     "speak": "Let me check that...",
//!     "consult_assistant": {
//!       "prompt": "<full prompt text>",
//!       "continuation_context": "<utterance>"
//!     }
//!   }
//!   ```
//!
//! - **Continuation input** (engine passes to the skill, bypassing
//!   keyword routing and `normalize_input`):
//!   ```json
//!   { "_ari_continuation": { "context": "<ctx>", "response": "<text>" } }
//!   ```
//!
//! - **Assistant response shape** (what we ask the assistant to
//!   return):
//!   ```json
//!   { "title": "...",
//!     "datetime": "YYYY-MM-DDTHH:MM:SS" | null,
//!     "confidence": "high" | "partial" | "low" }
//!   ```
//!   Empty response string means the assistant was unreachable — skill
//!   falls back to the optimistic warn-and-commit path.

use alloc::format;
use alloc::string::{String, ToString};
use serde::Deserialize;

use crate::parse;

/// Structured assistant reply. Fields match what the prompt asks for;
/// a malformed response returns `None` from
/// [`parse_assistant_response`] rather than a half-populated record.
///
/// `clarification` + `follow_up` are only populated (and only acted on)
/// when `confidence == "partial"` — that's the flow where we want to
/// go back to the user and ask a targeted question before committing.
/// `high` skips straight to commit; `low` falls through to the
/// warn-and-commit path since the AI itself isn't sure enough to
/// compose a useful clarification.
#[derive(Debug, Deserialize)]
pub struct AssistantResponse {
    pub title: String,
    pub datetime: Option<String>,
    pub confidence: String,
    #[serde(default)]
    pub clarification: Option<String>,
    #[serde(default)]
    pub follow_up: Option<String>,
}

impl AssistantResponse {
    /// True when the response carries a usable yes/no clarification —
    /// partial confidence, non-empty clarification text, and the
    /// follow-up shape the skill knows how to render (`yes_no`).
    /// Open-ended follow-ups are left for a future iteration; if the
    /// AI emits `open_ended`, we fall through to warn-and-commit.
    pub fn is_actionable_yes_no_clarification(&self) -> bool {
        self.confidence.eq_ignore_ascii_case("partial")
            && self
                .clarification
                .as_deref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false)
            && self
                .follow_up
                .as_deref()
                .map(|s| s.eq_ignore_ascii_case("yes_no"))
                .unwrap_or(false)
    }
}

/// Parsed continuation input — what the engine passes the skill after
/// the assistant round-trip. `context` is the string the skill put in
/// `consult_assistant.continuation_context` (for reminders: the
/// original utterance). `response` is the assistant's raw reply text,
/// or empty on failure.
#[derive(Debug)]
pub struct Continuation {
    pub context: String,
    pub response: String,
}

/// Decoded `ariconfirmreminder` utterance — the magic-prefix action
/// the Yes button on a clarification card emits. Carries the AI's
/// pre-staged reminder details directly so the skill can commit on
/// receipt without another assistant round-trip or any stored state.
#[derive(Debug, PartialEq)]
pub struct Confirm {
    /// Tasks / calendar / both.
    pub destination: String,
    /// UTC epoch ms of the reminder time, or 0 for untimed.
    pub epoch_ms: i64,
    /// The reminder title.
    pub title: String,
}

/// Format: `ariconfirmreminder <destination> <epoch_ms_or_0> <title_hex>`
/// — three alphanumeric tokens after the prefix, all of which survive
/// `normalize_input`. Hex-encoding the title (UTF-8 bytes) lets titles
/// containing punctuation, quotes, accented characters, etc. round-trip
/// through the engine's normaliser unharmed.
pub fn parse_confirm(input: &str) -> Option<Confirm> {
    let mut tokens = input.trim().split_whitespace();
    if tokens.next()? != "ariconfirmreminder" {
        return None;
    }
    let destination = tokens.next()?;
    if !matches!(destination, "tasks" | "calendar" | "both") {
        return None;
    }
    let epoch_ms: i64 = tokens.next()?.parse().ok()?;
    let title_hex = tokens.next()?;
    let title = hex_decode_utf8(title_hex)?;
    Some(Confirm {
        destination: destination.to_string(),
        epoch_ms,
        title,
    })
}

/// Encode a `Confirm` back into the wire format the engine will route
/// to the skill on the next user turn. Inverse of [`parse_confirm`];
/// skills call this when composing the Yes button's action utterance.
pub fn encode_confirm(destination: &str, epoch_ms: i64, title: &str) -> String {
    let hex = hex_encode(title.as_bytes());
    format!("ariconfirmreminder {destination} {epoch_ms} {hex}")
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(nibble_to_char(b >> 4));
        out.push(nibble_to_char(b & 0x0f));
    }
    out
}

fn hex_decode_utf8(hex: &str) -> Option<String> {
    if hex.len() % 2 != 0 {
        return None;
    }
    let bytes: Option<alloc::vec::Vec<u8>> = hex
        .as_bytes()
        .chunks(2)
        .map(|pair| {
            let hi = char_to_nibble(pair[0])?;
            let lo = char_to_nibble(pair[1])?;
            Some((hi << 4) | lo)
        })
        .collect();
    let bytes = bytes?;
    String::from_utf8(bytes).ok()
}

fn nibble_to_char(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'a' + n - 10) as char,
        _ => unreachable!(),
    }
}

fn char_to_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Detects the `{"_ari_continuation":{"context":"...","response":"..."}}`
/// shape the engine uses to resume skill execution. Returns `None` for
/// any input that isn't a continuation (the caller treats that as a
/// normal user utterance).
///
/// Uses serde_json rather than byte-scanning: the response string can
/// contain quotes, braces, JSON fragments etc., and we can't reliably
/// split the two string values with a manual parser.
pub fn parse_continuation_input(input: &str) -> Option<Continuation> {
    // Cheap rejection before paying the full JSON parse. The engine's
    // default [`Skill::execute_continuation`] always produces this
    // exact key, so the prefix check is reliable.
    let trimmed = input.trim_start();
    if !trimmed.starts_with("{\"_ari_continuation\"") {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    let inner = value.get("_ari_continuation")?.as_object()?;
    let context = inner.get("context")?.as_str()?.to_string();
    let response = inner.get("response")?.as_str()?.to_string();
    Some(Continuation { context, response })
}

/// Extract the assistant's structured reply. Tolerant of a surrounding
/// code fence (```json ... ```) and leading/trailing chatter — strips
/// to the outer `{...}` before parsing. Returns `None` when the input
/// is empty (engine signal for "assistant unavailable") or unparseable.
pub fn parse_assistant_response(text: &str) -> Option<AssistantResponse> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let body = strip_to_outer_object(trimmed)?;
    serde_json::from_str::<AssistantResponse>(body).ok()
}

/// Find the first `{` and its matching closing `}`, accounting for
/// nested braces. Skips common preamble ("Here is the JSON:", code
/// fences) that small cloud models sometimes prepend.
fn strip_to_outer_object(s: &str) -> Option<&str> {
    let start = s.find('{')?;
    let bytes = s.as_bytes();
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escape = false;
    for i in start..bytes.len() {
        let b = bytes[i];
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[start..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Build the prompt the engine sends to the assistant. Substitutes the
/// utterance, the parser's first-pass extraction, the flagged residue,
/// and today's local date so the model can resolve relative references
/// like "the 7th" or "next Friday". `today` is a pre-formatted
/// human-readable date plus ISO form, e.g.
/// `"Monday, 27 April 2026 (2026-04-27)"`. Deliberately verbose —
/// small cloud and on-device models reward unambiguous framing over
/// terseness.
pub fn compose_prompt(
    utterance: &str,
    parsed: &parse::Parsed,
    today: &str,
    locale: &str,
) -> String {
    let when_desc = when_summary(&parsed.when);
    let unparsed = parsed
        .unparsed
        .as_deref()
        .unwrap_or("(parser didn't flag a specific fragment)");

    // Locale tail: appended for every non-English locale we ship.
    // Tells the model to render the human-language fields (`title`,
    // `clarification`) in the user's language while keeping the
    // structural fields (`datetime`, `confidence`, `follow_up`) as
    // machine-readable enums. Cloud LLMs honour this reliably; the
    // on-device LLM tier still defaults to English-trained instruction
    // following so we phrase the directive in English.
    let locale_tail = match locale {
        "it" => "\n\nThe user is speaking Italian. Output `title` and `clarification` in Italian. Keep `datetime`, `confidence`, and `follow_up` as the exact enum strings specified above.",
        "es" => "\n\nThe user is speaking Spanish. Output `title` and `clarification` in Spanish. Keep `datetime`, `confidence`, and `follow_up` as the exact enum strings specified above.",
        "fr" => "\n\nThe user is speaking French. Output `title` and `clarification` in French. Keep `datetime`, `confidence`, and `follow_up` as the exact enum strings specified above.",
        "de" => "\n\nThe user is speaking German. Output `title` and `clarification` in German. Keep `datetime`, `confidence`, and `follow_up` as the exact enum strings specified above.",
        _ => "",
    };

    format!(
        "You are helping interpret an ambiguous voice-assistant request. \
         The user said: \"{utterance}\"\n\
         \n\
         A first-pass parser on-device extracted this much: title=\"{title}\", when={when}, \
         and flagged \"{unparsed}\" as a fragment it couldn't resolve.\n\
         \n\
         Today: {today}\n\
         \n\
         Return a STRICT JSON object and nothing else — no preamble, no code fences, no trailing text:\n\
         {{\"title\": string, \"datetime\": string|null, \"confidence\": string, \
         \"clarification\": string, \"follow_up\": string}}\n\
         \n\
         Field rules:\n\
         - title: the reminder text, cleaned of date/time/list phrasing\n\
         - datetime: ISO-8601 local time \"YYYY-MM-DDTHH:MM:SS\" when the user mentioned any \
           time or date, including when you're only partly sure (fill in your best guess; \
           the user will confirm or reject via the clarification). \
           Only return null when the user genuinely wanted an untimed to-do (no time words at all). \
           Saying \"at 3pm\" always needs a datetime — never null.\n\
         - confidence: \"high\" if you're sure what they meant, \"partial\" if unsure about one \
           specific detail (one word, one date, one reading), \"low\" if the request is too vague \
           to act on at all\n\
         - clarification: when confidence is \"partial\", a short one-sentence question asking \
           the user to confirm the specific detail you were unsure about. PHRASING RULES — \
           READ CAREFULLY:\n\
           1. Reference the *concrete resolved value* you put in `datetime` or `title`, not the \
              user's original phrasing. If the user said \"next Friday\" and you resolved it to \
              `2026-05-01`, your clarification must say something like \"the 1st of May\", NOT \
              \"next Friday\". The whole point is to surface what you decided so the user can \
              confirm or reject the resolution. Echoing their words back tells them nothing.\n\
           2. Phrase it for a yes/no answer. \"Did you mean the 1st of May at 3pm?\" — good. \
              \"When did you mean?\" — wrong format.\n\
           3. One short sentence, no preamble.\n\
           Bad clarifications (avoid): \"Did you mean next Friday at 3pm?\" (paraphrasing the \
           user's input). \"Are you sure?\" (vague). \"Could you clarify?\" (open-ended).\n\
           Good clarifications: \"Did you mean the 1st of May at 3pm?\" / \"Did you mean the \
           27th of this month?\" / \"Is that Sarah from Marketing?\".\n\
           Empty string when confidence is \"high\" or \"low\".\n\
         - follow_up: \"yes_no\" when the clarification expects a yes/no answer, \"open_ended\" \
           when it needs more than that (currently treat open_ended as a future extension — \
           prefer yes_no whenever you can sensibly phrase it that way). Empty string when \
           confidence is \"high\" or \"low\".\n\
         \n\
         Output the JSON object only.{locale_tail}",
        utterance = escape_for_prompt(utterance),
        title = escape_for_prompt(&parsed.title),
        when = when_desc,
        unparsed = escape_for_prompt(unparsed),
        today = today,
        locale_tail = locale_tail,
    )
}

/// Quick-and-nasty escape so embedded double quotes don't break the
/// surrounding prompt framing. Not JSON escape — the prompt itself is
/// plain text, we just need the inline quoted strings to stay quoted.
fn escape_for_prompt(s: &str) -> String {
    s.replace('"', "'")
}

/// Describes the parser's first-pass `When` extraction in plain English
/// for the assistant prompt — gives it a starting point to refine or
/// reject.
fn when_summary(when: &parse::When) -> &'static str {
    match when {
        parse::When::None => "unknown (no time extracted)",
        parse::When::InSeconds(_) => "a relative duration (\"in N minutes\")",
        parse::When::LocalClock { .. } => "an absolute clock time, day offset from today",
        parse::When::LocalClockOnWeekday { .. } => "an absolute clock time on a named weekday",
        parse::When::LocalClockOnDate { .. } => "an absolute clock time on a calendar date",
        parse::When::DateOnly { .. } => "a date with no time-of-day",
        parse::When::DateOnlyWeekday { .. } => "a named weekday with no time-of-day",
        parse::When::DateOnlyDate { .. } => "a calendar date with no time-of-day",
    }
}

/// Parsed form of the assistant's ISO-8601 datetime string. Local time,
/// no zone — matches how the rest of the skill treats time. Fields are
/// validated to the same ranges as [`civil_to_epoch_ms`] expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedDatetime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

/// Hand-parser for `YYYY-MM-DDTHH:MM[:SS][Z|±HH:MM]`. Returns `None`
/// for anything that doesn't at least have the year/month/day/hour/min
/// portion. Any trailing zone suffix is ignored — we treat all AI
/// datetimes as local time, consistent with the parser's own `When`
/// semantics.
///
/// Small cloud models occasionally return `YYYY-MM-DD HH:MM:SS` (space
/// separator) or drop the seconds. Tolerant of both.
pub fn parse_iso_datetime(s: &str) -> Option<ParsedDatetime> {
    let s = s.trim();
    let bytes = s.as_bytes();
    // Minimum shape: YYYY-MM-DDTHH:MM  → 16 bytes.
    if bytes.len() < 16 {
        return None;
    }
    let year: i32 = core::str::from_utf8(&bytes[0..4]).ok()?.parse().ok()?;
    if bytes[4] != b'-' {
        return None;
    }
    let month: u8 = core::str::from_utf8(&bytes[5..7]).ok()?.parse().ok()?;
    if bytes[7] != b'-' {
        return None;
    }
    let day: u8 = core::str::from_utf8(&bytes[8..10]).ok()?.parse().ok()?;
    if bytes[10] != b'T' && bytes[10] != b' ' {
        return None;
    }
    let hour: u8 = core::str::from_utf8(&bytes[11..13]).ok()?.parse().ok()?;
    if bytes[13] != b':' {
        return None;
    }
    let minute: u8 = core::str::from_utf8(&bytes[14..16]).ok()?.parse().ok()?;

    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
    {
        return None;
    }

    Some(ParsedDatetime {
        year,
        month,
        day,
        hour,
        minute,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuation_detected() {
        let input = r#"{"_ari_continuation":{"context":"remind me stuff","response":"{\"title\":\"x\"}"}}"#;
        let c = parse_continuation_input(input).unwrap();
        assert_eq!(c.context, "remind me stuff");
        assert_eq!(c.response, r#"{"title":"x"}"#);
    }

    #[test]
    fn continuation_rejects_normal_utterance() {
        assert!(parse_continuation_input("remind me to walk the dog").is_none());
        assert!(parse_continuation_input("aricancelreminder tasks 1").is_none());
        assert!(parse_continuation_input("").is_none());
    }

    #[test]
    fn continuation_tolerates_leading_whitespace() {
        let input = r#"   {"_ari_continuation":{"context":"c","response":"r"}}"#;
        let c = parse_continuation_input(input).unwrap();
        assert_eq!(c.context, "c");
        assert_eq!(c.response, "r");
    }

    #[test]
    fn assistant_response_parses_clean_json() {
        let text = r#"{"title":"call sarah","datetime":"2026-04-27T10:00:00","confidence":"high"}"#;
        let r = parse_assistant_response(text).unwrap();
        assert_eq!(r.title, "call sarah");
        assert_eq!(r.datetime.as_deref(), Some("2026-04-27T10:00:00"));
        assert_eq!(r.confidence, "high");
        // The three-field shape (pre-clarification, pre-follow_up) must
        // still parse — both extra fields are optional with serde
        // defaults so older AI responses / older prompts keep working.
        assert!(r.clarification.is_none());
        assert!(r.follow_up.is_none());
    }

    #[test]
    fn assistant_response_parses_clarification_fields() {
        let text = r#"{
            "title":"call mum",
            "datetime":"2026-05-01T15:00:00",
            "confidence":"partial",
            "clarification":"Did you mean this Friday (1st May) or the following Friday (8th May)?",
            "follow_up":"yes_no"
        }"#;
        let r = parse_assistant_response(text).unwrap();
        assert_eq!(r.confidence, "partial");
        assert!(r
            .clarification
            .as_ref()
            .unwrap()
            .contains("Did you mean this Friday"));
        assert_eq!(r.follow_up.as_deref(), Some("yes_no"));
        assert!(r.is_actionable_yes_no_clarification());
    }

    #[test]
    fn actionable_yes_no_requires_partial_confidence() {
        // High confidence → we commit directly, no clarification card.
        let r = AssistantResponse {
            title: "x".into(),
            datetime: None,
            confidence: "high".into(),
            clarification: Some("would you like X?".into()),
            follow_up: Some("yes_no".into()),
        };
        assert!(!r.is_actionable_yes_no_clarification());
    }

    #[test]
    fn actionable_yes_no_requires_non_empty_clarification() {
        let r = AssistantResponse {
            title: "x".into(),
            datetime: None,
            confidence: "partial".into(),
            clarification: Some("   ".into()),
            follow_up: Some("yes_no".into()),
        };
        assert!(!r.is_actionable_yes_no_clarification());
    }

    #[test]
    fn actionable_yes_no_rejects_open_ended() {
        // open_ended follow-ups would need a re-prompt flow we don't
        // build yet — fall through to warn-and-commit instead.
        let r = AssistantResponse {
            title: "x".into(),
            datetime: None,
            confidence: "partial".into(),
            clarification: Some("Which meeting?".into()),
            follow_up: Some("open_ended".into()),
        };
        assert!(!r.is_actionable_yes_no_clarification());
    }

    #[test]
    fn assistant_response_parses_null_datetime() {
        let text = r#"{"title":"buy milk","datetime":null,"confidence":"high"}"#;
        let r = parse_assistant_response(text).unwrap();
        assert!(r.datetime.is_none());
    }

    #[test]
    fn assistant_response_strips_code_fence() {
        let text = "```json\n{\"title\":\"x\",\"datetime\":null,\"confidence\":\"high\"}\n```";
        let r = parse_assistant_response(text).unwrap();
        assert_eq!(r.title, "x");
    }

    #[test]
    fn assistant_response_strips_preamble() {
        let text = "Here's the JSON:\n{\"title\":\"x\",\"datetime\":null,\"confidence\":\"high\"}";
        let r = parse_assistant_response(text).unwrap();
        assert_eq!(r.title, "x");
    }

    #[test]
    fn assistant_response_handles_nested_braces_in_string() {
        // Title containing '}' used to prematurely terminate the
        // strip_to_outer_object scan. Now the string-state machine
        // ignores braces inside quoted strings.
        let text = r#"{"title":"weird } title","datetime":null,"confidence":"high"}"#;
        let r = parse_assistant_response(text).unwrap();
        assert_eq!(r.title, "weird } title");
    }

    #[test]
    fn assistant_response_rejects_empty() {
        assert!(parse_assistant_response("").is_none());
        assert!(parse_assistant_response("   ").is_none());
    }

    #[test]
    fn assistant_response_rejects_garbage() {
        assert!(parse_assistant_response("not json").is_none());
        assert!(parse_assistant_response("{not json}").is_none());
    }

    #[test]
    fn iso_datetime_parses_full_form() {
        let p = parse_iso_datetime("2026-04-27T14:00:00").unwrap();
        assert_eq!(p.year, 2026);
        assert_eq!(p.month, 4);
        assert_eq!(p.day, 27);
        assert_eq!(p.hour, 14);
        assert_eq!(p.minute, 0);
    }

    #[test]
    fn iso_datetime_parses_without_seconds() {
        let p = parse_iso_datetime("2026-04-27T09:30").unwrap();
        assert_eq!(p.hour, 9);
        assert_eq!(p.minute, 30);
    }

    #[test]
    fn iso_datetime_parses_space_separator() {
        let p = parse_iso_datetime("2026-04-27 09:30:00").unwrap();
        assert_eq!(p.year, 2026);
        assert_eq!(p.hour, 9);
    }

    #[test]
    fn iso_datetime_ignores_trailing_zone() {
        let p = parse_iso_datetime("2026-04-27T14:00:00Z").unwrap();
        assert_eq!(p.hour, 14);
        let p = parse_iso_datetime("2026-04-27T14:00:00+01:00").unwrap();
        assert_eq!(p.hour, 14);
    }

    #[test]
    fn iso_datetime_rejects_junk() {
        assert!(parse_iso_datetime("not a date").is_none());
        assert!(parse_iso_datetime("2026").is_none());
        assert!(parse_iso_datetime("2026-13-01T00:00").is_none()); // month 13
        assert!(parse_iso_datetime("2026-04-27T25:00").is_none()); // hour 25
    }

    #[test]
    fn confirm_round_trips_simple_title() {
        let encoded = encode_confirm("tasks", 1777993200000, "call mum");
        assert_eq!(
            encoded,
            "ariconfirmreminder tasks 1777993200000 63616c6c206d756d"
        );
        let decoded = parse_confirm(&encoded).unwrap();
        assert_eq!(decoded.destination, "tasks");
        assert_eq!(decoded.epoch_ms, 1777993200000);
        assert_eq!(decoded.title, "call mum");
    }

    #[test]
    fn confirm_round_trips_unicode_title() {
        let title = "café déjeuner";
        let encoded = encode_confirm("calendar", 0, title);
        let decoded = parse_confirm(&encoded).unwrap();
        assert_eq!(decoded.title, title);
        assert_eq!(decoded.destination, "calendar");
        assert_eq!(decoded.epoch_ms, 0);
    }

    #[test]
    fn confirm_round_trips_punctuation_title() {
        // Quotes, colons, exclamation marks — all normally mangled by
        // `normalize_input`, but hex-encoded they survive.
        let title = "Remember: \"get milk\"!";
        let encoded = encode_confirm("tasks", 123, title);
        let decoded = parse_confirm(&encoded).unwrap();
        assert_eq!(decoded.title, title);
    }

    #[test]
    fn confirm_rejects_wrong_prefix() {
        assert!(parse_confirm("remind me about mum").is_none());
        assert!(parse_confirm("aricancelreminder tasks 1").is_none());
    }

    #[test]
    fn confirm_rejects_bad_destination() {
        assert!(parse_confirm("ariconfirmreminder fridge 0 6d756d").is_none());
    }

    #[test]
    fn confirm_rejects_bad_epoch() {
        assert!(parse_confirm("ariconfirmreminder tasks notanumber 6d756d").is_none());
    }

    #[test]
    fn confirm_rejects_bad_hex() {
        // Odd length, invalid bytes.
        assert!(parse_confirm("ariconfirmreminder tasks 0 xyz").is_none());
        assert!(parse_confirm("ariconfirmreminder tasks 0 abc").is_none());
    }

    #[test]
    fn confirm_survives_normaliser_shape() {
        // The engine's normalize_input collapses whitespace and lower-
        // cases; every token in the encoded form is already lowercase
        // alphanumeric (destination is one of three lowercase words,
        // epoch is digits, title hex is lowercase). No mangling.
        let encoded = encode_confirm("tasks", 1777993200000, "call mum");
        for ch in encoded.chars() {
            assert!(ch.is_ascii_alphanumeric() || ch == ' ');
            assert!(!ch.is_ascii_uppercase());
        }
    }

    #[test]
    fn compose_prompt_substitutes_fields() {
        let parsed = parse::Parsed {
            title: "call sarah on the 27th".to_string(),
            when: parse::When::None,
            list_hint: None,
            speak_template: String::new(),
            confidence: parse::Confidence::Low,
            unparsed: Some("27th".to_string()),
        };
        let prompt = compose_prompt(
            "remind me to call sarah on the 27th",
            &parsed,
            "Monday, 27 April 2026 (2026-04-27)",
            "en",
        );
        assert!(prompt.contains("remind me to call sarah on the 27th"));
        assert!(prompt.contains("call sarah on the 27th"));
        assert!(prompt.contains("27th"));
        assert!(prompt.contains("STRICT JSON"));
        assert!(prompt.contains("Monday, 27 April 2026"));
        // English locale → no language tail appended
        assert!(!prompt.contains("speaking Italian"));
    }

    #[test]
    fn compose_prompt_appends_italian_language_tail() {
        let parsed = parse::Parsed {
            title: "chiamare sara".to_string(),
            when: parse::When::None,
            list_hint: None,
            speak_template: String::new(),
            confidence: parse::Confidence::Partial,
            unparsed: Some("27".to_string()),
        };
        let prompt = compose_prompt(
            "ricordami di chiamare sara il 27",
            &parsed,
            "lunedì, 27 aprile 2026 (2026-04-27)",
            "it",
        );
        // Italian-locale call should append the explicit "respond in
        // Italian" directive, while leaving the structural enums
        // (datetime, confidence, follow_up) untouched.
        assert!(
            prompt.contains("speaking Italian"),
            "prompt must include the Italian language tail when locale=it"
        );
        assert!(
            prompt.contains("`title` and `clarification` in Italian"),
            "prompt must scope the language directive to the human-language fields"
        );
        // Structural backbone of the prompt is unchanged.
        assert!(prompt.contains("STRICT JSON"));
    }

    #[test]
    fn compose_prompt_no_tail_for_unknown_locale() {
        let parsed = parse::Parsed {
            title: "x".to_string(),
            when: parse::When::None,
            list_hint: None,
            speak_template: String::new(),
            confidence: parse::Confidence::High,
            unparsed: None,
        };
        let prompt = compose_prompt("x", &parsed, "today", "ja");
        // Locales we haven't explicitly mapped fall through to plain
        // English prompt — the `ja` user gets no language hint at all
        // rather than a wrong one.
        assert!(!prompt.contains("speaking Italian"));
        assert!(!prompt.contains("speaking Spanish"));
        assert!(!prompt.contains("speaking French"));
        assert!(!prompt.contains("speaking German"));
    }

    #[test]
    fn compose_prompt_includes_concrete_clarification_guidance() {
        // Regression for the "Did you mean next Friday at 3pm?" echo
        // bug — the prompt must explicitly tell the model to phrase
        // its clarification using the resolved concrete value, not
        // the user's original wording.
        let parsed = parse::Parsed {
            title: "call mum".to_string(),
            when: parse::When::None,
            list_hint: None,
            speak_template: String::new(),
            confidence: parse::Confidence::Partial,
            unparsed: Some("next".to_string()),
        };
        let prompt = compose_prompt(
            "remind me next friday at 3pm to call mum",
            &parsed,
            "Monday, 27 April 2026 (2026-04-27)",
            "en",
        );
        // Two guard rails: instruction to use concrete resolved values,
        // AND a worked example so the model has a pattern to match.
        assert!(
            prompt.contains("concrete resolved value"),
            "prompt must explicitly require the concrete resolved value in the clarification"
        );
        assert!(
            prompt.contains("the 1st of May"),
            "prompt should include a worked example of a good clarification phrasing"
        );
        assert!(
            prompt.contains("paraphrasing the user's input"),
            "prompt should call out the bad paraphrase pattern explicitly so the model avoids it"
        );
    }
}
