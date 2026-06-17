#![allow(dead_code)] // consumed by lib.rs wire-up (later task)

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

/// `args_json` is the FunctionGemma extraction (may be None when matched by
/// keyword); `raw` is the normalised utterance, used as a fallback for
/// `when` and to detect the facet (which is never in args).
pub fn parse_request(args_json: Option<&str>, raw: &str) -> Request {
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
        let r = parse_request(Some(r#"{"location":"tokyo","when":"tomorrow"}"#), "weather in tokyo tomorrow");
        assert_eq!(r.location.as_deref(), Some("tokyo"));
        assert_eq!(r.when, When::Tomorrow);
    }
    #[test]
    fn empty_location_is_gps_path() {
        let r = parse_request(Some(r#"{"location":"","when":"now"}"#), "how is the weather");
        assert!(r.location.is_none());
        assert_eq!(r.when, When::Now);
    }
    #[test]
    fn facet_detected_from_raw_text_when_no_args() {
        let r = parse_request(None, "is it windy");
        assert_eq!(r.facet, Facet::Wind);
        let r2 = parse_request(None, "what is the uv index");
        assert_eq!(r2.facet, Facet::Uv);
        let r3 = parse_request(None, "will it rain today");
        assert_eq!(r3.facet, Facet::Rain);
        assert_eq!(r3.when, When::Today);
    }
    #[test]
    fn when_parsed_from_raw_text() {
        assert_eq!(parse_request(None, "weather this week").when, When::ThisWeek);
        assert_eq!(parse_request(None, "weather tomorrow").when, When::Tomorrow);
        assert_eq!(parse_request(None, "weather").when, When::Now);
    }
    #[test]
    fn metno_only_for_gps_now() {
        assert!(parse_request(None, "weather").use_metno());
        assert!(!parse_request(None, "weather this week").use_metno());
        assert!(!parse_request(Some(r#"{"location":"rome","when":"now"}"#), "weather in rome").use_metno());
    }
    #[test]
    fn italian_when_and_facet_detected() {
        assert_eq!(parse_request(None, "che tempo fa domani").when, When::Tomorrow);
        assert_eq!(parse_request(None, "previsioni questa settimana").when, When::ThisWeek);
        assert_eq!(parse_request(None, "che tempo fa oggi").when, When::Today);
        assert_eq!(parse_request(None, "c'e vento").facet, Facet::Wind);
        assert_eq!(parse_request(None, "pioverà oggi").facet, Facet::Rain);
        assert_eq!(parse_request(None, "indice uv").facet, Facet::Uv);
    }
}
