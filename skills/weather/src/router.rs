use alloc::string::{String, ToString};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum When { Now, Today, Tomorrow, ThisWeek }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facet { None, Wind, Rain, Uv }

#[derive(Debug, Clone, PartialEq)]
pub struct Request { pub location: Option<String>, pub when: When, pub facet: Facet }

fn when_from_str(s: &str) -> When {
    let s = s.to_lowercase();
    // English + Italian time words (it is a launch language; `when` and the
    // facet are detected from text, so they must be bilingual).
    if s.contains("week") || s.contains("settimana") { When::ThisWeek }
    else if s.contains("tomorrow") || s.contains("domani") { When::Tomorrow }
    else if s.contains("today") || s.contains("oggi") { When::Today }
    else { When::Now }
}

fn facet_from_text(t: &str) -> Facet {
    let t = t.to_lowercase();
    if t.contains("uv") { Facet::Uv }
    else if t.contains("wind") || t.contains("vento") { Facet::Wind }
    else if t.contains("rain") || t.contains("piov") || t.contains("piogg") { Facet::Rain }
    else { Facet::None }
}

/// Best-effort place extraction from the raw utterance, used as a fallback
/// when the model's arg extraction doesn't supply a `location` (e.g. a
/// freshly-installed skill the on-device router isn't trained on yet). Takes
/// the text after the last locative preposition (`in` for en, `a` for it) and
/// strips any trailing time word. Returns `None` for the GPS path.
fn location_from_text(raw: &str, locale: &str) -> Option<String> {
    let lower = raw.to_lowercase();
    let prep = if locale.starts_with("it") { " a " } else { " in " };
    let idx = lower.rfind(prep)?;
    let mut rest = lower[idx + prep.len()..].trim().to_string();
    // Drop a trailing time word so "london today" → "london".
    for w in [
        " this week", " tomorrow", " today", " now",
        " questa settimana", " domani", " oggi",
    ] {
        if let Some(s) = rest.strip_suffix(w) {
            rest = s.trim().to_string();
        }
    }
    if rest.is_empty() { None } else { Some(rest) }
}

/// `args_json` is the model's argument extraction (may be `None` when the
/// skill was matched by keyword); `raw` is the normalised utterance, used to
/// detect the facet (never in args), as a fallback for `when`, and — when the
/// model didn't extract a `location` — as a fallback place source. `locale`
/// selects the locative preposition for that place fallback.
pub fn parse_request(args_json: Option<&str>, raw: &str, locale: &str) -> Request {
    let mut location: Option<String> = None;
    let mut when = when_from_str(raw);
    if let Some(j) = args_json {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(j) {
            if let Some(loc) = v.get("location").and_then(|x| x.as_str()) {
                let loc = loc.trim();
                if !loc.is_empty() { location = Some(loc.to_string()); }
            }
            if let Some(w) = v.get("when").and_then(|x| x.as_str()) {
                when = when_from_str(w);
            }
        }
    }
    // Fall back to parsing the place from the utterance when extraction didn't
    // give us one (e.g. the on-device router isn't trained on this skill yet).
    if location.is_none() {
        location = location_from_text(raw, locale);
    }
    Request { location, when, facet: facet_from_text(raw) }
}

impl Request {
    /// True when this should hit MET Norway (GPS + current). Everything else
    /// goes to Open-Meteo.
    pub fn use_metno(&self) -> bool {
        self.location.is_none() && self.when == When::Now
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_args_json() {
        let r = parse_request(Some(r#"{"location":"tokyo","when":"tomorrow"}"#), "weather in tokyo tomorrow", "en");
        assert_eq!(r.location.as_deref(), Some("tokyo"));
        assert_eq!(r.when, When::Tomorrow);
    }
    #[test]
    fn empty_location_is_gps_path() {
        let r = parse_request(Some(r#"{"location":"","when":"now"}"#), "how is the weather", "en");
        assert!(r.location.is_none());
        assert_eq!(r.when, When::Now);
    }
    #[test]
    fn facet_detected_from_raw_text_when_no_args() {
        let r = parse_request(None, "is it windy", "en");
        assert_eq!(r.facet, Facet::Wind);
        let r2 = parse_request(None, "what is the uv index", "en");
        assert_eq!(r2.facet, Facet::Uv);
        let r3 = parse_request(None, "will it rain today", "en");
        assert_eq!(r3.facet, Facet::Rain);
        assert_eq!(r3.when, When::Today);
    }
    #[test]
    fn when_parsed_from_raw_text() {
        assert_eq!(parse_request(None, "weather this week", "en").when, When::ThisWeek);
        assert_eq!(parse_request(None, "weather tomorrow", "en").when, When::Tomorrow);
        assert_eq!(parse_request(None, "weather", "en").when, When::Now);
    }
    #[test]
    fn metno_only_for_gps_now() {
        assert!(parse_request(None, "weather", "en").use_metno());
        assert!(!parse_request(None, "weather this week", "en").use_metno());
        assert!(!parse_request(Some(r#"{"location":"rome","when":"now"}"#), "weather in rome", "en").use_metno());
    }
    #[test]
    fn italian_when_and_facet_detected() {
        assert_eq!(parse_request(None, "che tempo fa domani", "it").when, When::Tomorrow);
        assert_eq!(parse_request(None, "previsioni questa settimana", "it").when, When::ThisWeek);
        assert_eq!(parse_request(None, "che tempo fa oggi", "it").when, When::Today);
        assert_eq!(parse_request(None, "c'e vento", "it").facet, Facet::Wind);
        assert_eq!(parse_request(None, "pioverà oggi", "it").facet, Facet::Rain);
        assert_eq!(parse_request(None, "indice uv", "it").facet, Facet::Uv);
    }
    #[test]
    fn location_parsed_from_text_when_extraction_empty() {
        // No args (router didn't extract) → place comes from the utterance.
        let r = parse_request(None, "weather in tokyo", "en");
        assert_eq!(r.location.as_deref(), Some("tokyo"));
        let r2 = parse_request(None, "will it rain in london today", "en");
        assert_eq!(r2.location.as_deref(), Some("london"));
        assert_eq!(r2.when, When::Today);
        // Italian uses the "a" preposition.
        let r3 = parse_request(None, "meteo a roma domani", "it");
        assert_eq!(r3.location.as_deref(), Some("roma"));
        assert_eq!(r3.when, When::Tomorrow);
        // No locative preposition → GPS path.
        assert!(parse_request(None, "how is the weather", "en").location.is_none());
    }
    #[test]
    fn extracted_location_wins_over_text() {
        // When the model DID extract a place, it takes precedence.
        let r = parse_request(Some(r#"{"location":"paris","when":"now"}"#), "weather in london", "en");
        assert_eq!(r.location.as_deref(), Some("paris"));
    }
}
