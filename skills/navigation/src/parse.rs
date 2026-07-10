use alloc::string::{String, ToString};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    Navigate { destination: String },
    NeedDestination,
}

/// English trigger phrases whose tail (everything after the phrase) is the
/// destination. Longest-first: a longer phrase that contains a shorter one must
/// be tried first. Each ends with a trailing space so a bare "take me to" (no
/// destination) does NOT match.
const EN_PREFIXES: &[&str] = &[
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
];

/// Italian trigger verbs. Longest-first so "come ci arrivo" is tried before
/// "come arrivo". The tail after the verb is the destination, but Italian
/// contracts the preposition with the article, so a leading connector (below)
/// must be stripped from that tail before cleaning.
const IT_VERBS: &[&str] = &[
    "come ci arrivo",
    "come arrivo",
    "indicazioni per",
    "portami",
    "andiamo",
    "vai",
];

/// Leading Italian prepositions/contractions to strip off the tail after an
/// IT verb (al = a+il, alla = a+la, allo = a+lo, all' = a+l', ai = a+i,
/// agli = a+gli, alle = a+le, plus in/per/a and the "nel*" family for "in").
/// Longest-first so "al " is tried before "a ". Each ends with a trailing space
/// so we only strip a genuine leading preposition, not a word that starts with
/// one (e.g. destination "asda" is untouched by "a ").
const IT_CONNECTORS: &[&str] = &[
    "allo ", "alla ", "agli ", "alle ", "all ",
    "nello ", "nella ", "negli ", "nelle ", "nei ",
    "ai ", "al ", "nel ", "in ", "per ", "a ",
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

    if let Some(dest) = extract_destination(text) {
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

/// Find the first trigger present in `text` and return the cleaned destination.
/// Returns `None` only when no trigger matched at all; a matched-but-empty tail
/// yields `Some("")` so the caller can ask for a destination.
fn extract_destination(text: &str) -> Option<String> {
    // English: verb + "to " — the tail is the destination directly.
    for prefix in EN_PREFIXES {
        if let Some(idx) = text.find(prefix) {
            return Some(clean(&text[idx + prefix.len()..]));
        }
    }
    // Italian: trigger verb, then strip a leading contracted preposition.
    for verb in IT_VERBS {
        if let Some(idx) = text.find(verb) {
            let tail = text[idx + verb.len()..].trim_start();
            return Some(clean(strip_connector(tail)));
        }
    }
    None
}

/// Strip one leading Italian preposition/contraction (longest-first) from a tail.
fn strip_connector(tail: &str) -> &str {
    for c in IT_CONNECTORS {
        if let Some(rest) = tail.strip_prefix(c) {
            return rest;
        }
    }
    tail
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

    // --- Italian contracted prepositions (al/alla/allo/in) ---
    #[test] fn it_come_arrivo_alla() { assert_eq!(dest("come arrivo alla stazione"), "stazione"); }
    #[test] fn it_come_arrivo_al() { assert_eq!(dest("come arrivo al museo"), "museo"); }
    #[test] fn it_vai_alla() { assert_eq!(dest("vai alla stazione"), "stazione"); }
    #[test] fn it_andiamo_al() { assert_eq!(dest("andiamo al mare"), "mare"); }
    #[test] fn it_portami_still_works() { assert_eq!(dest("portami al lavoro"), "lavoro"); }
}
