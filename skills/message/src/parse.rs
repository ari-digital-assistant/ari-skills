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
];

/// How many words a name may run to when the address book is doing the
/// matching. Four covers "Gail Marie van der Berg"; beyond that we're
/// swallowing the message.
const MAX_NAME_WORDS: usize = 4;

/// Phrases that separate a recipient from the message body. Longest first —
/// "that i am" must not match before " that ".
const BODY_MARKERS: &[&str] = &[" saying that ", " to say that ", " saying ", " to say ", " that "];

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
    for verb in ["reply ", "reply, "] {
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
    let mut take = words.len().min(MAX_NAME_WORDS);
    while take > 1 {
        let candidate = words[..take].join(" ");
        if is_contact(&candidate) {
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
    let lower = raw.to_lowercase();
    let anchor = anchor.trim();
    if anchor.is_empty() {
        return None;
    }
    let at = find_word(&lower, anchor)?;
    let rest = raw[at + anchor.len()..].trim();
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
    let lower = raw.to_lowercase();
    let first = recipient.split(' ').next()?;
    let at = find_word(&lower, first)?;
    let taken: Vec<&str> = raw[at..].split_whitespace().collect();
    let want = recipient.split(' ').filter(|w| !w.is_empty()).count();
    if taken.len() < want {
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
fn find_word(hay: &str, needle: &str) -> Option<usize> {
    let bytes = hay.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = hay[from..].find(needle) {
        let at = from + rel;
        let end = at + needle.len();
        let before_ok = at == 0 || !bytes[at - 1].is_ascii_alphanumeric();
        let after_ok = end >= bytes.len() || !bytes[end].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return Some(at);
        }
        from = at + 1;
        if from >= hay.len() {
            break;
        }
    }
    None
}

/// Trailing "on whatsapp" / "on telegram", removed before verb matching so
/// it never ends up inside the message body.
fn strip_trailing_service(s: &str) -> (String, Option<String>) {
    let lower = s.to_lowercase();
    for (id, words) in SERVICES {
        for word in *words {
            for prefix in [" on ", " over ", " via ", " through ", " using "] {
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
}
