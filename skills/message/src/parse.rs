//! Turns an utterance into a recipient, a body and (sometimes) a service.
//!
//! Two inputs matter and they are not interchangeable. The *normalised* text
//! is what we match on — lowercase, punctuation stripped, so the verb tables
//! below are written for it. The *raw* text is where the body comes from,
//! because the body is quoted to another human: normalisation turns "I'll be
//! home soon" into "i will be home soon", which is fine for matching and
//! embarrassing in somebody's chat window.
//!
//! Body and recipient are located in the raw text by **token search**, never
//! by word index. Contraction expansion means the normalised text can have
//! more words than the raw text, so an index taken from one and applied to
//! the other silently eats the start of the message.

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// A parsed request. `service` is only ever set when the user was explicit —
/// either by naming it ("on whatsapp") or by using it as a verb ("text",
/// "whatsapp"). The default is applied by the caller from settings, so the
/// parser never has to know what it is.
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    /// Empty only when [`explicit_reply`] is set and the user named nobody —
    /// "reply, on my way" means the newest live thread. Every other path
    /// rejects an empty recipient outright, so the two can't be confused.
    pub recipient: String,
    pub body: Option<String>,
    pub service: Option<String>,
    pub confidence: f32,
    /// The user said "reply" or "answer", so they mean an existing thread
    /// rather than a new message.
    pub explicit_reply: bool,
}

/// Service ids the parser recognises, with the words a user might say for
/// each. Kept in sync with the frontend's launcher registry by review, not
/// by code — an id the frontend doesn't know still reaches the user through
/// the share chooser, so drift costs a tap rather than a failure.
const SERVICES: &[(&str, &[&str])] = &[
    ("sms", &["sms", "text", "text message", "message", "messages"]),
    ("whatsapp", &["whatsapp"]),
    ("telegram", &["telegram"]),
    ("signal", &["signal"]),
    ("messenger", &["messenger", "facebook", "facebook messenger"]),
    ("slack", &["slack"]),
    ("matrix", &["matrix", "element"]),
    ("email", &["email", "e mail", "mail"]),
];

/// Words that can follow a verb but are never a person we can message.
/// "tell me a joke" is the one that matters — without this guard the skill
/// would steal general-knowledge questions from the assistant.
const NOT_A_RECIPIENT: &[&str] = &[
    // Pronouns — "tell me a joke" is the one that matters.
    "me", "us", "you", "him", "her", "them", "it", "everyone", "everybody",
    "someone", "somebody", "anyone", "anybody", "myself", "yourself",
    // Determiners — "tell the truth", "tell a story". Cheap to block here,
    // and `score()` can't consult the address book to know better: it runs
    // for every skill on every utterance, and reading contacts that often
    // would be both slow and creepy.
    "the", "a", "an", "my", "your", "our", "their", "his",
    // Italian pronouns — "dimmi" is clitic and never reaches here, but the
    // detached forms do: "di a lui che ...", "scrivi a tutti".
    "mi", "ti", "ci", "vi", "gli", "li", "ne", "te", "lui", "lei", "noi",
    "voi", "loro", "tutti", "tutte", "qualcuno", "nessuno", "chiunque",
    // Italian determiners. "lo", "la", "le" and "gli" are articles as well
    // as pronouns; one entry covers both readings.
    "il", "lo", "la", "i", "le", "un", "uno", "una", "mio", "mia", "tuo",
    "tua", "suo", "sua", "nostro", "nostra", "vostro", "vostra",
    // Italian interrogatives. These are the guard that stops a bare "di "
    // opener stealing questions: "di che colore è il cielo" parses its
    // recipient as "che" and dies here.
    "che", "cosa", "chi", "quando", "come", "dove", "quanto", "quale",
    "perche", "perché",
];

/// How many words a name may run to when the address book is doing the
/// matching. Four covers "Gail Marie van der Berg"; beyond that we're
/// swallowing the message.
const MAX_NAME_WORDS: usize = 4;

/// Phrases that separate a recipient from the message body. Longest first —
/// "that i am" must not match before " that ".
///
/// Italian shares the list rather than getting its own: the markers are
/// disjoint across the two languages, so a union match keeps every call site
/// locale-free. Same reasoning as the reminder skill's parser.
const BODY_MARKERS: &[&str] = &[
    " saying that ", " to say that ", " saying ", " to say ", " that ",
    " dicendo che ", " dicendo ", " che ",
];

/// Verbs that name their own service. "text gail" is an SMS, always, whatever
/// the default service setting says.
const SERVICE_VERBS: &[(&str, &str)] = &[
    // Longest first: "send an email to gail" must not fall through to the
    // service-less "send a message to" family and lose the service.
    ("send an email to ", "email"),
    ("send a email to ", "email"),
    ("send an e mail to ", "email"),
    ("send an sms to ", "sms"),
    ("send a text to ", "sms"),
    ("send text to ", "sms"),
    ("text ", "sms"),
    ("whatsapp ", "whatsapp"),
    ("telegram ", "telegram"),
    ("signal ", "signal"),
    ("slack ", "slack"),
    ("email ", "email"),
];

/// Openers where the recipient follows immediately.
const DIRECT_VERBS: &[&str] = &[
    "send a message to ",
    "send an message to ",
    "send message to ",
    "message ",
    "tell ",
];

// ── Italian ───────────────────────────────────────────────────────────────
//
// Italian gets its own tables rather than joining the English ones, for one
// reason: it puts a preposition between the verb and the name — "scrivi *a*
// Mario", "scrivi *alla* mamma". Stripping a leading preposition is what
// makes that work, and the English tables must never do it: "tell a story"
// would strip the "a" and send a message to "story".
//
// Requiring the preposition is also the poaching guard. `custom_score` means
// this parser scores every utterance for every locale, so a bare "di " or
// "scrivi " opener with no preposition slot would claim "di che colore è il
// cielo" and "scrivi una poesia". No preposition, no match.

/// Prepositions that can introduce the recipient, including the article
/// contractions. `all` is `all'` after the engine's elision strip turns the
/// apostrophe into a space ("all'avvocato" → "all avvocato").
const IT_PREPS: &[&str] = &["a", "ad", "al", "allo", "alla", "ai", "agli", "alle", "all"];

/// Italian openers that name their own service, e.g. "manda un sms a Gail".
/// Longest first so "manda un messaggio" can't swallow "manda un sms".
const IT_SERVICE_VERBS: &[(&str, &str)] = &[
    ("manda una email ", "email"),
    ("invia una email ", "email"),
    ("scrivi una email ", "email"),
    ("manda un email ", "email"),
    ("manda una mail ", "email"),
    ("invia una mail ", "email"),
    ("scrivi una mail ", "email"),
    ("manda un whatsapp ", "whatsapp"),
    ("invia un whatsapp ", "whatsapp"),
    ("manda un telegram ", "telegram"),
    ("manda un sms ", "sms"),
    ("invia un sms ", "sms"),
    ("scrivi un sms ", "sms"),
];

/// Italian openers that take a preposition and leave the service unsaid.
const IT_PREP_VERBS: &[&str] = &[
    "manda un messaggio ",
    "invia un messaggio ",
    "scrivi un messaggio ",
    "fai sapere ",
    "far sapere ",
    "scrivi ",
    "dici ",
    // "di' a Mario che ..." and "dì a Mario che ...". The engine's elision
    // strip flattens the first to "di a"; the accented form survives whole.
    "di ",
    "dì ",
];

/// Italian openers that take a direct object — "avvisa Gail", no preposition.
/// Kept apart from [`IT_PREP_VERBS`] because these verbs are specific enough
/// to stand without one; "scrivi" and "di" are not.
const IT_DIRECT_VERBS: &[&str] = &["avvisa ", "avverti ", "contatta "];

/// Strip a leading Italian preposition and return what follows. `None` when
/// the phrase doesn't start with one, which is how the Italian openers stay
/// off utterances that merely share their verb.
fn strip_it_prep(rest: &str) -> Option<&str> {
    for prep in IT_PREPS {
        if let Some(tail) = rest.strip_prefix(prep) {
            // The preposition must be a whole word: "alle" must not match
            // inside "allegato", and "a" must not match inside "andrea".
            if let Some(tail) = tail.strip_prefix(' ') {
                let tail = tail.trim_start();
                if !tail.is_empty() {
                    return Some(tail);
                }
            }
        }
    }
    None
}

/// Parse without an address book. Used by `score()`, which runs for every
/// skill on every utterance and must stay cheap — the recipient is taken as
/// the first word and refined later.
pub fn parse(normalized: &str, raw: &str) -> Option<Request> {
    parse_with(normalized, raw, |_| false)
}

/// Parse with a resolver that says whether a phrase names a real contact.
///
/// This is what makes bare multi-word names work: "tell gail marie i'll be
/// late" can only be split correctly by something that knows Gail Marie is
/// one person. Without it the parser has to guess, and guesses at the first
/// word.
pub fn parse_with(
    normalized: &str,
    raw: &str,
    is_contact: impl Fn(&str) -> bool + Copy,
) -> Option<Request> {
    let norm = normalized.trim();
    if norm.is_empty() {
        return None;
    }

    let (norm, service_from_suffix) = strip_trailing_service(norm);
    // The same suffix has to come off the raw text too, or "on WhatsApp"
    // survives into the message body — the user would send the routing
    // instruction to the person they were routing to.
    let (raw, _) = strip_trailing_service(raw.trim());
    let raw = raw.as_str();
    let norm = norm.as_str();

    // "reply to gail on my way" / "reply on my way". A bare reply names
    // nobody and means the newest live thread — the driving case, and the
    // whole reason this transport exists.
    for verb in ["reply to ", "answer ", "respond to "] {
        if let Some(rest) = norm.strip_prefix(verb) {
            let mut req = build_from_rest(rest, service_from_suffix, raw, 0.95, is_contact)?;
            req.explicit_reply = true;
            return Some(req);
        }
    }
    // "rispondi a Gail che arrivo". The preposition is required for the same
    // reason it is on every other Italian opener — without it "rispondi alla
    // domanda" and "rispondi che ore sono" would both land here.
    if let Some(rest) = norm.strip_prefix("rispondi ") {
        if let Some(rest) = strip_it_prep(rest) {
            let mut req = build_from_rest(rest, service_from_suffix, raw, 0.95, is_contact)?;
            req.explicit_reply = true;
            return Some(req);
        }
    }
    for verb in ["reply ", "reply, ", "rispondi ", "rispondi, "] {
        if norm.starts_with(verb) {
            let body = body_after(raw, verb.trim_end_matches([' ', ',']));
            return Some(Request {
                recipient: String::new(),
                body,
                service: service_from_suffix,
                confidence: 0.95,
                explicit_reply: true,
            });
        }
    }

    // "let gail know i am late" — recipient sits between two fixed words,
    // and the body starts after "know", not after the name.
    if let Some(rest) = norm.strip_prefix("let ") {
        if let Some((recipient, tail)) = split_once_str(rest, " know") {
            return build(recipient, tail, "know", service_from_suffix, raw, 0.95);
        }
    }

    // "send gail a message saying i am late" — recipient is in the middle.
    if let Some(rest) = norm.strip_prefix("send ") {
        for mid in [" a message", " a text", " an sms", " a whatsapp", " an email"] {
            if let Some((recipient, tail)) = split_once_str(rest, mid) {
                let anchor = mid.rsplit(' ').next().unwrap_or(mid);
                return build(recipient, tail, anchor, service_from_suffix, raw, 0.95);
            }
        }
    }

    for (verb, service) in SERVICE_VERBS {
        if let Some(rest) = norm.strip_prefix(verb) {
            let explicit = service_from_suffix.or(Some((*service).to_string()));
            return build_from_rest(rest, explicit, raw, 0.95, is_contact);
        }
    }

    for verb in DIRECT_VERBS {
        if let Some(rest) = norm.strip_prefix(verb) {
            // "tell" is the loose one — "tell me the time" must not land here.
            let confidence = if *verb == "tell " { 0.9 } else { 0.95 };
            return build_from_rest(rest, service_from_suffix, raw, confidence, is_contact);
        }
    }

    // ── Italian ──
    // Service-naming openers first: "manda un sms a Gail" must not fall
    // through to the service-less "manda un messaggio" family and lose it.
    for (verb, service) in IT_SERVICE_VERBS {
        if let Some(rest) = norm.strip_prefix(verb) {
            if let Some(rest) = strip_it_prep(rest) {
                let explicit = service_from_suffix.or(Some((*service).to_string()));
                return build_from_rest(rest, explicit, raw, 0.95, is_contact);
            }
            // The verb matched but the preposition didn't. Stop here rather
            // than falling through: "manda un sms" with no recipient is this
            // skill's problem to report, not another skill's to claim.
            return None;
        }
    }

    for verb in IT_PREP_VERBS {
        if let Some(rest) = norm.strip_prefix(verb) {
            if let Some(rest) = strip_it_prep(rest) {
                // "di" and "scrivi" are the loose ones, the way "tell" is in
                // English: they carry a message often enough to earn a match
                // and other things often enough not to win outright.
                let confidence = if *verb == "di " || *verb == "dì " || *verb == "scrivi " {
                    0.9
                } else {
                    0.95
                };
                return build_from_rest(rest, service_from_suffix, raw, confidence, is_contact);
            }
            return None;
        }
    }

    for verb in IT_DIRECT_VERBS {
        if let Some(rest) = norm.strip_prefix(verb) {
            return build_from_rest(rest, service_from_suffix, raw, 0.95, is_contact);
        }
    }

    None
}

/// Recipient is the first token of `rest`; the body is whatever follows it,
/// taken from the raw utterance.
fn build_from_rest(
    rest: &str,
    service: Option<String>,
    raw: &str,
    confidence: f32,
    is_contact: impl Fn(&str) -> bool,
) -> Option<Request> {
    // Longest match first, so "gail marie" wins over "gail" when both are in
    // the address book. Falls back to the first word when nothing matches —
    // the skill still says something useful about a name it can't find.
    let words: Vec<&str> = rest.split_whitespace().collect();
    let lower_raw = raw.to_lowercase();
    let mut take = words.len().min(MAX_NAME_WORDS);
    while take > 1 {
        let candidate = words[..take].join(" ");
        // The candidate has to cover whole words of the RAW text, not just
        // read plausibly in the normalised one. "I'm" normalises to "i am",
        // which makes "gail i" look like a two-word name — and a loose
        // contacts filter will match it against an Abigail — but its second
        // word is only part of the raw "I'm", so it cannot anchor the body.
        if is_contact(&candidate) && spans_whole_words(&candidate, &lower_raw) {
            let tail = rest[candidate.len().min(rest.len())..].trim_start();
            return build(&candidate, tail, &candidate, service, raw, confidence);
        }
        take -= 1;
    }
    let (recipient, tail) = match rest.split_once(' ') {
        Some((r, t)) => (r, t),
        None => (rest, ""),
    };
    build(recipient, tail, recipient, service, raw, confidence)
}

fn build(
    recipient: &str,
    tail: &str,
    anchor: &str,
    service: Option<String>,
    raw: &str,
    confidence: f32,
) -> Option<Request> {
    let recipient = recipient.trim();
    if recipient.is_empty() || NOT_A_RECIPIENT.contains(&recipient) {
        return None;
    }

    // A body marker lets a multi-word name through: everything before
    // "saying" is the recipient. Without contacts to check a name against,
    // this is the only reliable signal that "john smith" is one person.
    let (recipient, anchor) = match first_marker(tail) {
        Some((marker, idx)) => {
            let mut full = String::from(recipient);
            full.push(' ');
            full.push_str(&padded(tail)[..idx]);
            (trim_words(&full), marker.trim().to_string())
        }
        None => (recipient.to_string(), anchor.trim().to_string()),
    };
    if recipient.is_empty() || NOT_A_RECIPIENT.contains(&recipient.as_str()) {
        return None;
    }

    Some(Request {
        recipient: display_form(raw, &recipient).unwrap_or_else(|| recipient.clone()),
        body: body_after(raw, &anchor),
        service,
        confidence,
        explicit_reply: false,
    })
}

/// Markers carry a leading space so they can't match mid-word, but a tail
/// often *starts* with one ("gail saying …" → tail "saying …"). Padding makes
/// both positions look the same to the search.
fn padded(tail: &str) -> String {
    let mut s = String::from(" ");
    s.push_str(tail.trim());
    s
}

fn first_marker(tail: &str) -> Option<(&'static str, usize)> {
    let hay = padded(tail);
    BODY_MARKERS
        .iter()
        .filter_map(|m| hay.find(m).map(|i| (*m, i)))
        .min_by_key(|(_, i)| *i)
}

/// The message body, lifted from the raw utterance so capitals, apostrophes
/// and contractions survive. `anchor` is the last token before the body —
/// found by search rather than by index, see the module note.
fn body_after(raw: &str, anchor: &str) -> Option<String> {
    let anchor = anchor.trim();
    if anchor.is_empty() {
        return None;
    }
    // Whole words, not a literal find: an anchor of "o brien" has to clear
    // the raw "O'Brien", and "gail i" must not clear part of "I'm".
    let at = end_of_words(raw, anchor)?;
    let rest = raw[at..].trim();
    let rest = rest.trim_start_matches(|c: char| c == ',' || c == ':');
    let rest = rest.trim();
    if rest.is_empty() {
        None
    } else {
        Some(rest.to_string())
    }
}

/// The recipient as the user actually said it, so "Mario" is spoken back as
/// "Mario" rather than "mario".
fn display_form(raw: &str, recipient: &str) -> Option<String> {
    let first = alnum_lower(recipient.split(' ').next()?);
    let letters = alnum_lower(recipient);
    if first.is_empty() {
        return None;
    }
    let tokens: Vec<&str> = raw.split_whitespace().collect();
    // Letters only, and a prefix match: normalisation splits "O'Brien" into
    // "o brien", so the raw token a name starts in is often longer than the
    // word being looked for.
    let start = tokens.iter().position(|t| alnum_lower(t).starts_with(&first))?;
    // Take raw tokens until their letters cover the recipient's. Counting
    // recipient words instead would take two tokens for "o brien" and drag
    // the next word of the sentence into the name.
    let mut covered = String::new();
    let mut want = 0usize;
    for token in &tokens[start..] {
        covered.push_str(&alnum_lower(token));
        want += 1;
        if covered.len() >= letters.len() {
            break;
        }
    }
    let taken = &tokens[start..];
    if want == 0 || taken.len() < want {
        return None;
    }
    // "Send an email to Gail, remember the milk" — the raw text carries
    // punctuation the normalised text doesn't, and a trailing comma would
    // ride along into "That's ready for Gail, — just tap send."
    Some(
        taken[..want]
            .iter()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-'))
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
    )
}

/// Byte offset of `needle` in `hay` at a word boundary, so "sam" doesn't
/// match inside "same".
/// Letters and digits only, lowercased — the form in which a normalised
/// candidate and the raw text can be compared without punctuation getting in
/// the way.
fn alnum_lower(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(|c| c.to_lowercase()).collect()
}

/// Byte offset in `raw` just past the run of whole words that `phrase`
/// covers, or `None` if it covers no such run.
///
/// Comparing letters only, but insisting the run both starts and ends on a
/// word boundary, is what separates the two cases that matter. Normalisation
/// turns an apostrophe into a space, so "O'Brien" reaches the parser as
/// "o brien" — two words spanning one raw word, and legitimate. "gail i" is
/// not: its second word is a fragment of the raw "I'm", so no run of whole
/// words matches and the body is never cut mid-word.
fn end_of_words(raw: &str, phrase: &str) -> Option<usize> {
    let want = alnum_lower(phrase);
    if want.is_empty() {
        return None;
    }
    // split_whitespace drops the offsets, and we need them to slice the body
    // out afterwards. Each token is a subslice of `raw`, so the difference
    // between their pointers is its byte offset.
    let tokens: Vec<(usize, &str)> = raw
        .split_whitespace()
        .map(|t| (t.as_ptr() as usize - raw.as_ptr() as usize, t))
        .collect();
    for start in 0..tokens.len() {
        let mut acc = String::new();
        for (offset, token) in &tokens[start..] {
            acc.push_str(&alnum_lower(token));
            if acc == want {
                return Some(offset + token.len());
            }
            if !want.starts_with(&acc) {
                break;
            }
        }
    }
    None
}

fn spans_whole_words(candidate: &str, raw: &str) -> bool {
    end_of_words(raw, candidate).is_some()
}

/// Trailing "on whatsapp" / "on telegram", removed before verb matching so
/// it never ends up inside the message body.
fn strip_trailing_service(s: &str) -> (String, Option<String>) {
    let lower = s.to_lowercase();
    for (id, words) in SERVICES {
        for word in *words {
            // Italian: " su ", " con ", " tramite ", " per ". Each is only
            // ever consulted immediately before a known service word, so
            // even the loose ones can't eat an ordinary tail.
            for prefix in [
                " on ", " over ", " via ", " through ", " using ",
                " su ", " con ", " tramite ", " per ",
            ] {
                let mut suffix = String::from(prefix);
                suffix.push_str(word);
                if lower.ends_with(suffix.as_str()) {
                    let head = s[..s.len() - suffix.len()].trim_end();
                    return (head.to_string(), Some((*id).to_string()));
                }
            }
        }
    }
    (s.to_string(), None)
}

fn split_once_str<'a>(s: &'a str, sep: &str) -> Option<(&'a str, &'a str)> {
    let at = s.find(sep)?;
    Some((&s[..at], &s[at + sep.len()..]))
}

fn trim_words(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(raw: &str, normalized: &str) -> Option<Request> {
        parse(normalized, raw)
    }

    #[test]
    fn tell_with_a_name_is_a_message() {
        let r = p("Tell Mario I'll be home soon", "tell mario i will be home soon").unwrap();
        assert_eq!(r.recipient, "Mario");
        assert_eq!(r.body.as_deref(), Some("I'll be home soon"));
        assert_eq!(r.service, None);
    }

    #[test]
    fn tell_me_is_never_a_message() {
        // The whole reason this skill scores itself instead of using regex
        // patterns: Rust's regex crate has no negative lookahead.
        assert!(p("Tell me a joke", "tell me a joke").is_none());
        assert!(p("Tell me the time", "tell me the time").is_none());
        assert!(p("Tell me about Malta", "tell me about malta").is_none());
    }

    #[test]
    fn message_verb_takes_the_first_word_as_recipient() {
        let r = p("Message Gail on my way", "message gail on my way").unwrap();
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.body.as_deref(), Some("on my way"));
    }

    #[test]
    fn text_verb_forces_sms() {
        let r = p("Text Gail I'm late", "text gail i am late").unwrap();
        assert_eq!(r.service.as_deref(), Some("sms"));
        assert_eq!(r.body.as_deref(), Some("I'm late"));
    }

    #[test]
    fn service_as_verb_names_the_service() {
        let r = p("WhatsApp Mario see you at 8", "whatsapp mario see you at 8").unwrap();
        assert_eq!(r.service.as_deref(), Some("whatsapp"));
        assert_eq!(r.recipient, "Mario");
        assert_eq!(r.body.as_deref(), Some("see you at 8"));
    }

    #[test]
    fn trailing_service_is_stripped_from_the_body() {
        let r = p(
            "Tell Mario I'll be home soon on WhatsApp",
            "tell mario i will be home soon on whatsapp",
        )
        .unwrap();
        assert_eq!(r.service.as_deref(), Some("whatsapp"));
        assert_eq!(
            r.body.as_deref(),
            Some("I'll be home soon"),
            "the service must not survive inside the message",
        );
    }

    #[test]
    fn explicit_service_beats_the_verbs_own_service() {
        let r = p("Text Gail hello on Telegram", "text gail hello on telegram").unwrap();
        assert_eq!(r.service.as_deref(), Some("telegram"));
    }

    #[test]
    fn send_a_message_to_form() {
        let r = p(
            "Send a message to Gail saying I'll be late",
            "send a message to gail saying i will be late",
        )
        .unwrap();
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.body.as_deref(), Some("I'll be late"));
    }

    #[test]
    fn send_recipient_a_message_form() {
        let r = p(
            "Send Gail a message saying I'm on my way",
            "send gail a message saying i am on my way",
        )
        .unwrap();
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.body.as_deref(), Some("I'm on my way"));
    }

    #[test]
    fn let_x_know_form() {
        let r = p("Let Gail know I'm running late", "let gail know i am running late").unwrap();
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.body.as_deref(), Some("I'm running late"));
    }

    #[test]
    fn a_body_marker_rescues_a_two_word_name() {
        let r = p(
            "Send a message to Gail Marie saying I'll be late",
            "send a message to gail marie saying i will be late",
        )
        .unwrap();
        assert_eq!(r.recipient, "Gail Marie");
        assert_eq!(r.body.as_deref(), Some("I'll be late"));
    }

    #[test]
    fn no_body_is_allowed_so_the_skill_can_ask() {
        let r = p("Message Gail", "message gail").unwrap();
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.body, None);
    }

    #[test]
    fn body_keeps_capitals_and_apostrophes_from_the_raw_utterance() {
        // The point of ari::raw_input(). Normalised, this body would reach
        // the recipient as "i will be home soon".
        let r = p("Tell Mario I'll be home soon", "tell mario i will be home soon").unwrap();
        assert!(r.body.as_deref().unwrap().starts_with("I'll"));
    }

    #[test]
    fn recipient_search_respects_word_boundaries() {
        // "sam" must not match inside "same".
        let r = p("Tell Sam same time tomorrow", "tell sam same time tomorrow").unwrap();
        assert_eq!(r.recipient, "Sam");
        assert_eq!(r.body.as_deref(), Some("same time tomorrow"));
    }

    #[test]
    fn unrelated_utterances_do_not_match() {
        assert!(p("What is the weather", "what is the weather").is_none());
        assert!(p("Set a timer for 5 minutes", "set a timer for 5 minutes").is_none());
        assert!(p("", "").is_none());
    }

    #[test]
    fn tell_scores_lower_than_the_unambiguous_verbs() {
        let loose = p("Tell Mario hello", "tell mario hello").unwrap();
        let tight = p("Message Mario hello", "message mario hello").unwrap();
        assert!(loose.confidence < tight.confidence);
    }

    #[test]
    fn a_known_multi_word_name_beats_the_first_word() {
        // The bare form, with no "saying" to split on — only the address
        // book can tell that "gail marie" is one person.
        let r = parse_with(
            "tell gail marie i will be late",
            "Tell Gail Marie I'll be late",
            |name| name == "gail marie",
        )
        .unwrap();
        assert_eq!(r.recipient, "Gail Marie");
        assert_eq!(r.body.as_deref(), Some("I'll be late"));
    }

    #[test]
    fn the_longest_matching_name_wins() {
        // Both are real contacts; the longer one is what the user said.
        let r = parse_with(
            "tell gail marie hello",
            "Tell Gail Marie hello",
            |name| name == "gail" || name == "gail marie",
        )
        .unwrap();
        assert_eq!(r.recipient, "Gail Marie");
        assert_eq!(r.body.as_deref(), Some("hello"));
    }

    #[test]
    fn an_unknown_name_still_parses_as_one_word() {
        // Nothing matched, so the skill can still say "I couldn't find Gail"
        // rather than refusing to understand the sentence at all.
        let r = parse_with("tell gail hello", "Tell Gail hello", |_| false).unwrap();
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.body.as_deref(), Some("hello"));
    }

    #[test]
    fn name_matching_stops_before_swallowing_the_message() {
        // A resolver that says yes to everything must not eat the body.
        let r = parse_with(
            "tell gail i am going to be a bit late tonight",
            "Tell Gail I am going to be a bit late tonight",
            |_| true,
        )
        .unwrap();
        assert!(
            r.recipient.split_whitespace().count() <= 4,
            "recipient ran away: {}",
            r.recipient,
        );
        assert!(r.body.is_some());
    }

    #[test]
    fn determiners_are_not_recipients() {
        // score() has no address book, so these are blocked by name.
        assert!(parse("tell the truth", "Tell the truth").is_none());
        assert!(parse("tell a story", "Tell a story").is_none());
    }

    #[test]
    fn send_an_email_to_form() {
        let r = p(
            "Send an email to Gail, remember the milk honey",
            "send an email to gail remember the milk honey",
        )
        .unwrap();
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.service.as_deref(), Some("email"));
        assert_eq!(r.body.as_deref(), Some("remember the milk honey"));
    }

    #[test]
    fn punctuation_after_a_name_is_not_part_of_it() {
        let r = p("Tell Gail, I am on my way", "tell gail i am on my way").unwrap();
        assert_eq!(r.recipient, "Gail");
    }

    #[test]
    fn a_contraction_is_not_part_of_the_name() {
        // "tell gail I'm on the way" went out as "'m on the way". The engine
        // normalises "I'm" to "i am", which offers "gail i" as a two-word
        // name; a loose contacts filter matches it against an Abigail, and
        // the body was then cut from the middle of the raw "I'm".
        let r = parse_with("tell gail i am on the way", "tell gail I'm on the way", |n| {
            n == "gail" || n == "gail i"
        })
        .unwrap();
        assert_eq!(r.recipient, "gail");
        assert_eq!(r.body.as_deref(), Some("I'm on the way"));
    }

    #[test]
    fn an_anchor_will_not_clear_part_of_a_contraction() {
        let raw = "tell gail i'm on the way";
        assert_eq!(end_of_words(raw, "gail i"), None);
        assert_eq!(end_of_words(raw, "gail").map(|i| raw[i..].trim()), Some("i'm on the way"));
    }

    #[test]
    fn an_anchor_clears_an_apostrophe_the_normaliser_split() {
        let raw = "tell O'Brien hello there";
        assert_eq!(end_of_words(raw, "o brien").map(|i| raw[i..].trim()), Some("hello there"));
    }

    #[test]
    fn anchors_work_on_non_ascii_names() {
        // Byte-stepping used to risk landing inside a multi-byte character.
        let raw = "dì a Sofía ciao";
        assert_eq!(end_of_words(raw, "sofía").map(|i| raw[i..].trim()), Some("ciao"));
        assert_eq!(end_of_words(raw, "sofi"), None);
    }

    #[test]
    fn a_candidate_must_cover_whole_words_of_the_raw_text() {
        // "o brien" spans the single raw word "O'Brien" — legitimate.
        assert!(spans_whole_words("o brien", "tell o'brien hello"));
        // "gail i" only covers part of "I'm" — not something the user said.
        assert!(!spans_whole_words("gail i", "tell gail i'm on the way"));
        assert!(spans_whole_words("gail", "tell gail i'm on the way"));
    }

    #[test]
    fn a_two_word_name_spanning_one_raw_word_does_not_eat_the_sentence() {
        // Counting recipient words rather than raw ones took "O'Brien hello".
        let r = parse_with("tell o brien hello there", "tell O'Brien hello there", |n| {
            n == "o brien"
        })
        .unwrap();
        assert_eq!(r.recipient, "O'Brien");
        assert_eq!(r.body.as_deref(), Some("hello there"));
    }

    #[test]
    fn a_name_keeps_its_own_punctuation() {
        // O'Brien and Anne-Marie are names, not names with punctuation.
        let r = p("Tell O'Brien hello", "tell o brien hello").unwrap();
        assert_eq!(r.recipient, "O'Brien");
        let r = p("Tell Anne-Marie hello", "tell anne-marie hello").unwrap();
        assert_eq!(r.recipient, "Anne-Marie");
    }

    #[test]
    fn email_as_a_verb_names_the_service() {
        let r = p("Email Gail the invoice is attached", "email gail the invoice is attached")
            .unwrap();
        assert_eq!(r.service.as_deref(), Some("email"));
        assert_eq!(r.recipient, "Gail");
    }

    #[test]
    fn reply_to_a_name_targets_that_thread() {
        let r = p("Reply to Gail I'm on my way", "reply to gail i am on my way").unwrap();
        assert!(r.explicit_reply);
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.body.as_deref(), Some("I'm on my way"));
    }

    #[test]
    fn a_bare_reply_names_nobody_and_means_the_newest_thread() {
        // The hands-free case: a message arrives while driving and the answer
        // is one sentence with no name in it.
        let r = p("Reply on my way", "reply on my way").unwrap();
        assert!(r.explicit_reply);
        assert_eq!(r.recipient, "");
        assert_eq!(r.body.as_deref(), Some("on my way"));
    }

    #[test]
    fn answer_is_the_same_as_reply_to() {
        let r = p("Answer Mario five minutes", "answer mario five minutes").unwrap();
        assert!(r.explicit_reply);
        assert_eq!(r.recipient, "Mario");
    }

    #[test]
    fn an_ordinary_message_is_not_flagged_as_a_reply() {
        assert!(!p("Tell Gail hello", "tell gail hello").unwrap().explicit_reply);
    }

    #[test]
    fn an_unknown_service_is_left_for_the_chooser() {
        // We only recognise a handful of names, and the world has hundreds.
        // An unrecognised one must not become a service id we then fail to
        // launch — no service means the frontend offers the share chooser.
        let r = p("Tell Mario hello on KakaoTalk", "tell mario hello on kakaotalk").unwrap();
        assert_eq!(r.recipient, "Mario");
        assert_eq!(r.service, None);
    }

    // ── Italian ──────────────────────────────────────────────────────────
    //
    // The normalised text in each of these is what the engine's Italian
    // pipeline actually produces: lowercased, apostrophes turned into
    // spaces by the elision strip ("di'" → "di", "all'" → "all").

    #[test]
    fn it_scrivi_with_a_preposition_is_a_message() {
        let r = p("Scrivi a Mario che arrivo tardi", "scrivi a mario che arrivo tardi").unwrap();
        assert_eq!(r.recipient, "Mario");
        assert_eq!(r.body.as_deref(), Some("arrivo tardi"));
        assert_eq!(r.service, None);
    }

    #[test]
    fn it_article_contractions_introduce_the_recipient() {
        // "alla mamma", not "a mamma" — the contracted preposition is the
        // normal Italian form and has to reach the same place.
        let r = p("Scrivi alla mamma che torno presto", "scrivi alla mamma che torno presto").unwrap();
        assert_eq!(r.recipient, "mamma");
        assert_eq!(r.body.as_deref(), Some("torno presto"));
    }

    #[test]
    fn it_elided_preposition_survives_the_normaliser() {
        // "all'avvocato" reaches the parser as "all avvocato".
        let r = p("Scrivi all'avvocato che richiamo", "scrivi all avvocato che richiamo").unwrap();
        assert_eq!(r.recipient, "avvocato");
        assert_eq!(r.body.as_deref(), Some("richiamo"));
    }

    #[test]
    fn it_manda_un_messaggio_form() {
        let r = p(
            "Manda un messaggio a Gail dicendo che sono in ritardo",
            "manda un messaggio a gail dicendo che sono in ritardo",
        )
        .unwrap();
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.body.as_deref(), Some("sono in ritardo"));
    }

    #[test]
    fn it_service_verb_names_the_service() {
        let r = p("Manda un sms a Gail ci vediamo alle 8", "manda un sms a gail ci vediamo alle 8").unwrap();
        assert_eq!(r.service.as_deref(), Some("sms"));
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.body.as_deref(), Some("ci vediamo alle 8"));
    }

    #[test]
    fn it_email_verb_names_the_service() {
        let r = p(
            "Manda una mail a Gail ricordati del latte",
            "manda una mail a gail ricordati del latte",
        )
        .unwrap();
        assert_eq!(r.service.as_deref(), Some("email"));
        assert_eq!(r.body.as_deref(), Some("ricordati del latte"));
    }

    #[test]
    fn it_service_verb_beats_the_generic_messaggio_form() {
        // "manda un sms" must not fall through to "manda un messaggio" and
        // arrive with no service at all.
        let r = p("Manda un sms a Mario ciao", "manda un sms a mario ciao").unwrap();
        assert_eq!(r.service.as_deref(), Some("sms"));
    }

    #[test]
    fn it_avvisa_takes_a_direct_object() {
        // "avvisare qualcuno" — no preposition, unlike every other opener.
        let r = p("Avvisa Gail che faccio tardi", "avvisa gail che faccio tardi").unwrap();
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.body.as_deref(), Some("faccio tardi"));
    }

    #[test]
    fn it_fai_sapere_form() {
        let r = p(
            "Fai sapere a Gail che sono sull'autobus",
            "fai sapere a gail che sono sull autobus",
        )
        .unwrap();
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.body.as_deref(), Some("sono sull'autobus"));
    }

    #[test]
    fn it_di_with_an_apostrophe_is_a_message() {
        // "Di' a Mario ..." — the elision strip flattens it to "di a mario".
        let r = p("Di' a Mario che esco ora", "di a mario che esco ora").unwrap();
        assert_eq!(r.recipient, "Mario");
        assert_eq!(r.body.as_deref(), Some("esco ora"));
    }

    #[test]
    fn it_accented_di_is_a_message() {
        let r = p("Dì a Mario che esco ora", "dì a mario che esco ora").unwrap();
        assert_eq!(r.recipient, "Mario");
        assert_eq!(r.body.as_deref(), Some("esco ora"));
    }

    #[test]
    fn it_a_verb_without_its_preposition_never_matches() {
        // The poaching guard. `custom_score` runs this parser on every
        // utterance in every locale, so these must all come back empty.
        assert!(p("Di che colore è il cielo", "di che colore è il cielo").is_none());
        assert!(p("Dimmi che ore sono", "dimmi che ore sono").is_none());
        assert!(p("Scrivi una poesia", "scrivi una poesia").is_none());
        assert!(p("Scrivi un promemoria", "scrivi un promemoria").is_none());
    }

    #[test]
    fn it_the_preposition_must_be_a_whole_word() {
        // "a" must not match inside "andrea", "alle" not inside "allegato".
        assert!(p("Scrivi andrea ciao", "scrivi andrea ciao").is_none());
    }

    #[test]
    fn it_trailing_service_is_stripped_from_the_body() {
        let r = p(
            "Scrivi a Mario che sono fuori su Telegram",
            "scrivi a mario che sono fuori su telegram",
        )
        .unwrap();
        assert_eq!(r.service.as_deref(), Some("telegram"));
        assert_eq!(
            r.body.as_deref(),
            Some("sono fuori"),
            "the service must not survive inside the message",
        );
    }

    #[test]
    fn it_tramite_also_names_a_service() {
        let r = p(
            "Manda un messaggio a Gail buon compleanno tramite WhatsApp",
            "manda un messaggio a gail buon compleanno tramite whatsapp",
        )
        .unwrap();
        assert_eq!(r.service.as_deref(), Some("whatsapp"));
        assert_eq!(r.body.as_deref(), Some("buon compleanno"));
    }

    #[test]
    fn it_explicit_service_beats_the_verbs_own_service() {
        let r = p(
            "Manda un sms a Gail ciao su Telegram",
            "manda un sms a gail ciao su telegram",
        )
        .unwrap();
        assert_eq!(r.service.as_deref(), Some("telegram"));
    }

    #[test]
    fn it_rispondi_with_a_name_is_a_reply() {
        let r = p("Rispondi a Gail arrivo", "rispondi a gail arrivo").unwrap();
        assert!(r.explicit_reply);
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.body.as_deref(), Some("arrivo"));
    }

    #[test]
    fn it_bare_rispondi_names_nobody() {
        let r = p("Rispondi sto arrivando", "rispondi sto arrivando").unwrap();
        assert!(r.explicit_reply);
        assert_eq!(r.recipient, "");
        assert_eq!(r.body.as_deref(), Some("sto arrivando"));
    }

    #[test]
    fn it_no_body_is_allowed_so_the_skill_can_ask() {
        let r = p("Scrivi a Gail", "scrivi a gail").unwrap();
        assert_eq!(r.recipient, "Gail");
        assert_eq!(r.body, None);
    }

    #[test]
    fn it_body_keeps_capitals_and_accents_from_the_raw_utterance() {
        let r = p(
            "Scrivi a Mario che è già partito",
            "scrivi a mario che è già partito",
        )
        .unwrap();
        assert_eq!(r.body.as_deref(), Some("è già partito"));
    }

    #[test]
    fn it_pronouns_are_never_a_recipient() {
        // The Italian half of "tell me a joke".
        assert!(p("Di a tutti che ho finito", "di a tutti che ho finito").is_none());
        assert!(p("Scrivi a chiunque", "scrivi a chiunque").is_none());
    }

    #[test]
    fn it_a_known_multi_word_name_beats_the_first_word() {
        let r = parse_with(
            "scrivi a gail marie arrivo tardi",
            "Scrivi a Gail Marie arrivo tardi",
            |name| name == "gail marie",
        )
        .unwrap();
        assert_eq!(r.recipient, "Gail Marie");
        assert_eq!(r.body.as_deref(), Some("arrivo tardi"));
    }

    #[test]
    fn it_loose_verbs_score_lower_than_the_unambiguous_ones() {
        let loose = p("Scrivi a Mario ciao", "scrivi a mario ciao").unwrap();
        let tight = p("Manda un messaggio a Mario ciao", "manda un messaggio a mario ciao").unwrap();
        assert!(loose.confidence < tight.confidence);
    }
}
