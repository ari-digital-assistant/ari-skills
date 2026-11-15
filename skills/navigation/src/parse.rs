use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    Navigate { destination: String },
    /// The user's own home. `said` is the word they used ("home", "casa"),
    /// for a maps app that knows no saved home and can only search for it.
    Home { said: String },
    NeedDestination,
}

/// Destinations that mean the user's own home, after [`clean`].
const HOME_WORDS: &[&str] = &["home", "my home", "casa", "casa mia", "mia casa"];

/// English words that introduce the destination. Tried in this order, so
/// " to " wins over " for " when a sentence has both.
const EN_MARKERS: &[&str] = &[" to ", " towards ", " for "];

/// A marker followed by one of these is part of the request, not the
/// destination: "i want to GO to the station", "show me how to GET to it".
const EN_MOTION_VERBS: &[&str] = &["go", "get", "drive", "head", "navigate", "find", "reach"];

/// English phrasings with no marker word, whose tail is the destination:
/// "take me the station", "fastest route the station".
const EN_BARE_PREFIXES: &[&str] = &["take me ", "fastest route ", "quickest route "];

/// Italian navigation verbs and nouns, as token runs. The destination follows
/// the LAST one in the utterance: "qual è la strada più veloce per arrivare a
/// X" names X after "arrivare", not after "strada".
const IT_TRIGGERS: &[&[&str]] = &[
    &["come", "ci", "si", "arriva"],
    &["come", "si", "arriva"],
    &["come", "ci", "arrivo"],
    &["come", "arrivo"],
    &["indicazioni", "per"],
    &["portami"], &["riportami"], &["portaci"], &["portarmi"], &["porti"],
    &["accompagnami"], &["guidami"], &["conducimi"],
    &["naviga"], &["navigatore"], &["navigazione"],
    &["andare"], &["andiamo"], &["vai"],
    &["arrivare"], &["tornare"], &["torna"], &["raggiungere"],
    &["strada"], &["rotta"],
];

/// Leading words to drop between an Italian trigger and the destination:
/// prepositions and their contractions (al = a+il, all = a+l', …), plus the
/// fillers that sit there ("fino al", "di nuovo a", "più veloce per").
const IT_LEADING: &[&str] = &[
    "a", "ad", "al", "allo", "alla", "ai", "agli", "alle", "all",
    "in", "nel", "nello", "nella", "nei", "negli", "nelle",
    "per", "verso", "fino", "da", "di", "nuovo",
    "più", "piu", "veloce", "breve", "rapida", "migliore", "subito", "velocemente",
];

/// Trailing modifiers that describe the trip, not the place.
const TRAILING: &[&str] = &[
    " il prima possibile", " entro un ora", " in macchina", " in auto", " da qui",
    " adesso", " subito", " please", " thanks", " grazie", " per favore",
];

/// Leading articles to strip off an extracted destination (both languages).
const ARTICLES: &[&str] = &["the ", "a ", "an ", "il ", "la ", "lo ", "l "];

pub fn classify(input: &str) -> Intent {
    let text = input.trim();

    let destination = en_after_marker(text)
        .or_else(|| en_sat_nav(text))
        .or_else(|| {
            // "navigate home", "quickest way home", "how do i get home from
            // here": no marker, but "home" says where.
            text.split_whitespace().any(|w| w == "home").then(|| "home".to_string())
        })
        .or_else(|| {
            EN_BARE_PREFIXES
                .iter()
                .find_map(|p| text.find(p).map(|i| clean(&text[i + p.len()..])))
        })
        .or_else(|| it_after_trigger(text));

    match destination {
        Some(dest) if HOME_WORDS.contains(&dest.as_str()) => Intent::Home { said: dest },
        Some(dest) if !dest.is_empty() => Intent::Navigate { destination: dest },
        // Routed here (matched the skill's patterns) but nothing names a place.
        _ => Intent::NeedDestination,
    }
}

/// The text after the first marker that isn't followed by a motion verb.
/// `Some("")` for a bare "take me to", so the caller asks where.
fn en_after_marker(text: &str) -> Option<String> {
    let padded = alloc::format!(" {text} ");
    for marker in EN_MARKERS {
        for (i, _) in padded.match_indices(marker) {
            let tail = &padded[i + marker.len()..];
            let first = tail.split_whitespace().next().unwrap_or("");
            if !EN_MOTION_VERBS.contains(&first) {
                return Some(clean(tail));
            }
        }
    }
    None
}

/// "put the station in the sat nav", "put the station in my sat nav".
fn en_sat_nav(text: &str) -> Option<String> {
    let rest = &text[text.find("put ")? + 4..];
    let end = rest.find(" in the sat nav").or_else(|| rest.find(" in my sat nav"))?;
    Some(clean(&rest[..end]))
}

fn it_after_trigger(text: &str) -> Option<String> {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let end = IT_TRIGGERS
        .iter()
        .filter_map(|trigger| {
            tokens
                .windows(trigger.len())
                .rposition(|w| w == *trigger)
                .map(|start| start + trigger.len())
        })
        .max()?;
    // "prima" is filler only before a preposition: "come arrivo prima a
    // casa" (soonest), but "portami a prima porta" (a place in Rome).
    let rest = &tokens[end..];
    let skip = (0..rest.len())
        .take_while(|&i| {
            IT_LEADING.contains(&rest[i])
                || (rest[i] == "prima" && rest.get(i + 1).is_some_and(|n| IT_LEADING.contains(n)))
        })
        .count();
    Some(clean(&tokens[end + skip..].join(" ")))
}

/// Trim leading articles and trailing trip modifiers off a raw destination.
fn clean(raw: &str) -> String {
    let mut s = raw.trim();
    for art in ARTICLES {
        if let Some(rest) = s.strip_prefix(art) {
            s = rest.trim_start();
            break;
        }
    }
    while let Some(head) = TRAILING.iter().find_map(|tail| s.strip_suffix(tail)) {
        s = head.trim_end();
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dest(i: &str) -> String {
        match classify(i) {
            Intent::Navigate { destination } => destination,
            other => panic!("expected Navigate for {i:?}, got {other:?}"),
        }
    }

    fn home(i: &str) -> String {
        match classify(i) {
            Intent::Home { said } => said,
            other => panic!("expected Home for {i:?}, got {other:?}"),
        }
    }

    #[test] fn a_named_place_with_home_in_it_is_not_home() {
        assert_eq!(dest("navigate to home depot"), "home depot");
        assert_eq!(dest("portami a casa di marco"), "casa di marco");
    }
    #[test] fn my_home_is_home() {
        assert_eq!(home("take me to my home"), "my home");
        assert_eq!(home("portami alla mia casa"), "mia casa");
    }

    #[test] fn take_me_to() { assert_eq!(dest("take me to mcdonalds"), "mcdonalds"); }
    #[test] fn navigate_to_strips_article() { assert_eq!(dest("navigate to the science museum"), "science museum"); }
    #[test] fn directions_to() { assert_eq!(dest("directions to asda"), "asda"); }
    #[test] fn route_to() { assert_eq!(dest("route to the airport"), "airport"); }
    #[test] fn show_me_the_way_to() { assert_eq!(dest("show me the way to the station"), "station"); }
    #[test] fn how_do_i_get_to() { assert_eq!(dest("how do i get to the airport"), "airport"); }
    #[test] fn take_me_home() { assert_eq!(home("take me home"), "home"); }
    #[test] fn drive_me_home() { assert_eq!(home("drive me home"), "home"); }
    #[test] fn take_me_to_work() { assert_eq!(dest("take me to work"), "work"); }
    #[test] fn trailing_please_stripped() { assert_eq!(dest("take me to the o2 please"), "o2"); }

    #[test] fn bare_take_me_to_needs_destination() {
        assert_eq!(classify("take me to"), Intent::NeedDestination);
    }

    #[test] fn nothing_named_needs_destination() {
        assert_eq!(classify("start navigation"), Intent::NeedDestination);
    }

    // --- English phrasings from the manifest ---
    #[test] fn en_motion_verb_after_to_is_skipped() {
        assert_eq!(dest("i want to go to the train station"), "train station");
        assert_eq!(dest("i need to get to the station"), "station");
        assert_eq!(dest("show me how to get to the station"), "station");
        assert_eq!(dest("what is the best way to get to the station"), "station");
        assert_eq!(dest("i want to drive over to my office"), "my office");
    }
    #[test] fn en_other_to_phrasings() {
        assert_eq!(dest("guide me to the station"), "station");
        assert_eq!(dest("plot a course to the station"), "station");
        assert_eq!(dest("lets head to the station"), "station");
        assert_eq!(dest("whats the best way to the station"), "station");
        assert_eq!(dest("navigate me to the station"), "station");
        assert_eq!(dest("sat nav to the station"), "station");
        assert_eq!(dest("directions please to the station"), "station");
        assert_eq!(dest("can you map out a journey to the station"), "station");
    }
    #[test] fn en_for_and_towards() {
        assert_eq!(dest("get directions for the station"), "station");
        assert_eq!(dest("head for the station"), "station");
        assert_eq!(dest("fire up the sat nav for the station"), "station");
        assert_eq!(dest("point me towards the station"), "station");
    }
    #[test] fn en_sat_nav() {
        assert_eq!(dest("put the station in the sat nav"), "station");
        assert_eq!(dest("put the station in my sat nav"), "station");
    }
    #[test] fn en_home_without_a_marker() {
        assert_eq!(home("navigate home"), "home");
        assert_eq!(home("quickest way home"), "home");
        assert_eq!(home("i want to head home"), "home");
        assert_eq!(home("how do i get home from here"), "home");
        assert_eq!(home("route me home"), "home");
        assert_eq!(home("get me back home"), "home");
    }
    #[test] fn en_bare_prefixes() {
        assert_eq!(dest("take me the station"), "station");
        assert_eq!(dest("fastest route the station"), "station");
    }

    // --- Italian ---
    #[test] fn it_portami_a() { assert_eq!(dest("portami a asda"), "asda"); }
    #[test] fn it_portami_a_casa() { assert_eq!(home("portami a casa"), "casa"); }
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

    // --- Italian phrasings from the manifest ---
    #[test] fn it_more_verbs() {
        assert_eq!(home("voglio andare a casa"), "casa");
        assert_eq!(dest("voglio andare alla stazione centrale"), "stazione centrale");
        assert_eq!(dest("accompagnami in ufficio"), "ufficio");
        assert_eq!(dest("guidami fino al museo"), "museo");
        assert_eq!(dest("conducimi alla stazione per favore"), "stazione");
        assert_eq!(dest("mi porti alla stazione"), "stazione");
        assert_eq!(dest("puoi portarmi alla stazione"), "stazione");
        assert_eq!(dest("portaci al mare"), "mare");
        assert_eq!(dest("naviga verso milano"), "milano");
        assert_eq!(dest("naviga fino a milano"), "milano");
        assert_eq!(dest("come si arriva a milano"), "milano");
        assert_eq!(dest("come ci si arriva alla stazione"), "stazione");
        assert_eq!(dest("devo raggiungere la stazione"), "stazione");
        assert_eq!(home("voglio tornare a casa"), "casa");
        assert_eq!(dest("fammi da navigatore per la stazione"), "stazione");
        assert_eq!(dest("imposta la rotta per milano"), "milano");
        assert_eq!(dest("trovami la strada per la stazione"), "stazione");
    }
    #[test] fn it_last_trigger_wins() {
        assert_eq!(dest("qual è la strada più veloce per arrivare alla stazione"), "stazione");
        assert_eq!(dest("qual è la strada più veloce per la stazione"), "stazione");
        assert_eq!(dest("quanto ci metto ad arrivare in stazione"), "stazione");
        assert_eq!(dest("che strada faccio per arrivare alla stazione"), "stazione");
        assert_eq!(dest("mi indichi come arrivare al museo"), "museo");
    }
    #[test] fn it_trip_modifiers_are_dropped() {
        assert_eq!(dest("voglio andare alla stazione in macchina"), "stazione");
        assert_eq!(dest("voglio andare alla stazione adesso"), "stazione");
        assert_eq!(home("portami di nuovo a casa"), "casa");
        assert_eq!(dest("devo raggiungere la stazione entro un ora"), "stazione");
        assert_eq!(dest("portami alla stazione il prima possibile"), "stazione");
    }
    #[test] fn it_prima_is_filler_only_before_a_preposition() {
        assert_eq!(home("come arrivo prima a casa"), "casa");
        assert_eq!(dest("portami a prima porta"), "prima porta");
    }
    #[test] fn it_riportami() { assert_eq!(home("riportami a casa"), "casa"); }
}
