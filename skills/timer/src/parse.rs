//! Utterance parsing.
//!
//! The engine normalises input before it reaches us (lowercase, contractions
//! expanded, number words → digits, punctuation stripped). So by the time
//! parsing starts we're working with clean space-separated ASCII-ish text.
//!
//! The guiding principle: **parse durations first, then name from the
//! residual**. That way the adjective form ("set a 4 minute pasta timer")
//! and the prepositional form ("set a pasta timer for 4 minutes") both
//! collapse to the same shape once the duration span is stripped out.

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Classified intent for one utterance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    /// One or more create requests. Each tuple is `(name, duration_ms)`.
    /// `name` is `None` for anonymous timers.
    Create(Vec<(Option<String>, u64)>),
    /// Query remaining time. `None` = the only active timer (if there's
    /// exactly one) or a request-for-disambiguation.
    Query(Option<String>),
    /// Cancel one named timer (or the only one, if name is None and exactly
    /// one timer exists).
    Cancel(Option<String>),
    /// Cancel every active timer.
    CancelAll,
    /// List every active timer.
    List,
    /// Couldn't parse a usable request from the utterance.
    Unintelligible,
}

/// Stopwords that are never timer names. `for` is on here so "timer for 5"
/// doesn't extract "for" as a name.
const NAME_STOPWORDS: &[&str] = &[
    "a", "an", "the", "my", "another", "new", "for", "of", "please", "me",
];

/// Words that unambiguously signal "cancel every timer" when combined with
/// a cancel verb. `all`, `every`, `all of` → cancel_all.
fn is_cancel_all_qualifier(tok: &str) -> bool {
    matches!(tok, "all" | "every" | "everything")
}

pub fn classify(input: &str) -> Intent {
    let input = input.trim();

    // Easier to reason about with pre-split tokens. We still keep the raw
    // string around for regex-ish searches below.
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.is_empty() {
        return Intent::Unintelligible;
    }

    if has_list_phrasing(&tokens) {
        return Intent::List;
    }

    if let Some(verb_idx) = find_cancel_verb(&tokens) {
        let rest = &tokens[verb_idx + 1..];
        if rest.iter().any(|t| is_cancel_all_qualifier(t)) {
            return Intent::CancelAll;
        }
        return Intent::Cancel(extract_name(rest));
    }

    if has_query_phrasing(&tokens) {
        return Intent::Query(extract_name(&tokens));
    }

    if has_create_verb(&tokens) {
        return Intent::Create(parse_create_segments(input));
    }

    Intent::Unintelligible
}

fn has_list_phrasing(tokens: &[&str]) -> bool {
    // "what timers do i have", "list my timers", "show my timers"
    let lower = tokens.join(" ");
    lower.contains("what timers")
        || lower.contains("list")
        || (lower.contains("show") && lower.contains("timer"))
        || lower.contains("timers do i")
}

fn find_cancel_verb(tokens: &[&str]) -> Option<usize> {
    tokens.iter().position(|t| {
        matches!(*t, "cancel" | "stop" | "remove" | "delete" | "clear" | "kill")
    })
}

fn has_query_phrasing(tokens: &[&str]) -> bool {
    let lower = tokens.join(" ");
    lower.contains("how much")
        || lower.contains("how long")
        || lower.contains("time left")
        || lower.contains("time remaining")
        || lower.contains("how many")
}

fn has_create_verb(tokens: &[&str]) -> bool {
    tokens
        .iter()
        .any(|t| matches!(*t, "set" | "start" | "create" | "add" | "make"))
}

/// Pull a timer name out of a token slice. Strategy: find the word "timer"
/// and take the token immediately before it; reject if it's a stopword.
/// Also tolerates "pasta timer" and "timer pasta" and bare "pasta".
pub fn extract_name(tokens: &[&str]) -> Option<String> {
    // First preference: `<word> timer`.
    if let Some(pos) = tokens.iter().position(|t| *t == "timer" || *t == "timers") {
        if pos > 0 {
            let candidate = tokens[pos - 1];
            if !is_stopword(candidate) && !looks_like_number(candidate) && !looks_like_unit(candidate) {
                return Some(candidate.to_string());
            }
        }
    }

    // Second preference: first non-stopword, non-number, non-unit token
    // after "on"/"my" that isn't "timer" itself. This catches phrasings
    // like "how much time is left on pasta" where "timer" may be absent.
    let mut after_marker = false;
    for tok in tokens {
        if matches!(*tok, "on" | "my") {
            after_marker = true;
            continue;
        }
        if !after_marker {
            continue;
        }
        if *tok == "timer" || *tok == "timers" {
            continue;
        }
        if is_stopword(tok) || looks_like_number(tok) || looks_like_unit(tok) {
            continue;
        }
        return Some((*tok).to_string());
    }

    None
}

fn is_stopword(tok: &str) -> bool {
    NAME_STOPWORDS.iter().any(|sw| *sw == tok)
}

fn looks_like_number(tok: &str) -> bool {
    !tok.is_empty() && tok.chars().all(|c| c.is_ascii_digit())
}

fn looks_like_unit(tok: &str) -> bool {
    matches!(
        tok,
        "hour"
            | "hours"
            | "hr"
            | "hrs"
            | "h"
            | "minute"
            | "minutes"
            | "min"
            | "mins"
            | "m"
            | "second"
            | "seconds"
            | "sec"
            | "secs"
            | "s"
    )
}

/// Split a multi-create utterance on "and" / "and another" into separate
/// segments, then parse each one into `(name, duration_ms)`.
///
/// We keep the splitter conservative: only split on "and" that sits between
/// two recognisable duration phrases, so "pasta and egg timer for 5 minutes"
/// stays one segment. In practice the multi-create pattern is always
/// "<segment> and [another] <duration>".
pub fn parse_create_segments(input: &str) -> Vec<(Option<String>, u64)> {
    let mut segments = Vec::new();
    for seg in split_segments(input) {
        if let Some((name, ms)) = parse_single_segment(&seg) {
            segments.push((name, ms));
        }
    }
    segments
}

fn split_segments(input: &str) -> Vec<String> {
    // Splitter philosophy: compound durations inside a single segment must
    // stay together. Only split when the user clearly means a second timer:
    //   - "... and another ..."  → explicit marker
    //   - "... and <a|an|for|make|set|...>"  → fresh create-verb phrase after
    //     "and" implies a new segment.
    // A bare "<unit> and <num> <unit>" (e.g. "1 hour and 30 minutes")
    // falls through as a compound duration and is NOT split.

    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut segments: Vec<String> = Vec::new();
    let mut current_start = 0usize;

    let mut i = 0;
    while i < tokens.len() {
        if tokens[i] == "and" && i + 1 < tokens.len() {
            let next = tokens[i + 1];
            let skip_after_and = next == "another";
            let is_fresh_phrase = matches!(
                next,
                "another" | "a" | "an" | "for" | "set" | "start" | "create" | "add" | "make"
            );
            if is_fresh_phrase {
                let segment = tokens[current_start..i].join(" ");
                if !segment.is_empty() {
                    segments.push(segment);
                }
                current_start = if skip_after_and { i + 2 } else { i + 1 };
                i = current_start;
                continue;
            }
        }
        i += 1;
    }

    let tail = tokens[current_start..].join(" ");
    if !tail.is_empty() {
        segments.push(tail);
    }
    segments
}

/// Parse one segment. Extracts every "<digits> <unit>" span, sums them into
/// a single duration, strips the matched spans, then looks for a name in
/// the residual.
fn parse_single_segment(input: &str) -> Option<(Option<String>, u64)> {
    let tokens: Vec<&str> = input.split_whitespace().collect();

    let mut total_ms: u64 = 0;
    let mut consumed: Vec<bool> = vec![false; tokens.len()];
    let mut i = 0;
    while i + 1 < tokens.len() {
        if looks_like_number(tokens[i]) && looks_like_unit(tokens[i + 1]) {
            let n: u64 = tokens[i].parse().unwrap_or(0);
            if let Some(ms) = unit_to_ms(tokens[i + 1], n) {
                total_ms = total_ms.saturating_add(ms);
                consumed[i] = true;
                consumed[i + 1] = true;
                i += 2;
                continue;
            }
        }
        i += 1;
    }

    if total_ms == 0 {
        return None;
    }

    let residual: Vec<&str> = tokens
        .iter()
        .enumerate()
        .filter_map(|(idx, t)| if consumed[idx] { None } else { Some(*t) })
        .collect();

    Some((extract_name(&residual), total_ms))
}

fn unit_to_ms(unit: &str, n: u64) -> Option<u64> {
    let per = match unit {
        "hour" | "hours" | "hr" | "hrs" | "h" => 3_600_000,
        "minute" | "minutes" | "min" | "mins" | "m" => 60_000,
        "second" | "seconds" | "sec" | "secs" | "s" => 1_000,
        _ => return None,
    };
    Some(n.saturating_mul(per))
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;

    fn create(input: &str) -> Vec<(Option<String>, u64)> {
        match classify(input) {
            Intent::Create(v) => v,
            other => panic!("expected Create, got {other:?}"),
        }
    }

    #[test]
    fn plain_anonymous_timer() {
        assert_eq!(create("set a timer for 10 minutes"), vec![(None, 600_000)]);
    }

    #[test]
    fn named_timer_prepositional() {
        assert_eq!(
            create("set a pasta timer for 8 minutes"),
            vec![(Some("pasta".into()), 480_000)]
        );
    }

    #[test]
    fn named_timer_adjective_form() {
        // "set a 4 minute pasta timer" — duration word sits INSIDE the name
        // span. The parser must strip "4 minute" first, leaving "pasta timer"
        // from which "pasta" falls out as the name.
        assert_eq!(
            create("set a 4 minute pasta timer"),
            vec![(Some("pasta".into()), 240_000)]
        );
    }

    #[test]
    fn adjective_form_with_seconds() {
        assert_eq!(
            create("set a 30 second egg timer"),
            vec![(Some("egg".into()), 30_000)]
        );
    }

    #[test]
    fn anonymous_with_an_article() {
        assert_eq!(create("set an 8 minute timer"), vec![(None, 480_000)]);
    }

    #[test]
    fn compound_duration() {
        // 1 hour and 30 minutes — single segment, summed duration.
        assert_eq!(
            create("set a timer for 1 hour and 30 minutes"),
            vec![(None, 5_400_000)]
        );
    }

    #[test]
    fn multi_create_and_another() {
        assert_eq!(
            create("set a timer for 5 minutes and another for 15 minutes"),
            vec![(None, 300_000), (None, 900_000)]
        );
    }

    #[test]
    fn multi_create_named_and_anonymous() {
        assert_eq!(
            create("set a pasta timer for 5 minutes and another for 15 minutes"),
            vec![(Some("pasta".into()), 300_000), (None, 900_000)]
        );
    }

    #[test]
    fn query_with_name() {
        match classify("how much time is left on my pasta timer") {
            Intent::Query(Some(n)) => assert_eq!(n, "pasta"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn cancel_with_name() {
        match classify("cancel my pasta timer") {
            Intent::Cancel(Some(n)) => assert_eq!(n, "pasta"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn cancel_all() {
        assert_eq!(classify("cancel all timers"), Intent::CancelAll);
        assert_eq!(classify("stop every timer"), Intent::CancelAll);
    }

    #[test]
    fn list() {
        assert_eq!(classify("what timers do i have"), Intent::List);
        assert_eq!(classify("list my timers"), Intent::List);
    }
}
