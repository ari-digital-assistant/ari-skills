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

use alloc::format;
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
    "il", "lo", "la", "i", "gli", "le", "l", "un", "uno", "una", "mio", "mia",
    "miei", "mie", "tuo", "tua", "altro", "nuovo", "di", "da", "tra", "fra",
    "in", "più", "piu", "adesso", "subito", "favore", "grazie",
];

/// Italian prepositions that introduce a timer's name: "per la pasta",
/// "il timer della pasta", "quanto manca alla pasta".
const IT_NAME_MARKERS: &[&str] = &[
    "per", "del", "dello", "della", "dei", "degli", "delle", "al", "allo", "alla",
    "ai", "agli", "alle",
];

/// The parser reads English and Italian together, like the alarm and
/// reminder parsers: the two vocabularies don't collide, so nothing needs
/// the locale. The Italian verbs are kept apart only because
/// [`extract_name`] uses them to recognise an Italian phrase.
const EN_CREATE_VERBS: &[&str] = &["set", "start", "create", "add", "make"];
const IT_CREATE_VERBS: &[&str] = &[
    "imposta", "avvia", "metti", "mettimi", "crea", "fai", "fammi", "dammi",
    "avvisami", "suona", "parti", "aggiungi",
];
const EN_CANCEL_VERBS: &[&str] = &["cancel", "stop", "remove", "delete", "clear", "kill"];
const IT_CANCEL_VERBS: &[&str] = &[
    "annulla", "cancella", "elimina", "rimuovi", "togli", "ferma", "spegni",
    "disattiva",
];

/// Words that unambiguously signal "cancel every timer" when combined with
/// a cancel verb. `all`, `every`, `all of` → cancel_all.
fn is_cancel_all_qualifier(tok: &str) -> bool {
    matches!(tok, "all" | "every" | "everything" | "tutti" | "tutto")
}

pub fn classify(input: &str) -> Intent {
    let input = expand_italian_idioms(input.trim());
    let input = input.as_str();

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

    // No verb, but a timer and a duration ("timer 5 minuti", "a 5 minute
    // timer") can only mean one thing.
    if tokens.iter().any(|t| *t == "timer") {
        let segments = parse_create_segments(input);
        if !segments.is_empty() {
            return Intent::Create(segments);
        }
    }

    Intent::Unintelligible
}

/// Rewrite the Italian duration idioms, and the "conto alla rovescia"
/// synonym for a timer, into shapes the rest of the parser already reads.
/// The engine has split the elisions by now, so "un quarto d'ora" arrives
/// as "un quarto d ora".
fn expand_italian_idioms(input: &str) -> String {
    const IDIOMS: &[(&str, &str)] = &[
        (" conto alla rovescia ", " timer "),
        (" tre quarti d ora ", " 45 minuti "),
        (" un quarto d ora ", " 15 minuti "),
        (" quarto d ora ", " 15 minuti "),
        (" mezz ora ", " 30 minuti "),
        (" mezzora ", " 30 minuti "),
        (" mezza ora ", " 30 minuti "),
        (" ora e mezza ", " ora e 30 minuti "),
        (" ore e mezza ", " ore e 30 minuti "),
    ];
    let mut padded = format!(" {input} ");
    for (from, to) in IDIOMS {
        padded = padded.replace(from, to);
    }
    padded.trim().to_string()
}

fn has_list_phrasing(tokens: &[&str]) -> bool {
    // "what timers do i have", "list my timers", "show my timers",
    // "quali timer ho", "elenca i miei timer", "mostrami i timer attivi"
    let lower = tokens.join(" ");
    lower.contains("what timers")
        || lower.contains("list")
        || (lower.contains("show") && lower.contains("timer"))
        || lower.contains("timers do i")
        || lower.contains("elenca")
        || (lower.contains("mostra") && lower.contains("timer"))
        || lower.contains("quali timer")
        || lower.contains("che timer")
        || lower.contains("quanti timer")
}

fn find_cancel_verb(tokens: &[&str]) -> Option<usize> {
    tokens.iter().enumerate().position(|(i, t)| {
        EN_CANCEL_VERBS.contains(t)
            || IT_CANCEL_VERBS.contains(t)
            // "non mi serve più il timer della pasta"
            || (*t == "serve" && i >= 2 && tokens[i - 2] == "non" && tokens[i - 1] == "mi")
    })
}

fn has_query_phrasing(tokens: &[&str]) -> bool {
    let lower = tokens.join(" ");
    lower.contains("how much")
        || lower.contains("how long")
        || lower.contains("time left")
        || lower.contains("time remaining")
        || lower.contains("how many")
        || lower.contains("ancora molto")
        || tokens.iter().any(|t| {
            matches!(
                *t,
                "quanto" | "manca" | "mancano" | "resta" | "restano" | "rimane" | "rimangono"
            )
        })
}

fn has_create_verb(tokens: &[&str]) -> bool {
    tokens
        .iter()
        .any(|t| EN_CREATE_VERBS.contains(t) || IT_CREATE_VERBS.contains(t))
}

fn is_verb(tok: &str) -> bool {
    [EN_CREATE_VERBS, IT_CREATE_VERBS, EN_CANCEL_VERBS, IT_CANCEL_VERBS]
        .iter()
        .any(|verbs| verbs.contains(&tok))
}

/// Could this token be a timer's name? Rules out the words that sit around
/// one: articles, numbers, units, verbs and the Italian name prepositions.
fn is_name(tok: &str) -> bool {
    !is_stopword(tok)
        && !looks_like_number(tok)
        && !looks_like_unit(tok)
        && !is_verb(tok)
        && !IT_NAME_MARKERS.contains(&tok)
}

/// Pull a timer name out of a token slice. Strategy: find the word "timer"
/// and take the token immediately before it; reject if it's a stopword.
/// Also tolerates "pasta timer" and "timer pasta" and bare "pasta".
pub fn extract_name(tokens: &[&str]) -> Option<String> {
    let timer_pos = tokens.iter().position(|t| *t == "timer" || *t == "timers");

    // First preference: `<word> timer`.
    if let Some(pos) = timer_pos {
        if pos > 0 {
            let candidate = tokens[pos - 1];
            if is_name(candidate) {
                return Some(candidate.to_string());
            }
        }
    }

    // Second preference: first name-like token after "on"/"my" or an
    // Italian name preposition that isn't "timer" itself. This catches
    // phrasings like "how much time is left on pasta" where "timer" may be
    // absent, and "imposta un timer per la pasta".
    let mut after_marker = false;
    for tok in tokens {
        if matches!(*tok, "on" | "my") || IT_NAME_MARKERS.contains(tok) {
            after_marker = true;
            continue;
        }
        if !after_marker {
            continue;
        }
        if *tok == "timer" || *tok == "timers" {
            continue;
        }
        if !is_name(tok) {
            continue;
        }
        return Some((*tok).to_string());
    }

    // Third preference: Italian puts the name after the noun, "il timer
    // pasta", which is also how the skill's own Italian strings name a timer
    // ("annulla il mio timer {name}"). Read only inside an Italian phrase:
    // English never names a timer that way, and "stop the timer now" must
    // not find a timer called "now".
    if let Some(pos) = timer_pos {
        let italian = pos > 0
            && (matches!(tokens[pos - 1], "il" | "un" | "mio" | "tuo" | "quel" | "questo")
                || IT_CREATE_VERBS.contains(&tokens[pos - 1])
                || IT_CANCEL_VERBS.contains(&tokens[pos - 1]));
        if let Some(next) = tokens.get(pos + 1) {
            if italian && is_name(next) {
                return Some((*next).to_string());
            }
        }
    }

    None
}

fn is_stopword(tok: &str) -> bool {
    NAME_STOPWORDS.iter().any(|sw| *sw == tok)
}

fn looks_like_number(tok: &str) -> bool {
    number(tok).is_some()
}

/// A count from a digit token ("5") or an Italian number word ("cinque").
/// English number words never reach us: the engine turns them into digits,
/// but it leaves Italian ones alone.
fn number(tok: &str) -> Option<u64> {
    if !tok.is_empty() && tok.chars().all(|c| c.is_ascii_digit()) {
        return tok.parse().ok();
    }
    it_num(tok)
}

/// Italian cardinal 0–99. The tens drop their final vowel before uno and
/// otto ("ventuno", "trentotto"), and 23, 33… are written with an accent
/// ("ventitré") the tables don't carry.
fn it_num(tok: &str) -> Option<u64> {
    const UNITS: [&str; 20] = [
        "zero", "uno", "due", "tre", "quattro", "cinque", "sei", "sette", "otto",
        "nove", "dieci", "undici", "dodici", "tredici", "quattordici", "quindici",
        "sedici", "diciassette", "diciotto", "diciannove",
    ];
    const TENS: [&str; 8] = [
        "venti", "trenta", "quaranta", "cinquanta", "sessanta", "settanta",
        "ottanta", "novanta",
    ];
    let tok = tok.replace('é', "e");
    if let Some(n) = UNITS.iter().position(|w| *w == tok) {
        return Some(n as u64);
    }
    for (i, tens) in TENS.iter().enumerate() {
        let value = 20 + 10 * i as u64;
        if tok == *tens {
            return Some(value);
        }
        let unit = match tok.strip_prefix(tens) {
            Some(unit) => unit,
            None => match tok.strip_prefix(&tens[..tens.len() - 1]) {
                Some(unit) if unit == "uno" || unit == "otto" => unit,
                _ => continue,
            },
        };
        if let Some(n) = UNITS[1..10].iter().position(|w| *w == unit) {
            return Some(value + n as u64 + 1);
        }
    }
    None
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
            | "ora"
            | "ore"
            | "minuto"
            | "minuti"
            | "secondo"
            | "secondi"
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
        // Italian: "... e un altro di 15 minuti", "... e uno di 15 minuti",
        // "... e per 15 minuti". "1 ora e 30 minuti" stays one segment.
        if tokens[i] == "e" && i + 1 < tokens.len() {
            let skip = match &tokens[i + 1..] {
                ["un", "altro", ..] => 3,
                ["altro", ..] => 2,
                [next, ..] if *next == "uno" || *next == "per" || IT_CREATE_VERBS.contains(next) => 1,
                _ => 0,
            };
            if skip > 0 {
                let segment = tokens[current_start..i].join(" ");
                if !segment.is_empty() {
                    segments.push(segment);
                }
                current_start = i + skip;
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
        // Italian "un"/"una" is a count only in front of a unit ("un
        // minuto"), and "un secondo timer" is a second timer, not one second.
        let n = number(tokens[i]).or(matches!(tokens[i], "un" | "una").then_some(1));
        let ordinal = tokens[i + 1] == "secondo" && tokens.get(i + 2) == Some(&"timer");
        if let (Some(n), false) = (n, ordinal) {
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
        "hour" | "hours" | "hr" | "hrs" | "h" | "ora" | "ore" => 3_600_000,
        "minute" | "minutes" | "min" | "mins" | "m" | "minuto" | "minuti" => 60_000,
        "second" | "seconds" | "sec" | "secs" | "s" | "secondo" | "secondi" => 1_000,
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

    #[test]
    fn a_verb_is_never_a_name() {
        assert_eq!(create("set timer for 5 minutes"), vec![(None, 300_000)]);
    }

    #[test]
    fn a_word_after_an_english_timer_is_not_a_name() {
        assert_eq!(classify("stop the timer now"), Intent::Cancel(None));
    }

    #[test]
    fn a_bare_duration_timer_creates() {
        assert_eq!(create("a 5 minute timer"), vec![(None, 300_000)]);
    }

    // --- Italian, as the engine hands it over: lowercased, punctuation and
    //     sentence stop gone, elisions split, number words left as words ---

    #[test]
    fn it_the_utterances_from_issue_35() {
        assert_eq!(create("imposta un timer per 5 minuti"), vec![(None, 300_000)]);
        assert_eq!(create("imposta un timer per cinque minuti"), vec![(None, 300_000)]);
        assert_eq!(create("imposta timer 5 minuti"), vec![(None, 300_000)]);
        assert_eq!(create("timer 5 minuti"), vec![(None, 300_000)]);
    }

    #[test]
    fn it_create_verbs() {
        assert_eq!(create("avvia un timer di 4 minuti"), vec![(None, 240_000)]);
        assert_eq!(create("metti un timer di 10 minuti"), vec![(None, 600_000)]);
        assert_eq!(create("crea un timer da 3 minuti"), vec![(None, 180_000)]);
        assert_eq!(create("fai partire un timer di 2 minuti"), vec![(None, 120_000)]);
        assert_eq!(create("parti con un timer di 6 minuti"), vec![(None, 360_000)]);
        assert_eq!(create("imposta un timer di 30 secondi"), vec![(None, 30_000)]);
        assert_eq!(create("metti 7 minuti"), vec![(None, 420_000)]);
        assert_eq!(create("avvisami tra dieci minuti"), vec![(None, 600_000)]);
        assert_eq!(create("suona tra 20 minuti"), vec![(None, 1_200_000)]);
        assert_eq!(
            create("fammi sapere quando sono passati venti minuti"),
            vec![(None, 1_200_000)]
        );
    }

    #[test]
    fn it_named_timers() {
        let pasta = |ms| vec![(Some(String::from("pasta")), ms)];
        assert_eq!(create("imposta un timer per la pasta di 8 minuti"), pasta(480_000));
        assert_eq!(create("avvia un timer di 4 minuti per la pasta"), pasta(240_000));
        assert_eq!(create("imposta un timer per pasta da 5 minuti"), pasta(300_000));
        assert_eq!(create("mettimi otto minuti per la pasta"), pasta(480_000));
        assert_eq!(create("dammi 9 minuti per pasta"), pasta(540_000));
        assert_eq!(create("avvisami tra 10 minuti per la pasta"), pasta(600_000));
        assert_eq!(create("un timer da 5 minuti per la pasta"), pasta(300_000));
        assert_eq!(
            create("avvia il conto alla rovescia di 12 minuti per la pasta"),
            pasta(720_000)
        );
    }

    #[test]
    fn it_number_words() {
        assert_eq!(create("imposta un timer di ventuno minuti"), vec![(None, 1_260_000)]);
        assert_eq!(create("imposta un timer di trentotto minuti"), vec![(None, 2_280_000)]);
        assert_eq!(create("imposta un timer di ventitré minuti"), vec![(None, 1_380_000)]);
        assert_eq!(create("imposta un timer di novanta minuti"), vec![(None, 5_400_000)]);
        assert_eq!(create("imposta un timer di un minuto"), vec![(None, 60_000)]);
        assert_eq!(create("imposta un timer di due ore"), vec![(None, 7_200_000)]);
    }

    #[test]
    fn it_idioms() {
        assert_eq!(create("suona tra un quarto d ora"), vec![(None, 900_000)]);
        assert_eq!(create("imposta un timer di tre quarti d ora"), vec![(None, 2_700_000)]);
        assert_eq!(create("imposta un timer di mezz ora"), vec![(None, 1_800_000)]);
        assert_eq!(create("imposta un timer di un ora"), vec![(None, 3_600_000)]);
        assert_eq!(create("imposta un timer di un ora e mezza"), vec![(None, 5_400_000)]);
        assert_eq!(create("conto alla rovescia di 5 minuti"), vec![(None, 300_000)]);
    }

    #[test]
    fn it_compound_duration_is_one_timer() {
        assert_eq!(
            create("imposta un timer di 1 ora e 30 minuti"),
            vec![(None, 5_400_000)]
        );
    }

    #[test]
    fn it_a_second_timer_is_not_one_second() {
        assert_eq!(
            create("imposta un secondo timer per la pizza di 10 minuti"),
            vec![(Some(String::from("pizza")), 600_000)]
        );
    }

    #[test]
    fn it_multi_create() {
        assert_eq!(
            create("imposta un timer per 5 minuti e un altro per 15 minuti"),
            vec![(None, 300_000), (None, 900_000)]
        );
        assert_eq!(
            create("metti un timer di 5 minuti e uno di 15 minuti"),
            vec![(None, 300_000), (None, 900_000)]
        );
    }

    #[test]
    fn it_set_without_duration_asks_for_one() {
        assert_eq!(create("imposta il timer per la pasta"), vec![]);
    }

    #[test]
    fn it_query() {
        let pasta = Intent::Query(Some(String::from("pasta")));
        assert_eq!(classify("quanto manca al timer per pasta"), pasta);
        assert_eq!(classify("quanto manca al mio timer della pasta"), pasta);
        assert_eq!(classify("quanto resta alla pasta"), pasta);
        assert_eq!(classify("quanti minuti mancano al timer per la pasta"), pasta);
        assert_eq!(classify("ancora molto per la pasta"), pasta);
        assert_eq!(
            classify("ancora quanto per le uova"),
            Intent::Query(Some(String::from("uova")))
        );
        assert_eq!(classify("quanto manca"), Intent::Query(None));
    }

    #[test]
    fn it_cancel() {
        let pasta = Intent::Cancel(Some(String::from("pasta")));
        assert_eq!(classify("annulla il mio timer della pasta"), pasta);
        assert_eq!(classify("cancella il timer per la pasta"), pasta);
        assert_eq!(classify("togli il timer per pasta"), pasta);
        assert_eq!(classify("non mi serve più il timer della pasta"), pasta);
        assert_eq!(classify("ferma il timer"), Intent::Cancel(None));
        assert_eq!(classify("annulla il timer di 5 minuti"), Intent::Cancel(None));
        assert_eq!(classify("cancella tutti i timer"), Intent::CancelAll);
    }

    #[test]
    fn it_cancel_button_utterances_round_trip() {
        // What strings/it.json puts on the card's cancel button.
        assert_eq!(
            classify("annulla il mio timer pasta"),
            Intent::Cancel(Some(String::from("pasta")))
        );
        assert_eq!(classify("annulla il mio timer"), Intent::Cancel(None));
    }

    #[test]
    fn it_list() {
        assert_eq!(classify("quali timer ho"), Intent::List);
        assert_eq!(classify("che timer ho attivi"), Intent::List);
        assert_eq!(classify("elenca i miei timer"), Intent::List);
        assert_eq!(classify("mostrami i timer attivi"), Intent::List);
        assert_eq!(classify("quanti timer ho attivi"), Intent::List);
    }
}
