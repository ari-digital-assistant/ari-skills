extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::parse::Parsed;

pub enum Decision {
    Play { query: String, service: String },
    Ask { query: String, installed: Vec<String> },
    Clarify,
    NoApp,
}

fn ask(query: String, installed: &[String]) -> Decision {
    if installed.is_empty() {
        Decision::NoApp
    } else {
        Decision::Ask { query, installed: installed.to_vec() }
    }
}

/// Pure resolution. `default_setting` is the raw `default_service` value
/// ("last_used" | "ask" | a canonical id). `last_used` is the stored id or None.
/// `installed` is the canonical ids from media_services().
pub fn decide(
    parsed: Parsed,
    default_setting: &str,
    last_used: Option<String>,
    installed: &[String],
) -> Decision {
    let query = match parsed.query {
        Some(q) => q,
        None => return Decision::Clarify,
    };
    let is_installed = |id: &str| installed.iter().any(|x| x == id);

    if let Some(svc) = parsed.service {
        return if is_installed(&svc) {
            Decision::Play { query, service: svc }
        } else {
            ask(query, installed)
        };
    }
    match default_setting {
        "ask" => ask(query, installed),
        "last_used" => match last_used {
            Some(svc) if is_installed(&svc) => Decision::Play { query, service: svc },
            _ => ask(query, installed),
        },
        specific if crate::parse::SERVICE_IDS.contains(&specific) => {
            if is_installed(specific) {
                Decision::Play { query, service: specific.to_string() }
            } else {
                ask(query, installed)
            }
        }
        _ => ask(query, installed), // unknown setting → treat as last_used/ask
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn installed() -> Vec<String> {
        vec!["spotify".to_string(), "youtube_music".to_string()]
    }
    fn parsed(q: Option<&str>, s: Option<&str>) -> Parsed {
        Parsed { query: q.map(|x| x.to_string()), service: s.map(|x| x.to_string()) }
    }

    #[test]
    fn no_query_clarifies() {
        let d = decide(parsed(None, None), "last_used", None, &installed());
        assert!(matches!(d, Decision::Clarify));
    }

    #[test]
    fn named_installed_service_plays() {
        let d = decide(parsed(Some("x"), Some("spotify")), "last_used", None, &installed());
        assert!(matches!(d, Decision::Play { ref service, .. } if service == "spotify"));
    }

    #[test]
    fn named_uninstalled_service_asks() {
        // tidal not installed → fall to ASK rather than dead-end
        let d = decide(parsed(Some("x"), Some("tidal")), "last_used", None, &installed());
        assert!(matches!(d, Decision::Ask { .. }));
    }

    #[test]
    fn specific_setting_used_when_no_named_service() {
        let d = decide(parsed(Some("x"), None), "youtube_music", None, &installed());
        assert!(matches!(d, Decision::Play { ref service, .. } if service == "youtube_music"));
    }

    #[test]
    fn last_used_present_and_installed_plays() {
        let d = decide(parsed(Some("x"), None), "last_used", Some("spotify".to_string()), &installed());
        assert!(matches!(d, Decision::Play { ref service, .. } if service == "spotify"));
    }

    #[test]
    fn last_used_missing_asks() {
        let d = decide(parsed(Some("x"), None), "last_used", None, &installed());
        assert!(matches!(d, Decision::Ask { ref query, .. } if query == "x"));
    }

    #[test]
    fn last_used_uninstalled_asks() {
        let d = decide(parsed(Some("x"), None), "last_used", Some("tidal".to_string()), &installed());
        assert!(matches!(d, Decision::Ask { .. }));
    }

    #[test]
    fn ask_each_time_asks() {
        let d = decide(parsed(Some("x"), None), "ask", None, &installed());
        assert!(matches!(d, Decision::Ask { .. }));
    }

    #[test]
    fn ask_with_no_installed_apps_is_no_app() {
        let d = decide(parsed(Some("x"), None), "ask", None, &[]);
        assert!(matches!(d, Decision::NoApp));
    }
}
