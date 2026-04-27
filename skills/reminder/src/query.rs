//! Read-only query path for the reminder skill.
//!
//! Handles utterances like:
//! - "what reminders do I have today?"
//! - "what reminders do I have tomorrow?"
//! - "what's my next reminder?"
//! - "any reminders today?"
//!
//! Pure string crunching + a small range-classification enum. The
//! actual host calls (`ari::tasks_query_in_range` etc.) and envelope
//! assembly live in `lib.rs`; this module's job is just to identify
//! whether an input is a query and, if so, what time window the user
//! meant.

use alloc::string::String;
use alloc::string::ToString;

/// Time scope the user asked about. Resolved into a concrete
/// `[start_ms, end_ms)` window by [`Window::resolve`] using the host
/// clock — the parser itself stays clock-naive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Window {
    /// Today, local midnight to next local midnight.
    Today,
    /// Tomorrow, same width as Today shifted +1 day.
    Tomorrow,
    /// "Next reminder" — closest future timed reminder. Treated as a
    /// 7-day window starting now; the caller takes the first row.
    Next,
}

/// Did the user ask a query, and if so which window?
pub fn classify(input: &str) -> Option<Window> {
    let normalised = input.trim().to_lowercase();

    // "Next reminder" wins outright when the phrase is present —
    // catches "what's my next reminder", "what is the next reminder",
    // etc.
    if normalised.contains("next reminder") {
        return Some(Window::Next);
    }

    // Token-test for "today" / "tomorrow" so utterances like "do I
    // have any reminders today?" or "any reminders tomorrow" both
    // resolve. Order matters — check "tomorrow" first so the
    // substring "today" inside "tomorrow's" doesn't false-positive.
    if !is_query_utterance(&normalised) {
        return None;
    }
    if word_present(&normalised, "tomorrow") {
        return Some(Window::Tomorrow);
    }
    if word_present(&normalised, "today") {
        return Some(Window::Today);
    }
    // Defaulting "what reminders do I have" with no day specifier to
    // Today is the principle of least surprise — it's almost always
    // what someone asking that means in context.
    Some(Window::Today)
}

/// Cheap word-boundary check that doesn't pull in regex. The string
/// must contain the needle bracketed by either start/end of string
/// or non-letter characters.
fn word_present(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.is_empty() || bytes.len() < n.len() {
        return false;
    }
    let mut i = 0;
    while i + n.len() <= bytes.len() {
        if &bytes[i..i + n.len()] == n {
            let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphabetic();
            let after_ok =
                i + n.len() == bytes.len() || !bytes[i + n.len()].is_ascii_alphabetic();
            if before_ok && after_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Crude check that the utterance is asking about reminders rather
/// than setting one. The classifier returns `None` for anything that
/// doesn't look like a question; the caller treats `None` as "this
/// isn't a query, try the create path".
fn is_query_utterance(s: &str) -> bool {
    s.contains("what")
        || s.starts_with("any reminders")
        || s.starts_with("do i have")
        || s.starts_with("have i got")
        || s.starts_with("got any")
}

impl Window {
    /// Concrete UTC `[start_ms, end_ms)` for this window, given the
    /// host's current local time + UTC epoch ms + timezone offset.
    /// Day boundaries are local midnights, then converted to UTC by
    /// subtracting the supplied `tz_offset_ms`.
    ///
    /// `civil_to_epoch_ms` is the same helper from `lib.rs`'s civil
    /// date module — passed in as a function pointer so this module
    /// stays self-contained / testable on the host without importing
    /// the wasm-only side.
    pub fn resolve(
        &self,
        year: i32,
        month: u8,
        day: u8,
        tz_offset_ms: i64,
        now_utc_ms: i64,
        civil_to_epoch_ms: fn(i32, u8, u8, u8, u8) -> i64,
    ) -> (i64, i64) {
        match self {
            Window::Today => {
                let start_local = civil_to_epoch_ms(year, month, day, 0, 0);
                let end_local = start_local + 86_400_000;
                (start_local - tz_offset_ms, end_local - tz_offset_ms)
            }
            Window::Tomorrow => {
                let start_local =
                    civil_to_epoch_ms(year, month, day, 0, 0) + 86_400_000;
                let end_local = start_local + 86_400_000;
                (start_local - tz_offset_ms, end_local - tz_offset_ms)
            }
            // "Next" looks 7 days forward from now. A week's enough
            // for the common case ("what's my next reminder?") and
            // bounds the host query so a calendar with thousands of
            // events doesn't spill back. Skill caps the result list
            // to 1 anyway when rendering.
            Window::Next => (now_utc_ms, now_utc_ms + 7 * 86_400_000),
        }
    }

    /// Human label used in the response speak ("today" / "tomorrow"
    /// / "" — the `Next` case doesn't need a label).
    pub fn day_label(&self) -> &'static str {
        match self {
            Window::Today => "today",
            Window::Tomorrow => "tomorrow",
            Window::Next => "",
        }
    }
}

/// Render an HH:MM clock from a UTC epoch ms + the local TZ offset.
/// Returns an `am`/`pm` formatted string. Uses the same conventions
/// as `format_when_phrase` in `lib.rs` so the query response and
/// the create confirmation phrase the same way.
pub fn format_clock_local(epoch_ms: i64, tz_offset_ms: i64, all_day: bool) -> String {
    if all_day {
        return String::from("all day");
    }
    let local_ms = epoch_ms + tz_offset_ms;
    let total_secs = local_ms.div_euclid(1000);
    let secs_of_day = total_secs.rem_euclid(86_400);
    let hour = (secs_of_day / 3600) as u8;
    let minute = ((secs_of_day % 3600) / 60) as u8;
    let (h12, ampm) = if hour == 0 {
        (12, "am")
    } else if hour < 12 {
        (hour, "am")
    } else if hour == 12 {
        (12, "pm")
    } else {
        (hour - 12, "pm")
    };
    if minute == 0 {
        alloc::format!("{}{}", h12, ampm)
    } else {
        alloc::format!("{}:{:02}{}", h12, minute, ampm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_today() {
        assert_eq!(classify("what reminders do I have today?"), Some(Window::Today));
        assert_eq!(classify("any reminders today"), Some(Window::Today));
        assert_eq!(classify("do i have any reminders today"), Some(Window::Today));
    }

    #[test]
    fn classify_tomorrow() {
        assert_eq!(classify("what reminders do I have tomorrow?"), Some(Window::Tomorrow));
        assert_eq!(classify("any reminders tomorrow"), Some(Window::Tomorrow));
    }

    #[test]
    fn classify_next() {
        assert_eq!(classify("what's my next reminder"), Some(Window::Next));
        assert_eq!(classify("what is my next reminder?"), Some(Window::Next));
        assert_eq!(classify("what's the next reminder"), Some(Window::Next));
    }

    #[test]
    fn classify_default_today_when_no_day_specified() {
        // "what reminders do I have" without a time word defaults to today.
        assert_eq!(classify("what reminders do I have"), Some(Window::Today));
    }

    #[test]
    fn classify_rejects_non_queries() {
        assert!(classify("remind me to walk the dog at 5pm").is_none());
        assert!(classify("hello").is_none());
        assert!(classify("").is_none());
    }

    #[test]
    fn word_boundary_rejects_substring() {
        // "today" must be a word, not a substring of "todayish".
        assert!(!word_present("any reminders todayish", "today"));
        assert!(word_present("any reminders today?", "today"));
    }

    #[test]
    fn format_clock_local_handles_morning() {
        // 09:30 UTC + 0 offset = 9:30am
        let ms = 9 * 3_600_000 + 30 * 60_000;
        assert_eq!(format_clock_local(ms, 0, false), "9:30am");
    }

    #[test]
    fn format_clock_local_handles_pm() {
        let ms = 15 * 3_600_000;
        assert_eq!(format_clock_local(ms, 0, false), "3pm");
    }

    #[test]
    fn format_clock_local_handles_midnight() {
        assert_eq!(format_clock_local(0, 0, false), "12am");
    }

    #[test]
    fn format_clock_local_handles_all_day() {
        assert_eq!(format_clock_local(0, 0, true), "all day");
    }

    #[test]
    fn format_clock_local_applies_offset() {
        // 09:00 UTC + 1 hour offset = 10am local
        let ms = 9 * 3_600_000;
        assert_eq!(format_clock_local(ms, 3_600_000, false), "10am");
    }

    fn fake_civil_to_epoch_ms(year: i32, month: u8, day: u8, h: u8, m: u8) -> i64 {
        // Just for the resolve() unit test. Real impl lives in lib.rs.
        // Encodes: epoch is 1970-01-01 00:00, treat passed values as
        // 2026-04-27-relative to fit the test below.
        let _ = (year, month, day);
        h as i64 * 3_600_000 + m as i64 * 60_000
    }

    #[test]
    fn today_window_is_24h_starting_at_local_midnight_utc() {
        // tz_offset = 0, day = (any) → today is [0, 86_400_000_000_i64].
        // Use a real-ish civil_to_epoch_ms via the test stub.
        let (start, end) = Window::Today.resolve(2026, 4, 27, 0, 0, fake_civil_to_epoch_ms);
        assert_eq!(end - start, 86_400_000);
    }

    #[test]
    fn tomorrow_window_is_one_day_after_today() {
        let (today_start, _) =
            Window::Today.resolve(2026, 4, 27, 0, 0, fake_civil_to_epoch_ms);
        let (tomorrow_start, _) =
            Window::Tomorrow.resolve(2026, 4, 27, 0, 0, fake_civil_to_epoch_ms);
        assert_eq!(tomorrow_start - today_start, 86_400_000);
    }

    #[test]
    fn next_window_is_seven_days_from_now() {
        let now = 1_000_000_i64;
        let (start, end) = Window::Next.resolve(2026, 4, 27, 0, now, fake_civil_to_epoch_ms);
        assert_eq!(start, now);
        assert_eq!(end - start, 7 * 86_400_000);
    }
}
