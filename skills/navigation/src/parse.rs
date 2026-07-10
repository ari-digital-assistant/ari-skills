use alloc::string::{String, ToString};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    Navigate { destination: String },
    NeedDestination,
}

/// Trigger phrases whose tail (everything after the phrase) is the destination.
/// Longest-first: a longer phrase that contains a shorter one must be tried
/// first. Each ends with a trailing space so a bare "take me to" (no
/// destination) does NOT match.
const NAV_PREFIXES: &[&str] = &[
    // English
    "show me the way to ",
    "how do i get to ",
    "take me to ",
    "bring me to ",
    "drive me to ",
    "get me to ",
    "navigate to ",
    "directions to ",
    "route to ",
    "the way to ",
    // Italian (include the common contracted prepositions al/alla/allo/in)
    "come ci arrivo a ",
    "come arrivo a ",
    "indicazioni per ",
    "portami alla ",
    "portami allo ",
    "portami al ",
    "portami in ",
    "portami a ",
    "andiamo a ",
    "vai a ",
];

/// "Take me home" style phrases (no explicit destination token) → "home".
/// (Italian "portami a casa" is caught by the "portami a " prefix → "casa",
/// which the maps app resolves as home.)
const HOME_PHRASES: &[&str] = &[
    "take me home", "bring me home", "drive me home", "get me home",
    "take me back home",
];

/// Leading articles to strip off an extracted destination (both languages).
const ARTICLES: &[&str] = &["the ", "a ", "an ", "il ", "la ", "lo ", "l "];

/// Trailing politeness to strip.
const POLITE_TAILS: &[&str] = &[" please", " thanks", " grazie", " per favore"];

pub fn classify(input: &str) -> Intent {
    let text = input.trim();

    if let Some(tail) = extract_destination(text) {
        let dest = clean(tail);
        if dest.is_empty() {
            return Intent::NeedDestination;
        }
        return Intent::Navigate { destination: dest };
    }

    if HOME_PHRASES.iter().any(|p| text.contains(p)) {
        return Intent::Navigate { destination: "home".to_string() };
    }

    // Routed here (matched the skill patterns) but no destination parsed.
    Intent::NeedDestination
}

/// Find the first trigger prefix present in `text` and return the tail after it.
fn extract_destination(text: &str) -> Option<&str> {
    for prefix in NAV_PREFIXES {
        if let Some(idx) = text.find(prefix) {
            return Some(&text[idx + prefix.len()..]);
        }
    }
    None
}

/// Trim leading articles and trailing politeness off a raw destination.
fn clean(raw: &str) -> String {
    let mut s = raw.trim();
    for art in ARTICLES {
        if let Some(rest) = s.strip_prefix(art) {
            s = rest.trim_start();
            break;
        }
    }
    for tail in POLITE_TAILS {
        if let Some(head) = s.strip_suffix(tail) {
            s = head.trim_end();
            break;
        }
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dest(i: &str) -> String {
        match classify(i) {
            Intent::Navigate { destination } => destination,
            other => panic!("expected Navigate, got {other:?}"),
        }
    }

    #[test] fn take_me_to() { assert_eq!(dest("take me to mcdonalds"), "mcdonalds"); }
    #[test] fn navigate_to_strips_article() { assert_eq!(dest("navigate to the science museum"), "science museum"); }
    #[test] fn directions_to() { assert_eq!(dest("directions to asda"), "asda"); }
    #[test] fn route_to() { assert_eq!(dest("route to the airport"), "airport"); }
    #[test] fn show_me_the_way_to() { assert_eq!(dest("show me the way to the station"), "station"); }
    #[test] fn how_do_i_get_to() { assert_eq!(dest("how do i get to the airport"), "airport"); }
    #[test] fn take_me_home() { assert_eq!(dest("take me home"), "home"); }
    #[test] fn drive_me_home() { assert_eq!(dest("drive me home"), "home"); }
    #[test] fn take_me_to_work() { assert_eq!(dest("take me to work"), "work"); }
    #[test] fn trailing_please_stripped() { assert_eq!(dest("take me to the o2 please"), "o2"); }

    #[test] fn bare_take_me_to_needs_destination() {
        assert_eq!(classify("take me to"), Intent::NeedDestination);
    }

    // --- Italian ---
    #[test] fn it_portami_a() { assert_eq!(dest("portami a asda"), "asda"); }
    #[test] fn it_portami_a_casa() { assert_eq!(dest("portami a casa"), "casa"); }
    #[test] fn it_portami_al_lavoro() { assert_eq!(dest("portami al lavoro"), "lavoro"); }
    #[test] fn it_come_arrivo_a() { assert_eq!(dest("come arrivo a mcdonalds"), "mcdonalds"); }
    #[test] fn it_come_ci_arrivo_a() { assert_eq!(dest("come ci arrivo a asda"), "asda"); }
    #[test] fn it_indicazioni_per_strips_article() { assert_eq!(dest("indicazioni per il museo"), "museo"); }
}
