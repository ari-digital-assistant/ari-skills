//! Intent extraction for the reminder skill.
//!
//! Pure string crunching — no host imports, no clock reads. The
//! Android frontend resolves the structured `When` descriptor against
//! the device's local zone at insert time, so this module never has
//! to know what "now" or "today" mean in any concrete sense.
//!
//! Two utterance shapes are recognised:
//!
//! - **Named list**: `add X to my Y list` / `put X on the Y list` →
//!   produces `Parsed { title: X, list_hint: Some(Y), when: None }`.
//!   No time parsing because list-add utterances are always untimed
//!   in v0.1.
//!
//! - **Reminder**: `remind me [at TIME|in DURATION|tomorrow|today] to X`
//!   in any of the common orderings. Produces
//!   `Parsed { title: X, list_hint: None, when: ... }` where `when`
//!   is one of the four [`When`] variants.
//!
//! Anything that doesn't match either shape falls through to a bare
//! `Parsed { title: <input verbatim>, ... }` so the frontend gets
//! something usable rather than nothing.

use alloc::string::{String, ToString};

#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    pub title: String,
    pub when: When,
    pub list_hint: Option<String>,
    pub speak_template: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum When {
    /// No time given. Frontend creates an untimed VTODO regardless of
    /// the user's destination setting.
    None,
    /// Relative offset from the moment the frontend processes the
    /// action. Skill computed from "in N minutes/hours/seconds".
    InSeconds(u64),
    /// Absolute local clock + day offset. `day_offset` is 0 for today,
    /// 1 for tomorrow, etc. The frontend bumps a "today at past time"
    /// entry to "tomorrow at that time" defensively.
    LocalClock {
        hour: u8,
        minute: u8,
        day_offset: u32,
    },
    /// Date only ("tomorrow" with no time-of-day). Frontend inserts a
    /// VTODO with a due date but no due time.
    DateOnly { day_offset: u32 },
}

const SPEAK_TEMPLATE: &str = "Added {title} to your {list_name} list";

pub fn parse(input: &str) -> Parsed {
    let normalised = input.trim().to_lowercase();

    // Named-list shape first — these utterances don't start with
    // "remind me" so we pattern-match them on their own.
    if let Some(p) = parse_named_list(&normalised) {
        return p;
    }

    // Strip the "remind me" / "set a reminder" lead, leaving the
    // payload (everything after the trigger phrase). If no recognised
    // lead was found, treat the whole input as the payload — better to
    // create a usable reminder than to refuse with an empty title.
    let payload = strip_reminder_lead(&normalised).unwrap_or(normalised.as_str());

    let (when, payload_after_when) = extract_when(payload);
    let title = clean_title(&payload_after_when);

    Parsed {
        title,
        when,
        list_hint: None,
        speak_template: SPEAK_TEMPLATE.to_string(),
    }
}

// ── Named-list shape ───────────────────────────────────────────────────

/// `add X to my Y list` / `put X on the Y list` etc. Returns None if
/// the input doesn't fit; reminder parsing then takes over.
fn parse_named_list(input: &str) -> Option<Parsed> {
    // Anchored on a leading "add " or "put ". Other verbs ("save",
    // "stick", "throw") could be added later if real users ask for
    // them — for v0.1 keep the surface narrow so we don't accidentally
    // grab utterances that should match a different skill.
    let rest = input.strip_prefix("add ").or_else(|| input.strip_prefix("put "))?;

    // Locate the connector " to " or " on ", then a possessive
    // determiner ("my " / "the "), then capture up to " list". Walk
    // every occurrence of the connector — the first one might lead to
    // text that doesn't end in " list" (e.g. "add cream to the
    // coffee"), in which case we fall through to the next match.
    for connector in [" to ", " on "] {
        let mut search_from = 0usize;
        while let Some(rel) = rest[search_from..].find(connector) {
            let split = search_from + rel;
            let item = rest[..split].trim();
            let after = &rest[split + connector.len()..];

            // Possessive determiner is required so we don't match
            // bare utterances like "add cream to the coffee" — that's
            // not a list intent.
            let after_det = after
                .strip_prefix("my ")
                .or_else(|| after.strip_prefix("the "));

            if let Some(after_det) = after_det {
                if let Some(list_name) = after_det.strip_suffix(" list") {
                    let list_name = list_name.trim();
                    if !list_name.is_empty() && !item.is_empty() {
                        return Some(Parsed {
                            title: item.to_string(),
                            when: When::None,
                            list_hint: Some(list_name.to_string()),
                            speak_template: SPEAK_TEMPLATE.to_string(),
                        });
                    }
                }
            }

            // This connector match didn't pan out — advance past it
            // and try again. Without the `+1` we'd loop forever on the
            // same index.
            search_from = split + 1;
        }
    }

    None
}

// ── Reminder lead stripping ────────────────────────────────────────────

fn strip_reminder_lead(input: &str) -> Option<&str> {
    // Try the longer phrases first so we don't half-strip.
    for lead in [
        "set me a reminder ",
        "set a reminder ",
        "create a reminder ",
        "remind me ",
    ] {
        if let Some(rest) = input.strip_prefix(lead) {
            return Some(rest);
        }
    }
    None
}

// ── When extraction ────────────────────────────────────────────────────

/// Pull the time descriptor out of the payload and return it alongside
/// the payload with the time phrase removed. Order matters — relative
/// ("in N min") is checked before absolute ("at HH") because "remind
/// me in 10 minutes at the next opportunity" should resolve to the
/// relative offset, not the literal "at" pattern.
fn extract_when(payload: &str) -> (When, String) {
    if let Some((when, remainder)) = extract_in_relative(payload) {
        return (when, remainder);
    }
    if let Some((when, remainder)) = extract_at_time(payload) {
        return (when, remainder);
    }
    if let Some((when, remainder)) = extract_day_only(payload) {
        return (when, remainder);
    }
    (When::None, payload.to_string())
}

/// Match "in N minutes" / "in N hours" / "in N seconds" anywhere in
/// the payload. Returns the offset and the payload with that phrase
/// stripped out.
fn extract_in_relative(payload: &str) -> Option<(When, String)> {
    let units: [(&[&str], u64); 3] = [
        (&["seconds", "second", "secs", "sec"], 1),
        (&["minutes", "minute", "mins", "min"], 60),
        (&["hours", "hour", "hrs", "hr"], 3600),
    ];

    let words: alloc::vec::Vec<&str> = payload.split_whitespace().collect();
    for i in 0..words.len() {
        if words[i] != "in" {
            continue;
        }
        // Need "in <number> <unit>" — at least two more tokens.
        if i + 2 >= words.len() {
            continue;
        }
        let n: u64 = match words[i + 1].parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let unit_word = words[i + 2].trim_end_matches(',');
        let secs_per = units
            .iter()
            .find_map(|(names, secs)| if names.contains(&unit_word) { Some(*secs) } else { None });
        let secs_per = match secs_per {
            Some(s) => s,
            None => continue,
        };

        let when = When::InSeconds(n * secs_per);
        let remainder = remove_words(payload, i, 3);
        return Some((when, remainder));
    }
    None
}

/// Match "at H[:MM]?(am|pm)?" / "at noon" / "at midnight". Picks up
/// "tomorrow"/"today" as a sibling token to set the day_offset.
fn extract_at_time(payload: &str) -> Option<(When, String)> {
    let words: alloc::vec::Vec<&str> = payload.split_whitespace().collect();
    for i in 0..words.len() {
        if words[i] != "at" {
            continue;
        }
        if i + 1 >= words.len() {
            continue;
        }
        let token = words[i + 1].trim_end_matches(',');
        let (hour, minute) = match parse_clock_token(token) {
            Some(hm) => hm,
            None => continue,
        };

        // Look for a sibling "tomorrow"/"today" anywhere in the
        // payload; it can sit before or after the "at TIME" phrase.
        let mut day_offset: u32 = 0;
        let mut day_word_index: Option<usize> = None;
        for (j, w) in words.iter().enumerate() {
            if j == i || j == i + 1 {
                continue;
            }
            match w.trim_end_matches(',') {
                "tomorrow" => {
                    day_offset = 1;
                    day_word_index = Some(j);
                    break;
                }
                "today" => {
                    day_offset = 0;
                    day_word_index = Some(j);
                    break;
                }
                _ => {}
            }
        }

        let when = When::LocalClock {
            hour,
            minute,
            day_offset,
        };

        // Strip "at TIME" first, then the day word if it was set.
        // Strip in descending index order so the earlier strip doesn't
        // shift indices used by the later one.
        let mut indices_to_drop: alloc::vec::Vec<usize> = alloc::vec::Vec::new();
        indices_to_drop.push(i);
        indices_to_drop.push(i + 1);
        if let Some(j) = day_word_index {
            indices_to_drop.push(j);
        }
        indices_to_drop.sort_unstable();
        let kept: alloc::vec::Vec<&str> = words
            .iter()
            .enumerate()
            .filter_map(|(idx, w)| if indices_to_drop.contains(&idx) { None } else { Some(*w) })
            .collect();
        return Some((when, kept.join(" ")));
    }
    None
}

/// Match a bare "tomorrow" or "today" with no time-of-day → date-only
/// VTODO. Skipped if `at TIME` already consumed a day word (extract_at_time
/// runs first).
fn extract_day_only(payload: &str) -> Option<(When, String)> {
    let words: alloc::vec::Vec<&str> = payload.split_whitespace().collect();
    for (i, w) in words.iter().enumerate() {
        let cleaned = w.trim_end_matches(',');
        let day_offset = match cleaned {
            "tomorrow" => 1u32,
            "today" => 0u32,
            _ => continue,
        };
        let when = When::DateOnly { day_offset };
        let remainder = remove_words(payload, i, 1);
        return Some((when, remainder));
    }
    None
}

/// Parse "5pm" / "5:30pm" / "9am" / "17:00" / "noon" / "midnight" into
/// (hour, minute) in 24h. Returns None for anything else.
fn parse_clock_token(token: &str) -> Option<(u8, u8)> {
    if token == "noon" {
        return Some((12, 0));
    }
    if token == "midnight" {
        return Some((0, 0));
    }

    // am/pm suffix detection.
    let (numeric, ampm) = if let Some(stripped) = token.strip_suffix("am") {
        (stripped, Some(false))
    } else if let Some(stripped) = token.strip_suffix("pm") {
        (stripped, Some(true))
    } else {
        (token, None)
    };

    let (hour_str, minute_str) = match numeric.find(':') {
        Some(idx) => (&numeric[..idx], &numeric[idx + 1..]),
        None => (numeric, "0"),
    };

    let mut hour: u8 = hour_str.parse().ok()?;
    let minute: u8 = minute_str.parse().ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }

    if let Some(is_pm) = ampm {
        // 12am → 0, 12pm → 12, 1-11am unchanged, 1-11pm += 12.
        hour = match (hour, is_pm) {
            (12, false) => 0,
            (12, true) => 12,
            (h, false) => h,
            (h, true) => h + 12,
        };
    }

    Some((hour, minute))
}

// ── Title cleanup ──────────────────────────────────────────────────────

fn clean_title(payload: &str) -> String {
    let mut s = payload.trim().to_string();

    // Drop a leading "to " ("remind me to walk the dog" → payload is
    // "to walk the dog" after the lead-strip).
    if let Some(rest) = s.strip_prefix("to ") {
        s = rest.to_string();
    }
    // Or "about " for "remind me about the meeting".
    else if let Some(rest) = s.strip_prefix("about ") {
        s = rest.to_string();
    }

    // Collapse runs of whitespace introduced by removing words from
    // the middle of the payload.
    let collapsed: String = s.split_whitespace().collect::<alloc::vec::Vec<_>>().join(" ");

    // Drop a trailing comma if one was orphaned by the time strip.
    collapsed.trim_end_matches(',').trim().to_string()
}

// ── Helpers ────────────────────────────────────────────────────────────

/// Drop `count` whitespace-separated tokens starting at index `start`,
/// returning the remaining payload joined by spaces.
fn remove_words(payload: &str, start: usize, count: usize) -> String {
    payload
        .split_whitespace()
        .enumerate()
        .filter_map(|(i, w)| {
            if i >= start && i < start + count {
                None
            } else {
                Some(w)
            }
        })
        .collect::<alloc::vec::Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_clock_pm() {
        assert_eq!(parse_clock_token("5pm"), Some((17, 0)));
        assert_eq!(parse_clock_token("5:30pm"), Some((17, 30)));
        assert_eq!(parse_clock_token("12pm"), Some((12, 0)));
    }

    #[test]
    fn parse_clock_am() {
        assert_eq!(parse_clock_token("9am"), Some((9, 0)));
        assert_eq!(parse_clock_token("12am"), Some((0, 0)));
        assert_eq!(parse_clock_token("9:15am"), Some((9, 15)));
    }

    #[test]
    fn parse_clock_24h() {
        assert_eq!(parse_clock_token("17:00"), Some((17, 0)));
        assert_eq!(parse_clock_token("9:00"), Some((9, 0)));
    }

    #[test]
    fn parse_clock_named() {
        assert_eq!(parse_clock_token("noon"), Some((12, 0)));
        assert_eq!(parse_clock_token("midnight"), Some((0, 0)));
    }

    #[test]
    fn parse_clock_rejects_garbage() {
        assert_eq!(parse_clock_token("dog"), None);
        assert_eq!(parse_clock_token("25"), None);
        assert_eq!(parse_clock_token("12:99pm"), None);
    }

    #[test]
    fn named_list_basic() {
        let p = parse("add milk to my shopping list");
        assert_eq!(p.title, "milk");
        assert_eq!(p.list_hint.as_deref(), Some("shopping"));
        assert_eq!(p.when, When::None);
    }

    #[test]
    fn named_list_put_on_the() {
        let p = parse("put eggs on the shopping list");
        assert_eq!(p.title, "eggs");
        assert_eq!(p.list_hint.as_deref(), Some("shopping"));
    }

    #[test]
    fn named_list_multi_word_name() {
        let p = parse("add deadline review to my work projects list");
        assert_eq!(p.title, "deadline review");
        assert_eq!(p.list_hint.as_deref(), Some("work projects"));
    }

    #[test]
    fn named_list_rejects_non_list_to_phrase() {
        // "to the coffee" is not a list intent — should fall through to
        // the reminder parser, which strips no lead and treats the whole
        // string as the title.
        let p = parse("add cream to the coffee");
        assert_eq!(p.list_hint, None);
    }

    #[test]
    fn untimed_reminder_strips_to_lead() {
        let p = parse("remind me to buy milk");
        assert_eq!(p.title, "buy milk");
        assert_eq!(p.when, When::None);
    }

    #[test]
    fn relative_minutes_extracts_offset_and_strips_phrase() {
        let p = parse("remind me in 30 minutes to check the oven");
        assert_eq!(p.title, "check the oven");
        assert_eq!(p.when, When::InSeconds(1800));
    }

    #[test]
    fn relative_hours_works() {
        let p = parse("remind me in 2 hours to check the oven");
        assert_eq!(p.when, When::InSeconds(7200));
    }

    #[test]
    fn at_time_today_extracts_clock_and_strips() {
        let p = parse("remind me to walk the dog at 5pm");
        assert_eq!(p.title, "walk the dog");
        assert_eq!(
            p.when,
            When::LocalClock { hour: 17, minute: 0, day_offset: 0 },
        );
    }

    #[test]
    fn at_time_with_tomorrow_sets_day_offset_one() {
        let p = parse("remind me at 9am tomorrow to call the dentist");
        assert_eq!(p.title, "call the dentist");
        assert_eq!(
            p.when,
            When::LocalClock { hour: 9, minute: 0, day_offset: 1 },
        );
    }

    #[test]
    fn at_time_tomorrow_can_lead_the_payload() {
        let p = parse("remind me tomorrow at 3pm to pick up the parcel");
        assert_eq!(p.title, "pick up the parcel");
        assert_eq!(
            p.when,
            When::LocalClock { hour: 15, minute: 0, day_offset: 1 },
        );
    }

    #[test]
    fn tomorrow_alone_yields_date_only() {
        let p = parse("remind me about laundry tomorrow");
        assert_eq!(p.title, "laundry");
        assert_eq!(p.when, When::DateOnly { day_offset: 1 });
    }

    #[test]
    fn empty_input_returns_empty_title() {
        let p = parse("");
        assert_eq!(p.title, "");
        assert_eq!(p.when, When::None);
    }

    #[test]
    fn unrecognised_input_falls_through_to_bare_title() {
        // Doesn't start with any reminder lead and doesn't match the
        // named-list shape — frontend gets the original text as the
        // title so the user still gets something usable.
        let p = parse("eggs and bacon");
        assert_eq!(p.title, "eggs and bacon");
        assert_eq!(p.when, When::None);
    }
}
