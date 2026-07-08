use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Day { Mon, Tue, Wed, Thu, Fri, Sat, Sun }

impl Day {
    pub fn code(&self) -> &'static str {
        match self {
            Day::Mon => "mon", Day::Tue => "tue", Day::Wed => "wed",
            Day::Thu => "thu", Day::Fri => "fri", Day::Sat => "sat", Day::Sun => "sun",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    Set { hour: u8, minute: u8, message: Option<String>, days: Vec<Day> },
    Show,
    NeedTime,
    Unintelligible,
}

const DAY_NAMES: &[(&str, Day)] = &[
    ("monday", Day::Mon), ("mondays", Day::Mon), ("lunedì", Day::Mon), ("lunedi", Day::Mon),
    ("tuesday", Day::Tue), ("tuesdays", Day::Tue), ("martedì", Day::Tue), ("martedi", Day::Tue),
    ("wednesday", Day::Wed), ("wednesdays", Day::Wed), ("mercoledì", Day::Wed), ("mercoledi", Day::Wed),
    ("thursday", Day::Thu), ("thursdays", Day::Thu), ("giovedì", Day::Thu), ("giovedi", Day::Thu),
    ("friday", Day::Fri), ("fridays", Day::Fri), ("venerdì", Day::Fri), ("venerdi", Day::Fri),
    ("saturday", Day::Sat), ("saturdays", Day::Sat), ("sabato", Day::Sat), ("sabati", Day::Sat),
    ("sunday", Day::Sun), ("sundays", Day::Sun), ("domenica", Day::Sun), ("domeniche", Day::Sun),
];

const WEEKDAYS: [Day; 5] = [Day::Mon, Day::Tue, Day::Wed, Day::Thu, Day::Fri];
const WEEKEND: [Day; 2] = [Day::Sat, Day::Sun];
const ALL_DAYS: [Day; 7] =
    [Day::Mon, Day::Tue, Day::Wed, Day::Thu, Day::Fri, Day::Sat, Day::Sun];

const CANCEL_VERBS: &[&str] = &[
    "cancel", "delete", "remove", "stop",
    "cancella", "elimina", "rimuovi", "togli", "disattiva", "spegni", "ferma",
];
const SET_VERBS: &[&str] = &[
    "set", "create", "add",
    "imposta", "crea", "metti", "aggiungi",
];

/// A number from a digit token ("7") or an Italian number-word ("sette").
/// English number-words never reach us — they're digits post-normalise.
fn num(tok: &str) -> Option<u8> {
    if let Ok(n) = tok.parse::<u8>() {
        return Some(n);
    }
    it_num(tok)
}

/// Italian cardinal 0–59 → value. Handles the units/teens table, the tens
/// (venti/trenta/quaranta/cinquanta) and their compounds with the standard
/// vowel-elision before uno(1)/otto(8): ventuno, ventotto, trentuno…
fn it_num(tok: &str) -> Option<u8> {
    const UNITS: &[(&str, u8)] = &[
        ("zero", 0), ("uno", 1), ("due", 2), ("tre", 3), ("quattro", 4),
        ("cinque", 5), ("sei", 6), ("sette", 7), ("otto", 8), ("nove", 9),
        ("dieci", 10), ("undici", 11), ("dodici", 12), ("tredici", 13),
        ("quattordici", 14), ("quindici", 15), ("sedici", 16),
        ("diciassette", 17), ("diciotto", 18), ("diciannove", 19),
    ];
    if let Some((_, n)) = UNITS.iter().find(|(w, _)| *w == tok) {
        return Some(*n);
    }
    // Bare tens.
    const TENS: &[(&str, u8)] = &[
        ("venti", 20), ("trenta", 30), ("quaranta", 40), ("cinquanta", 50),
    ];
    if let Some((_, n)) = TENS.iter().find(|(w, _)| *w == tok) {
        return Some(*n);
    }
    // Compounds: <tens><unit>, with the tens' final vowel dropped before
    // uno(1)/otto(8). e.g. "ventidue"=venti+due, "ventuno"=vent+uno.
    for (base, tenval) in [("venti", 20u8), ("trenta", 30), ("quaranta", 40), ("cinquanta", 50)] {
        for (uw, uv) in [
            ("uno", 1u8), ("due", 2), ("tre", 3), ("quattro", 4), ("cinque", 5),
            ("sei", 6), ("sette", 7), ("otto", 8), ("nove", 9),
        ] {
            let word = if uv == 1 || uv == 8 {
                let mut s = String::from(&base[..base.len() - 1]); // drop final vowel
                s.push_str(uw);
                s
            } else {
                let mut s = String::from(base);
                s.push_str(uw);
                s
            };
            if tok == word {
                return Some(tenval + uv);
            }
        }
    }
    None
}

pub fn classify(input: &str) -> Intent {
    let text = input.trim();
    let tokens: Vec<&str> = text.split_whitespace().collect();

    // Show intent: cancel/list/what-alarms (the platform API can't list/delete,
    // so we just open the Clock app). Bilingual keywords.
    let has_alarm = tokens.iter().any(|t| {
        matches!(*t, "alarm" | "alarms" | "sveglia" | "sveglie")
    });
    let is_cancel = tokens.windows(2).any(|w| w == ["turn", "off"])
        || tokens.iter().any(|t| CANCEL_VERBS.contains(t));
    let is_list = text.contains("what alarms")
        || text.contains("alarms do i")
        || text.contains("che sveglie ho")
        || text.contains("quali sveglie")
        || ((text.contains("list") || text.contains("elenca")) && has_alarm);
    if has_alarm && (is_cancel || is_list) {
        return Intent::Show;
    }

    let days = parse_days(&tokens);
    let time = parse_time(&tokens);

    match time {
        Some((hour, minute)) => {
            let message = parse_label(&tokens);
            Intent::Set { hour, minute, message, days }
        }
        None => {
            // Recognised a set request but no parseable time → ask for it.
            let set_verbish = tokens.iter().any(|t| SET_VERBS.contains(t)) && has_alarm;
            let wake = tokens.windows(3).any(|w| w == ["wake", "me", "up"])
                || tokens.iter().any(|t| *t == "svegliami");
            if set_verbish || wake {
                Intent::NeedTime
            } else {
                Intent::Unintelligible
            }
        }
    }
}

fn parse_days(tokens: &[&str]) -> Vec<Day> {
    let j = tokens.join(" ");
    // Weekday check MUST precede the every-day check: "ogni giorno feriale"
    // (every weekday) contains the substring "ogni giorno" (every day).
    if j.contains("weekday") || j.contains("feriale") || j.contains("feriali") {
        return WEEKDAYS.to_vec();
    }
    // weekend / "fine settimana"
    if j.contains("weekend") || j.contains("fine settimana") {
        return WEEKEND.to_vec();
    }
    // "every day" / daily / "ogni giorno" / "tutti i giorni"
    if j.contains("every day")
        || tokens.iter().any(|t| *t == "daily")
        || j.contains("ogni giorno")
        || j.contains("tutti i giorni")
    {
        return ALL_DAYS.to_vec();
    }
    // Named days, de-duplicated, in canonical week order.
    let mut found: Vec<Day> = Vec::new();
    for tok in tokens {
        if let Some((_, day)) = DAY_NAMES.iter().find(|(n, _)| n == tok) {
            if !found.contains(day) {
                found.push(*day);
            }
        }
    }
    if found.is_empty() {
        return Vec::new();
    }
    ALL_DAYS.iter().copied().filter(|d| found.contains(d)).collect()
}

/// Parse a time-of-day. Returns (hour 0-23, minute 0-59). Handles English
/// ("half past 6", "quarter to 7", "6 30 am") and Italian ("le sette e mezza",
/// "otto meno un quarto", "sette e venticinque", "mezzogiorno").
fn parse_time(tokens: &[&str]) -> Option<(u8, u8)> {
    let j = tokens.join(" ");
    if j.contains("mezzogiorno") || j.contains("noon") { return Some((12, 0)); }
    if j.contains("mezzanotte") || j.contains("midnight") { return Some((0, 0)); }

    // English fraction form: "half/quarter past/to <hour>".
    if let Some(pos) = tokens.iter().position(|t| *t == "past" || *t == "to") {
        let frac = if pos >= 1 { tokens[pos - 1] } else { "" };
        if let Some(mins) = match frac { "half" => Some(30u8), "quarter" => Some(15u8), _ => None } {
            if let Some(h) = tokens.get(pos + 1).and_then(|t| num(t)) {
                return Some(if tokens[pos] == "to" {
                    ((h + 23) % 24, 60 - mins)
                } else {
                    (h % 24, mins)
                });
            }
        }
    }

    // Italian fraction form, relative to the first hour number in the stream.
    if let Some(hidx) = tokens.iter().position(|t| num(t).map(|n| n <= 24).unwrap_or(false)) {
        let hour = num(tokens[hidx]).unwrap();
        let rest = &tokens[hidx + 1..];
        let rj = rest.join(" ");
        if rj.contains("e mezza") { return Some((hour % 24, 30)); }
        if rj.contains("e un quarto") { return Some((hour % 24, 15)); }
        if rj.contains("meno un quarto") { return Some(((hour + 23) % 24, 45)); }
        if rj.contains("in punto") { return Some((hour % 24, 0)); }
        // "e <minute>" where <minute> is a digit or an Italian number-word.
        if let Some(epos) = rest.iter().position(|t| *t == "e") {
            if let Some(m) = rest.get(epos + 1).and_then(|t| num(t)) {
                if m <= 59 { return Some((hour % 24, m)); }
            }
        }
        // No fraction/connector → fall through to plain numeric.
    }

    numeric_time(tokens)
}

/// Plain numeric time: "7", "7 am", "7 pm", "6 30", "6 30 am". `num` covers
/// digit and Italian-word hours.
fn numeric_time(tokens: &[&str]) -> Option<(u8, u8)> {
    let idx = tokens.iter().position(|t| num(t).map(|n| n <= 24).unwrap_or(false))?;
    let hour_raw = num(tokens[idx])?;

    let mut minute = 0u8;
    if let Some(next) = tokens.get(idx + 1) {
        if let Some(m) = num(next) {
            if m <= 59 { minute = m; }
        }
    }

    let meridian = tokens[idx + 1..].iter().take(2).find_map(|t| match *t {
        "am" => Some("am"),
        "pm" => Some("pm"),
        _ => None,
    });

    let hour = match meridian {
        Some("pm") if hour_raw < 12 => hour_raw + 12,
        Some("am") if hour_raw == 12 => 0,
        _ => hour_raw,
    };
    Some((hour % 24, minute))
}

/// Extract an alarm label. Word order differs by language:
/// - "alarm called X" / "sveglia chiamata X"  → X
/// - English adjective form "<X> alarm"        → words before "alarm"
/// - Italian noun-adjunct "sveglia <X>"        → words after "sveglia"
fn parse_label(tokens: &[&str]) -> Option<String> {
    // "called X" / "chiamata X" / "chiamato X" — single word after the keyword.
    if let Some(pos) = tokens.iter().position(|t| matches!(*t, "called" | "chiamata" | "chiamato")) {
        if let Some(name) = tokens.get(pos + 1) {
            if !LABEL_STOPWORDS.contains(name) && num(name).is_none() {
                return Some((*name).to_string());
            }
        }
    }
    // English adjective form: run of words directly before "alarm".
    if let Some(pos) = tokens.iter().position(|t| *t == "alarm") {
        let mut parts: Vec<&str> = Vec::new();
        for tok in tokens[..pos].iter().rev() {
            if LABEL_STOPWORDS.contains(tok) || num(tok).is_some() { break; }
            parts.push(tok);
        }
        if !parts.is_empty() {
            parts.reverse();
            return Some(parts.join(" "));
        }
    }
    // Italian noun-adjunct: run of words directly after "sveglia".
    if let Some(pos) = tokens.iter().position(|t| *t == "sveglia") {
        let mut parts: Vec<&str> = Vec::new();
        for tok in &tokens[pos + 1..] {
            if LABEL_STOPWORDS.contains(tok) || num(tok).is_some() { break; }
            parts.push(tok);
        }
        if !parts.is_empty() {
            return Some(parts.join(" "));
        }
    }
    None
}

/// Words that are never part of a label — verbs, articles, prepositions,
/// pronouns and time words, in both languages.
const LABEL_STOPWORDS: &[&str] = &[
    // English
    "set", "create", "add", "a", "an", "the", "my", "new", "for", "at", "please",
    "me", "past", "to", "half", "quarter", "in", "punto", "am", "pm", "wake", "up",
    "noon", "midnight", "every", "day", "weekday", "weekdays", "weekend",
    // Italian
    "imposta", "crea", "metti", "aggiungi", "una", "un", "uno", "la", "il", "le",
    "lo", "mia", "mio", "nuova", "nuovo", "per", "alle", "alla", "al", "svegliami",
    "e", "meno", "mezza", "mezzo", "ogni", "giorno", "giorni", "feriale", "feriali",
    "mezzogiorno", "mezzanotte", "chiamata", "chiamato",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn set(i: &str) -> (u8, u8, Option<String>, Vec<Day>) {
        match classify(i) {
            Intent::Set { hour, minute, message, days } => (hour, minute, message, days),
            other => panic!("expected Set, got {other:?}"),
        }
    }

    #[test]
    fn plain_am() {
        assert_eq!(set("set an alarm for 7 am"), (7, 0, None, vec![]));
    }

    #[test]
    fn pm_converts_to_24h() {
        assert_eq!(set("set an alarm for 7 pm"), (19, 0, None, vec![]));
    }

    #[test]
    fn colon_time_becomes_space_separated() {
        // "6:30" normalises to "6 30"
        assert_eq!(set("set an alarm for 6 30"), (6, 30, None, vec![]));
    }

    #[test]
    fn half_past() {
        // post-normalise: EN number-words are already digits ("six" → "6")
        assert_eq!(set("wake me up at half past 6"), (6, 30, None, vec![]));
    }

    #[test]
    fn quarter_to() {
        assert_eq!(set("set an alarm for quarter to 7"), (6, 45, None, vec![]));
    }

    #[test]
    fn noon_and_midnight() {
        assert_eq!(set("set an alarm for noon").0, 12);
        assert_eq!(set("set an alarm for midnight"), (0, 0, None, vec![]));
    }

    #[test]
    fn weekday_recurrence() {
        let (h, m, _, days) = set("set an alarm for 6 30 every weekday");
        assert_eq!((h, m), (6, 30));
        assert_eq!(days, vec![Day::Mon, Day::Tue, Day::Wed, Day::Thu, Day::Fri]);
    }

    #[test]
    fn weekend_recurrence() {
        let (_, _, _, days) = set("set an alarm for 8 am on saturdays and sundays");
        assert_eq!(days, vec![Day::Sat, Day::Sun]);
    }

    #[test]
    fn every_day() {
        let (_, _, _, days) = set("set an alarm for 7 am every day");
        assert_eq!(days, vec![Day::Mon, Day::Tue, Day::Wed, Day::Thu, Day::Fri, Day::Sat, Day::Sun]);
    }

    #[test]
    fn labelled_adjective_form() {
        let (h, m, msg, _) = set("gym alarm at 5 45");
        assert_eq!((h, m), (5, 45));
        assert_eq!(msg, Some("gym".to_string()));
    }

    #[test]
    fn labelled_called_form() {
        let (_, _, msg, _) = set("set an alarm called gym for 6 am");
        assert_eq!(msg, Some("gym".to_string()));
    }

    #[test]
    fn cancel_is_show() {
        assert_eq!(classify("cancel my 7 am alarm"), Intent::Show);
    }

    #[test]
    fn list_is_show() {
        assert_eq!(classify("what alarms do i have"), Intent::Show);
    }

    #[test]
    fn set_without_time_needs_time() {
        assert_eq!(classify("set an alarm"), Intent::NeedTime);
    }

    // --- Italian (post-normalise-it: elisions stripped, NO number-word→digit,
    //     so Italian number words like "sei" reach the parser intact) ---

    #[test]
    fn it_digits() {
        assert_eq!(set("imposta una sveglia per le 7"), (7, 0, None, vec![]));
    }

    #[test]
    fn it_number_word_hour() {
        assert_eq!(set("svegliami alle sette"), (7, 0, None, vec![]));
    }

    #[test]
    fn it_e_mezza() {
        assert_eq!(set("svegliami alle sei e mezza"), (6, 30, None, vec![]));
    }

    #[test]
    fn it_e_un_quarto() {
        assert_eq!(set("imposta una sveglia per le sette e un quarto"), (7, 15, None, vec![]));
    }

    #[test]
    fn it_meno_un_quarto() {
        assert_eq!(set("sveglia alle otto meno un quarto"), (7, 45, None, vec![]));
    }

    #[test]
    fn it_e_minuti_number_word() {
        // "e trenta" → :30 via Italian number-word minute
        assert_eq!(set("svegliami alle sette e trenta"), (7, 30, None, vec![]));
        // compound: "e venticinque" → :25
        assert_eq!(set("svegliami alle sette e venticinque"), (7, 25, None, vec![]));
    }

    #[test]
    fn it_mezzogiorno_mezzanotte() {
        assert_eq!(set("imposta una sveglia per mezzogiorno").0, 12);
        assert_eq!(set("imposta una sveglia per mezzanotte"), (0, 0, None, vec![]));
    }

    #[test]
    fn it_weekday_recurrence() {
        let (h, m, _, days) = set("imposta una sveglia per le 6 30 ogni giorno feriale");
        assert_eq!((h, m), (6, 30));
        assert_eq!(days, vec![Day::Mon, Day::Tue, Day::Wed, Day::Thu, Day::Fri]);
    }

    #[test]
    fn it_named_days() {
        let (_, _, _, days) = set("imposta una sveglia per le 8 il sabato e la domenica");
        assert_eq!(days, vec![Day::Sat, Day::Sun]);
    }

    #[test]
    fn it_ogni_giorno() {
        let (_, _, _, days) = set("svegliami alle 7 ogni giorno");
        assert_eq!(days.len(), 7);
    }

    #[test]
    fn it_label_noun_first() {
        // Italian puts the noun first: "sveglia palestra" → label "palestra"
        let (h, m, msg, _) = set("sveglia palestra alle 5 45");
        assert_eq!((h, m), (5, 45));
        assert_eq!(msg, Some("palestra".to_string()));
    }

    #[test]
    fn it_cancel_is_show() {
        assert_eq!(classify("cancella la mia sveglia delle 7"), Intent::Show);
        assert_eq!(classify("disattiva la sveglia"), Intent::Show);
    }

    #[test]
    fn it_list_is_show() {
        assert_eq!(classify("che sveglie ho"), Intent::Show);
    }
}
