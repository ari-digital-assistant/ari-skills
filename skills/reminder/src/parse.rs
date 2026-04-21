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
    /// How sure the skill is about the extracted parameters. Drives
    /// the warn-and-commit UX on the frontend: `High` fires the
    /// normal flow, `Partial` speaks a "I set X — did you also mean
    /// Y?" confirmation, `Low` does the same but with a stronger
    /// "this might be wrong" framing. See [`Confidence`].
    pub confidence: Confidence,
    /// Residual title phrase that looked like a date/time hint but
    /// the skill couldn't consume. `None` when confidence is
    /// `High`. The frontend quotes this back to the user in the
    /// confirmation prompt so they can see what got lost.
    pub unparsed: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    High,
    Partial,
    Low,
}

impl Confidence {
    pub fn as_envelope_str(self) -> &'static str {
        match self {
            Confidence::High => "high",
            Confidence::Partial => "partial",
            Confidence::Low => "low",
        }
    }
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
    /// Absolute local clock on a named weekday ("at 3pm on Friday").
    /// Skill can't compute the day offset because it doesn't know the
    /// host's local weekday — Android resolves `weekday` to the next
    /// occurrence in local time, bumping to next week if the requested
    /// same-day time has already passed.
    /// `weekday` uses ISO ordering: 0=Monday .. 6=Sunday.
    LocalClockOnWeekday { hour: u8, minute: u8, weekday: u8 },
    /// Absolute local clock on a calendar date ("at 10am on the 27th
    /// of April"). Year is deliberately absent — the skill has no
    /// clock or locale; Android resolves to this year if the date is
    /// today or later, next year otherwise. `month` is 1..=12 and
    /// `day` is 1..=31; impossible combinations (Feb 31 etc.) are
    /// left to Android's date library to reject.
    LocalClockOnDate { hour: u8, minute: u8, month: u8, day: u8 },
    /// Date only ("tomorrow" with no time-of-day). Frontend inserts a
    /// VTODO with a due date but no due time.
    DateOnly { day_offset: u32 },
    /// Date only on a named weekday ("on Friday" with no time).
    /// Same weekday semantics as `LocalClockOnWeekday`.
    DateOnlyWeekday { weekday: u8 },
    /// Date only on a calendar date ("on the 27th of April"). Same
    /// year-inference rules as `LocalClockOnDate`.
    DateOnlyDate { month: u8, day: u8 },
}

/// Internal result of scanning the payload for a day anchor. Kept as a
/// private enum rather than threading two optional fields through
/// every call site.
#[derive(Debug, Clone, Copy)]
enum DayTarget {
    Offset(u32),
    Weekday(u8),
}

/// Map an English weekday name to its ISO index (Monday=0..Sunday=6).
/// Returns None for anything else; the caller uses that to decide
/// whether the word is a day anchor or a normal payload token.
fn weekday_from_word(w: &str) -> Option<u8> {
    match w {
        "monday" => Some(0),
        "tuesday" => Some(1),
        "wednesday" => Some(2),
        "thursday" => Some(3),
        "friday" => Some(4),
        "saturday" => Some(5),
        "sunday" => Some(6),
        _ => None,
    }
}

/// Map an English month name to its 1-based index.
/// Full names only for v0.1 — abbreviated forms ("jan", "feb") are a
/// future extension, listed as a coverage gap in the skill docs.
fn month_from_word(w: &str) -> Option<u8> {
    match w {
        "january" => Some(1),
        "february" => Some(2),
        "march" => Some(3),
        "april" => Some(4),
        "may" => Some(5),
        "june" => Some(6),
        "july" => Some(7),
        "august" => Some(8),
        "september" => Some(9),
        "october" => Some(10),
        "november" => Some(11),
        "december" => Some(12),
        _ => None,
    }
}

/// Parse a day-of-month token with optional ordinal suffix:
///   "27" / "27th" / "1st" / "2nd" / "3rd" / "22nd"
/// Rejects anything outside 1..=31. The suffix rules aren't strictly
/// enforced ("28st" would parse as 28) because the engine normaliser
/// often rewrites ordinal words into bare digits before we see them,
/// and being too pedantic here would reject legitimate input that
/// lost its suffix in normalisation.
fn parse_day_ordinal(token: &str) -> Option<u8> {
    let stripped = token
        .strip_suffix("st")
        .or_else(|| token.strip_suffix("nd"))
        .or_else(|| token.strip_suffix("rd"))
        .or_else(|| token.strip_suffix("th"))
        .unwrap_or(token);
    let n: u8 = stripped.parse().ok()?;
    if (1..=31).contains(&n) { Some(n) } else { None }
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
    let (confidence, unparsed) = assess_confidence(&title, &when);

    Parsed {
        title,
        when,
        list_hint: None,
        speak_template: SPEAK_TEMPLATE.to_string(),
        confidence,
        unparsed,
    }
}

/// Scan the post-extraction title for tokens that look like date/time
/// anchors the parser should have consumed. Each survivor is a sign
/// the user said something the parser didn't understand.
///
/// Severity:
///   High     — no suspicious residue.
///   Partial  — residue present, but the skill did extract a concrete
///              day/time anchor. User's intent partly captured.
///   Low      — residue present AND the `when` fell back to the
///              "no anchor" default (`LocalClock{day_offset:0}` from
///              nothing, or `None`). User's intent probably missed.
fn assess_confidence(title: &str, when: &When) -> (Confidence, Option<String>) {
    let mut residue: alloc::vec::Vec<&str> = alloc::vec::Vec::new();
    for tok in title.split_whitespace() {
        let cleaned = tok.trim_end_matches(',');
        if is_datetime_residue(cleaned) {
            residue.push(tok);
        }
    }

    if residue.is_empty() {
        return (Confidence::High, None);
    }

    let unparsed = residue.join(" ");
    let fell_back = matches!(
        when,
        When::None | When::LocalClock { day_offset: 0, .. }
    );
    let level = if fell_back { Confidence::Low } else { Confidence::Partial };
    (level, Some(unparsed))
}

/// Is this title word a likely-missed date/time anchor?
///
/// Kept deliberately narrow — false positives here produce spurious
/// warnings, which is worse than letting the occasional genuine
/// residue slip through. Plain numbers without an ordinal suffix
/// aren't flagged (a title like "buy 3 apples" isn't a date hint);
/// general prepositions (`at`, `on`, `by`, `in`) aren't flagged
/// either because they're too common in ordinary English titles.
fn is_datetime_residue(w: &str) -> bool {
    if weekday_from_word(w).is_some() {
        return true;
    }
    if month_from_word(w).is_some() {
        return true;
    }
    if looks_like_ordinal_date(w) {
        return true;
    }
    // "next / this / last" surviving in the title almost always means
    // the user qualified a weekday or date and we dropped the
    // qualifier ("next tuesday" → tuesday consumed, "next" stranded).
    // The false-positive rate in reminder titles (e.g. "next thing on
    // the agenda") is low enough that flagging is the right call.
    matches!(
        w,
        "today" | "tomorrow" | "tonight" | "next" | "this" | "last"
    )
}

/// `27th` / `1st` / `22nd` / `3rd` / even `40th` (out-of-range for a
/// day but still looks like the user was trying to say a date).
/// Plain digits (`27`, `5`) are intentionally excluded — they appear
/// in legitimate titles as counts, and we don't want to flag them.
/// This is about "the word looks date-shaped", not "the word is a
/// valid day number"; the scanner already rejects invalid days
/// upstream, so by the time we see one in the title the point is
/// that the user said something the parser ignored.
fn looks_like_ordinal_date(w: &str) -> bool {
    for suffix in ["st", "nd", "rd", "th"] {
        if let Some(stripped) = w.strip_suffix(suffix) {
            if !stripped.is_empty() && stripped.chars().all(|c| c.is_ascii_digit()) {
                return true;
            }
        }
    }
    false
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
                        // Named-list utterances are always untimed in
                        // v0.1 and don't carry any date/time anchors
                        // the scanner could miss, so confidence is
                        // always High here.
                        return Some(Parsed {
                            title: item.to_string(),
                            when: When::None,
                            list_hint: Some(list_name.to_string()),
                            speak_template: SPEAK_TEMPLATE.to_string(),
                            confidence: Confidence::High,
                            unparsed: None,
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

/// Match "at H[:MM]?(am|pm)?" / "at H M [am|pm]" / "at noon" /
/// "at midnight". Picks up "tomorrow"/"today" as a sibling token to
/// set the day_offset.
///
/// The multi-token "H M am/pm" shape exists because the engine's
/// number-word normaliser splits "nine thirty" into "9 30" and
/// "nine thirty pm" into "9 30 pm" before the skill ever sees the
/// payload. Single-token forms like "9:30pm" or "5pm" are handled
/// by [`parse_clock_token`]; multi-token forms are handled here.
fn extract_at_time(payload: &str) -> Option<(When, String)> {
    let words: alloc::vec::Vec<&str> = payload.split_whitespace().collect();
    for i in 0..words.len() {
        if words[i] != "at" {
            continue;
        }
        if i + 1 >= words.len() {
            continue;
        }
        let (hour, minute, clock_tokens) = match parse_clock_phrase(&words, i + 1) {
            Some(x) => x,
            None => continue,
        };

        // Indices taken by the "at" keyword plus the clock phrase. The
        // day-word scan must skip these, and the final strip must drop
        // them.
        let clock_end = i + clock_tokens;

        // Calendar-date anchor has priority: if the user gave a
        // specific date ("on the 27th of April"), that's what they
        // meant — don't let a stray weekday word earlier in the
        // payload overrule it. Fall back to weekday/today/tomorrow
        // if no calendar date was found.
        let mut indices_to_drop: alloc::vec::Vec<usize> = alloc::vec::Vec::new();
        for idx in i..=clock_end {
            indices_to_drop.push(idx);
        }

        let (cal_match, cal_indices) = scan_calendar_date(&words, i, clock_end);
        let when = if let Some((month, day)) = cal_match {
            indices_to_drop.extend(cal_indices);
            When::LocalClockOnDate { hour, minute, month, day }
        } else {
            // Look for a sibling day anchor anywhere in the payload;
            // it can sit before or after the "at TIME" phrase. Three
            // shapes matter:
            //   - "today" / "tomorrow" (one token, direct offset)
            //   - "<weekday>"          (one token, skill emits weekday,
            //                           host resolves)
            //   - "on <weekday>"       (two tokens, strip both)
            let (day_target, day_word_indices) = scan_day_anchor(&words, i, clock_end);
            indices_to_drop.extend(day_word_indices);
            match day_target {
                Some(DayTarget::Offset(off)) => When::LocalClock {
                    hour,
                    minute,
                    day_offset: off,
                },
                Some(DayTarget::Weekday(wd)) => When::LocalClockOnWeekday {
                    hour,
                    minute,
                    weekday: wd,
                },
                None => When::LocalClock {
                    hour,
                    minute,
                    day_offset: 0,
                },
            }
        };

        let kept: alloc::vec::Vec<&str> = words
            .iter()
            .enumerate()
            .filter_map(|(idx, w)| if indices_to_drop.contains(&idx) { None } else { Some(*w) })
            .collect();
        return Some((when, kept.join(" ")));
    }
    None
}

/// Scan the payload for a calendar-date anchor of the day-first shape:
///   (on)? (the)? <day-ordinal> (of)? <month>
///
/// Examples that hit:
///   "27th of april", "27th april", "on the 27th of april",
///   "twenty seventh of april" (after engine number-word normalisation
///    collapses the ordinal compound into "27")
///
/// Examples that miss, deliberately:
///   "april 27th" / "april the 27th" (month-first form, not in scope)
///   "the 27th" (no month)
///   "april" (no day)
///
/// Returns (month, day) on match, plus every token index the caller
/// should strip from the title — inclusive range from the leading
/// "on" / "the" through the month word, excluding anything inside
/// `exclude_lo..=exclude_hi` (so a clock phrase caught earlier can't
/// have its indices double-stripped).
fn scan_calendar_date(
    words: &[&str],
    exclude_lo: usize,
    exclude_hi: usize,
) -> (Option<(u8, u8)>, alloc::vec::Vec<usize>) {
    for j in 0..words.len() {
        if j >= exclude_lo && j <= exclude_hi {
            continue;
        }
        let day = match parse_day_ordinal(words[j].trim_end_matches(',')) {
            Some(d) => d,
            None => continue,
        };

        // Month follows the day, optionally with an "of" connector.
        let mut month_idx = j + 1;
        if month_idx < words.len() && words[month_idx].trim_end_matches(',') == "of" {
            month_idx += 1;
        }
        if month_idx >= words.len() || (month_idx >= exclude_lo && month_idx <= exclude_hi) {
            continue;
        }
        let month = match month_from_word(words[month_idx].trim_end_matches(',')) {
            Some(m) => m,
            None => continue,
        };

        // Walk backwards to also strip a leading "the", a leading
        // weekday name (so "on friday the 27th of april" doesn't
        // leave "on friday" stranded in the title), and a leading
        // "on". Order matters: "the" sits closest to the day, then
        // the weekday, then "on" wraps the whole lot.
        let mut first = j;
        if first > 0 && words[first - 1].trim_end_matches(',') == "the" {
            first -= 1;
        }
        if first > 0 && weekday_from_word(words[first - 1].trim_end_matches(',')).is_some() {
            first -= 1;
        }
        if first > 0 && words[first - 1].trim_end_matches(',') == "on" {
            first -= 1;
        }

        let mut indices = alloc::vec::Vec::new();
        for idx in first..=month_idx {
            if idx >= exclude_lo && idx <= exclude_hi {
                continue;
            }
            indices.push(idx);
        }
        return (Some((month, day)), indices);
    }
    (None, alloc::vec::Vec::new())
}

/// Scan the payload for the first day anchor outside the "at TIME"
/// phrase. Returns the matched target and every token index that
/// should be stripped from the title — one index for bare anchors,
/// two for "on <weekday>". Skips indices in `exclude_lo..=exclude_hi`
/// so a weekday-looking token caught as part of the clock phrase
/// itself (not that one exists today, but defensive) can't be
/// consumed twice.
fn scan_day_anchor(
    words: &[&str],
    exclude_lo: usize,
    exclude_hi: usize,
) -> (Option<DayTarget>, alloc::vec::Vec<usize>) {
    let mut j = 0;
    while j < words.len() {
        if j >= exclude_lo && j <= exclude_hi {
            j += 1;
            continue;
        }
        let cleaned = words[j].trim_end_matches(',');

        // Two-token "on <weekday>". Check before the single-token
        // weekday branch so "on" doesn't get stuck in the title.
        if cleaned == "on" && j + 1 < words.len() {
            let next_idx = j + 1;
            // Don't cross into the clock phrase with the lookahead.
            if !(next_idx >= exclude_lo && next_idx <= exclude_hi) {
                if let Some(wd) = weekday_from_word(words[next_idx].trim_end_matches(',')) {
                    let mut indices = alloc::vec::Vec::new();
                    indices.push(j);
                    indices.push(next_idx);
                    return (Some(DayTarget::Weekday(wd)), indices);
                }
            }
        }

        if let Some(wd) = weekday_from_word(cleaned) {
            return (Some(DayTarget::Weekday(wd)), alloc::vec![j]);
        }
        match cleaned {
            "tomorrow" => return (Some(DayTarget::Offset(1)), alloc::vec![j]),
            "today" => return (Some(DayTarget::Offset(0)), alloc::vec![j]),
            _ => {}
        }

        j += 1;
    }
    (None, alloc::vec::Vec::new())
}

/// Assemble a clock expression starting at `words[start..]`. Returns
/// `(hour, minute, tokens_consumed)` for the longest shape that fits.
///
/// Recognised shapes (longest match wins):
///   three tokens — `H M am|pm`   (from "nine thirty pm" after normalisation)
///   two tokens   — `H am|pm`     (from "nine pm" after normalisation)
///   two tokens   — `H M`         (from "nine thirty" after normalisation, 24h)
///   one token    — anything [`parse_clock_token`] accepts ("5pm", "9:30", "17:00", "noon")
fn parse_clock_phrase(words: &[&str], start: usize) -> Option<(u8, u8, usize)> {
    if start >= words.len() {
        return None;
    }
    let t0 = words[start].trim_end_matches(',');

    // Three-token: H M am|pm
    if start + 2 < words.len() {
        let t1 = words[start + 1].trim_end_matches(',');
        let t2 = words[start + 2].trim_end_matches(',');
        if matches!(t2, "am" | "pm") {
            if let (Ok(h12), Ok(m)) = (t0.parse::<u8>(), t1.parse::<u8>()) {
                if (1..=12).contains(&h12) && m <= 59 {
                    return Some((apply_ampm(h12, t2 == "pm"), m, 3));
                }
            }
        }
    }

    // Two-token: H am|pm, or bare H M (24h).
    if start + 1 < words.len() {
        let t1 = words[start + 1].trim_end_matches(',');
        if matches!(t1, "am" | "pm") {
            if let Ok(h12) = t0.parse::<u8>() {
                if (1..=12).contains(&h12) {
                    return Some((apply_ampm(h12, t1 == "pm"), 0, 2));
                }
            }
        }
        // Bare H M. Only fires when both tokens are plain integers in
        // the valid clock range — "at 9 tomorrow" doesn't trip it
        // because "tomorrow" isn't a digit. Gate on `t0` not already
        // containing ':' so we don't try to reinterpret "9:30 5" (a
        // pre-joined clock followed by a stray digit).
        if !t0.contains(':') {
            if let (Ok(h), Ok(m)) = (t0.parse::<u8>(), t1.parse::<u8>()) {
                if h <= 23 && m <= 59 {
                    return Some((h, m, 2));
                }
            }
        }
    }

    parse_clock_token(t0).map(|(h, m)| (h, m, 1))
}

fn apply_ampm(h12: u8, is_pm: bool) -> u8 {
    match (h12, is_pm) {
        (12, false) => 0,
        (12, true) => 12,
        (h, false) => h,
        (h, true) => h + 12,
    }
}

/// Match a bare "tomorrow", "today", weekday name, "on <weekday>",
/// or a calendar date form ("on the 27th of april") with no
/// time-of-day → date-only VTODO. Skipped if `at TIME` already
/// consumed a day word (extract_at_time runs first).
fn extract_day_only(payload: &str) -> Option<(When, String)> {
    let words: alloc::vec::Vec<&str> = payload.split_whitespace().collect();

    // Calendar date takes priority over weekday for the same reason it
    // does in extract_at_time: an explicit date is more specific.
    let (cal_match, cal_indices) = scan_calendar_date(&words, 1, 0);
    if let Some((month, day)) = cal_match {
        let when = When::DateOnlyDate { month, day };
        let kept: alloc::vec::Vec<&str> = words
            .iter()
            .enumerate()
            .filter_map(|(idx, w)| if cal_indices.contains(&idx) { None } else { Some(*w) })
            .collect();
        return Some((when, kept.join(" ")));
    }

    // Reuse scan_day_anchor for weekday/today/tomorrow. No "clock
    // phrase" to exclude here — pass an empty inclusive range
    // (lo=1, hi=0) so no token gets skipped.
    let (target, indices) = scan_day_anchor(&words, 1, 0);
    let target = target?;
    let when = match target {
        DayTarget::Offset(off) => When::DateOnly { day_offset: off },
        DayTarget::Weekday(wd) => When::DateOnlyWeekday { weekday: wd },
    };
    let kept: alloc::vec::Vec<&str> = words
        .iter()
        .enumerate()
        .filter_map(|(idx, w)| if indices.contains(&idx) { None } else { Some(*w) })
        .collect();
    Some((when, kept.join(" ")))
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

    // ── Split-token clock forms ────────────────────────────────────────
    // The engine normaliser turns "nine thirty" into "9 30" and
    // "nine thirty pm" into "9 30 pm" before the skill sees the
    // payload. These cases exercise the `parse_clock_phrase` helper.

    #[test]
    fn at_time_split_token_24h() {
        let p = parse("remind me to take out the trash at 9 30");
        assert_eq!(p.title, "take out the trash");
        assert_eq!(
            p.when,
            When::LocalClock { hour: 9, minute: 30, day_offset: 0 },
        );
    }

    #[test]
    fn at_time_split_token_with_pm() {
        let p = parse("remind me to take out the trash at 9 30 pm");
        assert_eq!(p.title, "take out the trash");
        assert_eq!(
            p.when,
            When::LocalClock { hour: 21, minute: 30, day_offset: 0 },
        );
    }

    #[test]
    fn at_time_split_token_with_am() {
        let p = parse("remind me to call the dentist at 9 30 am");
        assert_eq!(p.title, "call the dentist");
        assert_eq!(
            p.when,
            When::LocalClock { hour: 9, minute: 30, day_offset: 0 },
        );
    }

    #[test]
    fn at_time_hour_only_with_pm_split() {
        // "at nine pm" normalises to "at 9 pm" — two tokens after "at".
        let p = parse("remind me to leave at 9 pm");
        assert_eq!(p.title, "leave");
        assert_eq!(
            p.when,
            When::LocalClock { hour: 21, minute: 0, day_offset: 0 },
        );
    }

    #[test]
    fn at_time_split_token_with_tomorrow_anywhere() {
        let p = parse("remind me at 9 30 tomorrow to call mum");
        assert_eq!(p.title, "call mum");
        assert_eq!(
            p.when,
            When::LocalClock { hour: 9, minute: 30, day_offset: 1 },
        );
    }

    #[test]
    fn at_time_split_token_rejects_non_minute_second_token() {
        // "at 9 tomorrow" must NOT consume "tomorrow" as minutes.
        let p = parse("remind me at 9 tomorrow to buy milk");
        assert_eq!(p.title, "buy milk");
        assert_eq!(
            p.when,
            When::LocalClock { hour: 9, minute: 0, day_offset: 1 },
        );
    }

    // ── parse_clock_phrase direct tests ───────────────────────────────

    #[test]
    fn clock_phrase_three_token_pm() {
        assert_eq!(parse_clock_phrase(&["9", "30", "pm"], 0), Some((21, 30, 3)));
    }

    #[test]
    fn clock_phrase_two_token_bare_24h() {
        assert_eq!(parse_clock_phrase(&["17", "45"], 0), Some((17, 45, 2)));
    }

    #[test]
    fn clock_phrase_two_token_hour_plus_ampm() {
        assert_eq!(parse_clock_phrase(&["9", "pm"], 0), Some((21, 0, 2)));
        assert_eq!(parse_clock_phrase(&["12", "am"], 0), Some((0, 0, 2)));
    }

    #[test]
    fn clock_phrase_falls_back_to_single_token() {
        assert_eq!(parse_clock_phrase(&["5pm"], 0), Some((17, 0, 1)));
        assert_eq!(parse_clock_phrase(&["9:30pm"], 0), Some((21, 30, 1)));
        assert_eq!(parse_clock_phrase(&["noon"], 0), Some((12, 0, 1)));
    }

    #[test]
    fn clock_phrase_rejects_out_of_range_minute() {
        // "at 9 60" — 60 isn't a valid minute, don't consume it as one.
        // Falls through to single-token parse of "9" → (9, 0, 1).
        assert_eq!(parse_clock_phrase(&["9", "60"], 0), Some((9, 0, 1)));
    }

    #[test]
    fn clock_phrase_prefers_longest_match() {
        // Three-token beats two-token beats one-token.
        assert_eq!(parse_clock_phrase(&["9", "30", "pm"], 0), Some((21, 30, 3)));
        assert_eq!(parse_clock_phrase(&["9", "30"], 0), Some((9, 30, 2)));
        assert_eq!(parse_clock_phrase(&["9"], 0), Some((9, 0, 1)));
    }

    // ── Named weekdays ────────────────────────────────────────────────
    // Skill emits Weekday variants; Android resolves to the next
    // occurrence in local time. The skill itself can't compute an
    // offset because it doesn't know the host's current weekday.

    #[test]
    fn at_time_on_named_weekday_strips_on_prefix() {
        let p = parse("remind me to wash anu on friday at 3pm");
        assert_eq!(p.title, "wash anu");
        assert_eq!(
            p.when,
            When::LocalClockOnWeekday { hour: 15, minute: 0, weekday: 4 },
        );
    }

    #[test]
    fn at_time_bare_weekday_works() {
        let p = parse("remind me to call mum friday at 11am");
        assert_eq!(p.title, "call mum");
        assert_eq!(
            p.when,
            When::LocalClockOnWeekday { hour: 11, minute: 0, weekday: 4 },
        );
    }

    #[test]
    fn at_time_weekday_can_lead_the_payload() {
        let p = parse("remind me on tuesday at 9am to take my pills");
        assert_eq!(p.title, "take my pills");
        assert_eq!(
            p.when,
            When::LocalClockOnWeekday { hour: 9, minute: 0, weekday: 1 },
        );
    }

    #[test]
    fn day_only_on_weekday_yields_date_only_weekday() {
        let p = parse("remind me on friday to send the report");
        assert_eq!(p.title, "send the report");
        assert_eq!(p.when, When::DateOnlyWeekday { weekday: 4 });
    }

    #[test]
    fn day_only_bare_weekday_still_matches() {
        let p = parse("remind me saturday to water the plants");
        assert_eq!(p.title, "water the plants");
        assert_eq!(p.when, When::DateOnlyWeekday { weekday: 5 });
    }

    #[test]
    fn every_weekday_name_parses() {
        let expectations = [
            ("monday", 0u8),
            ("tuesday", 1),
            ("wednesday", 2),
            ("thursday", 3),
            ("friday", 4),
            ("saturday", 5),
            ("sunday", 6),
        ];
        for (name, expected) in expectations {
            let p = parse(&alloc::format!("remind me on {name} to do a thing"));
            assert_eq!(
                p.when,
                When::DateOnlyWeekday { weekday: expected },
                "weekday name '{name}' did not parse to {expected}",
            );
        }
    }

    #[test]
    fn on_without_weekday_is_not_consumed() {
        // "on the desk" shouldn't eat "on" as a day-anchor prefix —
        // only "on <weekday>" does.
        let p = parse("remind me to put the key on the desk");
        assert_eq!(p.title, "put the key on the desk");
        assert_eq!(p.when, When::None);
    }

    #[test]
    fn tomorrow_still_wins_over_absent_weekday() {
        // Regression guard: the new scanner's first-match behaviour must
        // still produce day_offset=1 for plain "tomorrow".
        let p = parse("remind me tomorrow at 3pm to call the plumber");
        assert_eq!(p.title, "call the plumber");
        assert_eq!(
            p.when,
            When::LocalClock { hour: 15, minute: 0, day_offset: 1 },
        );
    }

    // ── Calendar dates ────────────────────────────────────────────────
    // Skill emits LocalClockOnDate / DateOnlyDate variants; Android
    // resolves (month, day) to the next occurrence in local time
    // (this year unless the date already passed, in which case next
    // year). The four forms below are the ones the skill must accept
    // per the scope Keith signed off on.

    #[test]
    fn date_ordinal_with_of_and_at_time() {
        // "remind me to submit my tax return on the 27th of april at 10am"
        let p = parse("remind me to submit my tax return on the 27th of april at 10am");
        assert_eq!(p.title, "submit my tax return");
        assert_eq!(
            p.when,
            When::LocalClockOnDate { hour: 10, minute: 0, month: 4, day: 27 },
        );
    }

    #[test]
    fn date_ordinal_no_of_and_at_time() {
        // "on the 27th april at 10am"
        let p = parse("remind me to submit my tax return on the 27th april at 10am");
        assert_eq!(p.title, "submit my tax return");
        assert_eq!(
            p.when,
            When::LocalClockOnDate { hour: 10, minute: 0, month: 4, day: 27 },
        );
    }

    #[test]
    fn date_numberwords_with_of_and_at_time() {
        // "twenty seventh of april" — the engine's words_to_number
        // normaliser collapses "twenty seventh" to "27", and the
        // resulting "on the 27 of april" parses identically to the
        // ordinal-suffix form.
        let p = parse("remind me to submit my tax return on the 27 of april at 10am");
        assert_eq!(p.title, "submit my tax return");
        assert_eq!(
            p.when,
            When::LocalClockOnDate { hour: 10, minute: 0, month: 4, day: 27 },
        );
    }

    #[test]
    fn date_numberwords_no_of_and_at_time() {
        // "twenty seventh april" → normaliser → "27 april".
        let p = parse("remind me to submit my tax return on the 27 april at 10am");
        assert_eq!(p.title, "submit my tax return");
        assert_eq!(
            p.when,
            When::LocalClockOnDate { hour: 10, minute: 0, month: 4, day: 27 },
        );
    }

    #[test]
    fn date_without_time_emits_date_only_date() {
        let p = parse("remind me on the 27th of april to submit my tax return");
        assert_eq!(p.title, "submit my tax return");
        assert_eq!(p.when, When::DateOnlyDate { month: 4, day: 27 });
    }

    #[test]
    fn date_leading_order_with_time_still_strips_cleanly() {
        // "on the 27th of april at 10am to X" — date leads, clock
        // follows, "to X" is the title.
        let p = parse("remind me on the 27th of april at 10am to call the accountant");
        assert_eq!(p.title, "call the accountant");
        assert_eq!(
            p.when,
            When::LocalClockOnDate { hour: 10, minute: 0, month: 4, day: 27 },
        );
    }

    #[test]
    fn date_with_ordinal_suffix_one_st() {
        let p = parse("remind me to file the return on the 1st of may at 9am");
        assert_eq!(p.title, "file the return");
        assert_eq!(
            p.when,
            When::LocalClockOnDate { hour: 9, minute: 0, month: 5, day: 1 },
        );
    }

    #[test]
    fn date_rejects_impossible_day() {
        // 32 isn't a valid day-of-month. parse_day_ordinal returns
        // None, so the scanner skips the token and this utterance
        // falls through to LocalClock today at 9am with the stray
        // "32 of may" staying in the title. Not ideal, but honest:
        // the parser has no opinion on "that day can't exist".
        let p = parse("remind me to nothing on the 32 of may at 9am");
        assert!(
            matches!(p.when, When::LocalClock { hour: 9, minute: 0, day_offset: 0 }),
            "got {:?}",
            p.when,
        );
    }

    // ── Parse confidence (Layer A) ────────────────────────────────────
    // The skill self-reports how sure it is about the extracted
    // parameters. Clean parses are High; residue looking like a
    // missed anchor drops to Partial (day/time extracted but
    // something in the title suggests the user said more) or Low
    // (when fell back to the no-anchor default, so the residue is
    // probably the real intent).

    #[test]
    fn confidence_high_when_title_has_no_residue() {
        let p = parse("remind me to walk the dog at 5pm");
        assert_eq!(p.confidence, Confidence::High);
        assert_eq!(p.unparsed, None);
    }

    #[test]
    fn confidence_high_for_weekday_clean_parse() {
        let p = parse("remind me to call mum at 9am on friday");
        assert_eq!(p.confidence, Confidence::High);
        assert_eq!(p.unparsed, None);
    }

    #[test]
    fn confidence_partial_when_next_qualifier_remains() {
        // "next tuesday" — the scanner consumes "tuesday" as the
        // weekday anchor, but "next" stays in the title because the
        // parser doesn't understand the next/this/last qualifier.
        // Confidence drops to Partial (we did get a weekday anchor),
        // and unparsed="next" tells the frontend what was missed so
        // the user can be warned.
        let p = parse("remind me next tuesday at 9am to see the dentist");
        assert_eq!(p.confidence, Confidence::Partial, "title={:?}", p.title);
        assert_eq!(p.unparsed.as_deref(), Some("next"));
    }

    #[test]
    fn confidence_low_when_no_anchor_matched_at_all() {
        // "christmas" isn't a recognised anchor of any kind. When
        // falls back to LocalClock(9,0,0) (pure "today at 9am"
        // default) and the residue "christmas" is the user's actual
        // intent. Confidence should be Low.
        //
        // Note: "christmas" doesn't trip is_datetime_residue today
        // (no holiday table yet), so the residue is actually just
        // "tomorrow" — included to show the "fell back" branch fires.
        let p = parse("remind me tomorrow to buy christmas presents");
        // Today+tomorrow is a clean parse; use a case that actually
        // leaves residue:
        let p2 = parse("remind me on tuesday the 40th to do a thing");
        // "tuesday" is consumed as weekday; "40th" is not a valid
        // ordinal date. When is LocalClockOnWeekday, which is NOT a
        // fallback — so confidence is Partial. That's the right
        // behaviour even though "40th" is nonsense.
        assert_eq!(p2.confidence, Confidence::Partial);
        assert_eq!(p2.unparsed.as_deref(), Some("40th"));
        // Belt-and-braces: demonstrate Low separately via a pure
        // fallback case.
        let p3 = parse("remind me to do a thing tonight");
        // "tonight" isn't an anchor (no hour/minute mapping today).
        // When is None. Residue is "tonight" → Low.
        assert_eq!(p3.confidence, Confidence::Low);
        assert_eq!(p3.unparsed.as_deref(), Some("tonight"));
        // Quiet unused warning on p.
        let _ = p;
    }

    #[test]
    fn confidence_low_when_month_only_and_no_day() {
        // "in april" — parser has no anchor for "april" alone (needs a
        // day). Falls back, residue preserved.
        let p = parse("remind me in april to book the flight");
        assert_eq!(p.confidence, Confidence::Low);
        assert_eq!(p.unparsed.as_deref(), Some("april"));
    }

    #[test]
    fn confidence_low_when_ordinal_without_month_leaks_to_title() {
        // "on the 27th" (no month) — scan_calendar_date requires both
        // day and month, so nothing is consumed. Residue "27th"
        // survives in the title.
        let p = parse("remind me on the 27th at 10am to submit the form");
        assert_eq!(p.confidence, Confidence::Low, "title={:?}", p.title);
        assert_eq!(p.unparsed.as_deref(), Some("27th"));
    }

    #[test]
    fn confidence_high_for_calendar_date_clean_parse() {
        let p = parse("remind me to submit my tax return on the 27th of april at 10am");
        assert_eq!(p.confidence, Confidence::High);
        assert_eq!(p.unparsed, None);
    }

    #[test]
    fn confidence_high_when_plain_number_is_not_ordinal() {
        // "buy 3 apples" — bare digit, not a date ordinal. Must not
        // be mistaken for residue.
        let p = parse("remind me to buy 3 apples at 5pm");
        assert_eq!(p.confidence, Confidence::High, "title={:?}", p.title);
        assert_eq!(p.unparsed, None);
    }

    #[test]
    fn confidence_string_form_maps_to_envelope_values() {
        assert_eq!(Confidence::High.as_envelope_str(), "high");
        assert_eq!(Confidence::Partial.as_envelope_str(), "partial");
        assert_eq!(Confidence::Low.as_envelope_str(), "low");
    }

    #[test]
    fn calendar_date_beats_weekday() {
        // If an utterance contains both "friday" and "the 27th of
        // april", the explicit calendar date wins. Loose utterances
        // like "on friday the 27th of april" should still resolve to
        // the date.
        let p = parse("remind me on friday the 27th of april at 10am to do a thing");
        assert_eq!(p.title, "do a thing");
        // "friday" gets left in the title or stripped — we don't
        // assert its fate, only that the calendar date wins over the
        // weekday for the `when` field.
        assert_eq!(
            p.when,
            When::LocalClockOnDate { hour: 10, minute: 0, month: 4, day: 27 },
        );
    }
}
