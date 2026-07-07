extern crate alloc;
use alloc::string::String;

#[derive(Debug, PartialEq, Eq)]
pub enum Transport {
    Pause,
    Resume,
    Next,
    Previous,
    Stop,
    VolumeUp,
    VolumeDown,
    VolumeSet(u8),
    Mute,
    Unmute,
}

/// Ordered longest-first within each group so multi-word triggers win.
/// Matched against fully-normalised input (lowercase, no apostrophes).
pub fn parse(input: &str) -> Option<Transport> {
    let s = input.trim();

    // Volume set: "set volume [to] N[%]" / "imposta|metti il volume al N[%]".
    if let Some(level) = parse_volume_level(s) {
        return Some(Transport::VolumeSet(level));
    }

    // Exact-or-substring phrase groups. Order matters: check unmute before
    // mute ("togli il muto" contains "muto"), volume down/up before bare.
    const UNMUTE: &[&str] = &["unmute", "togli il muto", "riattiva l audio", "riattiva audio"];
    const MUTE: &[&str] = &["mute", "muto", "silenzia"];
    const VOL_UP: &[&str] = &["volume up", "louder", "alza il volume", "piu forte"];
    const VOL_DOWN: &[&str] = &["volume down", "quieter", "abbassa il volume", "piu piano"];
    const PAUSE: &[&str] = &["metti in pausa", "pause", "pausa"];
    const RESUME: &[&str] = &["riprendi la musica", "riprendi", "resume"];
    const NEXT: &[&str] = &["next", "skip", "prossima", "successiva", "avanti", "salta"];
    const PREVIOUS: &[&str] = &["previous", "back", "precedente", "torna indietro"];
    const STOP: &[&str] = &["stop", "ferma la musica", "ferma"];

    let matches = |group: &[&str]| group.iter().any(|t| phrase_hit(s, t));

    if matches(UNMUTE) { return Some(Transport::Unmute); }
    if matches(MUTE) { return Some(Transport::Mute); }
    if matches(VOL_UP) { return Some(Transport::VolumeUp); }
    if matches(VOL_DOWN) { return Some(Transport::VolumeDown); }
    if matches(PAUSE) { return Some(Transport::Pause); }
    if matches(RESUME) { return Some(Transport::Resume); }
    if matches(NEXT) { return Some(Transport::Next); }
    if matches(PREVIOUS) { return Some(Transport::Previous); }
    if matches(STOP) { return Some(Transport::Stop); }
    None
}

/// Whole-phrase match: the input equals the trigger, or contains it as a
/// space-delimited run. Prevents "play" matching inside "player" and keeps
/// short triggers ("back", "avanti") from firing mid-word.
fn phrase_hit(input: &str, trigger: &str) -> bool {
    if input == trigger {
        return true;
    }
    let bytes = input.as_bytes();
    let mut from = 0;
    while let Some(pos) = input[from..].find(trigger) {
        let abs = from + pos;
        let end = abs + trigger.len();
        let start_ok = abs == 0 || bytes[abs - 1] == b' ';
        let end_ok = end == input.len() || bytes[end] == b' ';
        if start_ok && end_ok {
            return true;
        }
        from = abs + 1;
    }
    false
}

/// Extracts N from "set volume [to] N[%]" / "imposta|metti il volume al N[%]".
/// Clamps to 0..=100. Returns None when no volume-set phrasing is present.
fn parse_volume_level(s: &str) -> Option<u8> {
    const PREFIXES: &[&str] = &[
        "set volume to ", "set volume ",
        "imposta il volume al ", "imposta il volume a ",
        "metti il volume al ", "metti il volume a ",
    ];
    for p in PREFIXES {
        if let Some(rest) = s.strip_prefix(p) {
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if digits.is_empty() {
                continue;
            }
            let n: u32 = digits.parse().ok()?;
            return Some(n.min(100) as u8);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn en_core_transport() {
        assert_eq!(parse("pause"), Some(Transport::Pause));
        assert_eq!(parse("resume"), Some(Transport::Resume));
        assert_eq!(parse("next"), Some(Transport::Next));
        assert_eq!(parse("skip"), Some(Transport::Next));
        assert_eq!(parse("previous"), Some(Transport::Previous));
        assert_eq!(parse("back"), Some(Transport::Previous));
        assert_eq!(parse("stop"), Some(Transport::Stop));
    }

    #[test]
    fn en_volume() {
        assert_eq!(parse("volume up"), Some(Transport::VolumeUp));
        assert_eq!(parse("louder"), Some(Transport::VolumeUp));
        assert_eq!(parse("volume down"), Some(Transport::VolumeDown));
        assert_eq!(parse("quieter"), Some(Transport::VolumeDown));
        assert_eq!(parse("mute"), Some(Transport::Mute));
        assert_eq!(parse("unmute"), Some(Transport::Unmute));
        assert_eq!(parse("set volume to 50%"), Some(Transport::VolumeSet(50)));
        assert_eq!(parse("set volume 30"), Some(Transport::VolumeSet(30)));
        assert_eq!(parse("set volume to 100%"), Some(Transport::VolumeSet(100)));
    }

    #[test]
    fn it_triggers() {
        assert_eq!(parse("pausa"), Some(Transport::Pause));
        assert_eq!(parse("metti in pausa"), Some(Transport::Pause));
        assert_eq!(parse("riprendi"), Some(Transport::Resume));
        assert_eq!(parse("prossima"), Some(Transport::Next));
        assert_eq!(parse("successiva"), Some(Transport::Next));
        assert_eq!(parse("avanti"), Some(Transport::Next));
        assert_eq!(parse("salta"), Some(Transport::Next));
        assert_eq!(parse("precedente"), Some(Transport::Previous));
        assert_eq!(parse("torna indietro"), Some(Transport::Previous));
        assert_eq!(parse("ferma la musica"), Some(Transport::Stop));
        assert_eq!(parse("alza il volume"), Some(Transport::VolumeUp));
        assert_eq!(parse("piu forte"), Some(Transport::VolumeUp));
        assert_eq!(parse("abbassa il volume"), Some(Transport::VolumeDown));
        assert_eq!(parse("piu piano"), Some(Transport::VolumeDown));
        assert_eq!(parse("muto"), Some(Transport::Mute));
        assert_eq!(parse("silenzia"), Some(Transport::Mute));
        assert_eq!(parse("togli il muto"), Some(Transport::Unmute));
        assert_eq!(parse("imposta il volume al 40%"), Some(Transport::VolumeSet(40)));
    }

    #[test]
    fn non_transport_returns_none() {
        // A play request must NOT be swallowed as transport.
        assert_eq!(parse("play hotel california"), None);
        assert_eq!(parse("metti hotel california su spotify"), None);
        assert_eq!(parse("what time is it"), None);
    }

    #[test]
    fn volume_set_clamps_over_100() {
        assert_eq!(parse("set volume to 250%"), Some(Transport::VolumeSet(100)));
    }
}
