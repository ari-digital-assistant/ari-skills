extern crate alloc;
use alloc::string::{String, ToString};

pub const SERVICE_IDS: [&str; 5] = [
    "spotify", "apple_music", "tidal", "deezer", "amazon_music",
];

/// (canonical id, alias). Longest aliases first so "amazon music" wins over "amazon".
const SERVICE_ALIASES: &[(&str, &str)] = &[
    ("amazon_music", "amazon music"),
    ("apple_music", "apple music"),
    ("spotify", "spotify"),
    ("tidal", "tidal"),
    ("deezer", "deezer"),
];

/// Tried in order, and the first that matches anywhere wins, so a verb must
/// come before any trigger it could follow: "vorrei" is last because "vorrei
/// ascoltare X" means X, not "ascoltare X".
const TRIGGERS: &[&str] = &[
    "listen to", "put on", "fai partire",
    "play", "riproduci", "mettimi", "mettere", "metti", "ascoltare", "ascolta",
    "sentire", "senti", "suona", "vorrei",
];

const CONNECTORS: &[&str] = &["on", "su"];

pub struct Parsed {
    pub query: Option<String>,
    pub service: Option<String>,
}

pub fn canonical_service(s: &str) -> Option<String> {
    let s = s.trim().to_lowercase();
    if SERVICE_IDS.contains(&s.as_str()) {
        return Some(s);
    }
    SERVICE_ALIASES.iter().find(|(_, a)| *a == s).map(|(id, _)| (*id).to_string())
}

fn after_trigger(input: &str) -> Option<&str> {
    let bytes = input.as_bytes();
    for trig in TRIGGERS {
        let mut from = 0;
        while let Some(pos) = input[from..].find(trig) {
            let abs = from + pos;
            let before_ok = abs == 0 || bytes[abs - 1] == b' ';
            let end = abs + trig.len();
            let followed_by_space = bytes.get(end) == Some(&b' ');
            if before_ok && followed_by_space {
                let rest = input[end + 1..].trim();
                if !rest.is_empty() {
                    return Some(rest);
                }
            }
            from = abs + 1;
        }
    }
    None
}

fn split_service(raw: &str) -> (String, Option<String>) {
    for conn in CONNECTORS {
        for (id, alias) in SERVICE_ALIASES {
            // suffix form: "<query> <conn> <alias>"
            let suffix = alloc::format!(" {conn} {alias}");
            if let Some(stripped) = raw.strip_suffix(&suffix) {
                return (stripped.trim().to_string(), Some((*id).to_string()));
            }
            // exact form: "<conn> <alias>" (service named, no query)
            let exact = alloc::format!("{conn} {alias}");
            if raw == exact {
                return (String::new(), Some((*id).to_string()));
            }
        }
    }
    (raw.trim().to_string(), None)
}

pub fn parse(input: &str) -> Parsed {
    match after_trigger(input) {
        Some(raw) => {
            // "metti su X" is Italian for "put on X"; the "su" is part of the
            // verb unless a service follows it ("metti su spotify").
            let raw = match raw.strip_prefix("su ") {
                Some(rest) if canonical_service(rest).is_none() => rest,
                _ => raw,
            };
            let (query, service) = split_service(raw);
            Parsed {
                query: if query.is_empty() { None } else { Some(query) },
                service,
            }
        }
        None => Parsed { query: None, service: None },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_query_only() {
        let p = parse("play hotel california");
        assert_eq!(p.query.as_deref(), Some("hotel california"));
        assert_eq!(p.service, None);
    }

    #[test]
    fn parses_named_service_tail() {
        let p = parse("play hotel california on spotify");
        assert_eq!(p.query.as_deref(), Some("hotel california"));
        assert_eq!(p.service.as_deref(), Some("spotify"));
    }

    #[test]
    fn song_title_containing_on_is_not_split() {
        let p = parse("play knockin on heavens door");
        assert_eq!(p.query.as_deref(), Some("knockin on heavens door"));
        assert_eq!(p.service, None);
    }

    #[test]
    fn italian_trigger_and_su_connector() {
        let p = parse("metti hotel california su spotify");
        assert_eq!(p.query.as_deref(), Some("hotel california"));
        assert_eq!(p.service.as_deref(), Some("spotify"));
    }

    #[test]
    fn italian_metti_su_is_put_on() {
        assert_eq!(parse("metti su vasco rossi").query.as_deref(), Some("vasco rossi"));
        let p = parse("metti su vasco rossi su spotify");
        assert_eq!(p.query.as_deref(), Some("vasco rossi"));
        assert_eq!(p.service.as_deref(), Some("spotify"));
        let p = parse("metti su spotify");
        assert_eq!(p.query, None);
        assert_eq!(p.service.as_deref(), Some("spotify"));
    }

    #[test]
    fn italian_want_to_hear() {
        let q = |i| parse(i).query;
        assert_eq!(q("voglio ascoltare vasco rossi").as_deref(), Some("vasco rossi"));
        assert_eq!(q("vorrei ascoltare i pink floyd").as_deref(), Some("i pink floyd"));
        assert_eq!(q("voglio sentire vasco rossi").as_deref(), Some("vasco rossi"));
        assert_eq!(q("fammi sentire un po di jazz").as_deref(), Some("un po di jazz"));
        assert_eq!(q("fammi ascoltare vasco rossi").as_deref(), Some("vasco rossi"));
        assert_eq!(q("mettimi vasco rossi").as_deref(), Some("vasco rossi"));
        assert_eq!(q("puoi mettere vasco rossi").as_deref(), Some("vasco rossi"));
        assert_eq!(q("senti vasco rossi").as_deref(), Some("vasco rossi"));
        assert_eq!(q("vorrei un po di jazz").as_deref(), Some("un po di jazz"));
    }

    #[test]
    fn no_trigger_yields_no_query() {
        assert_eq!(parse("what time is it").query, None);
    }

    #[test]
    fn service_only_yields_no_query() {
        let p = parse("play on spotify");
        assert_eq!(p.query, None);
        assert_eq!(p.service.as_deref(), Some("spotify"));
    }

    #[test]
    fn canonical_service_resolves_aliases_and_case() {
        assert_eq!(canonical_service("Spotify").as_deref(), Some("spotify"));
        assert_eq!(canonical_service("apple music").as_deref(), Some("apple_music"));
        assert_eq!(canonical_service("pandora"), None);
    }
}
