//! Ari message skill.
//!
//! One skill for every service, not one per service. "Tell Mario I'll be home
//! soon", "text Gail", "WhatsApp Sam see you at 8" all land here; the service
//! is a parameter, and the only thing that varies is whether Ari can send it
//! itself or has to hand it to another app.
//!
//! That distinction is the whole design:
//!
//!  - **Send** — Ari does it, nobody else is in the loop, so it reads the
//!    message back and asks first. A wrong contact match is unrecallable.
//!  - **Compose** — the message is handed to another app with the body filled
//!    in, and the user taps send. That tap *is* the confirmation, so asking
//!    first would be a wasted turn on a voice frontend.
//!
//! Scoring is custom because "tell" needs a negative match — "tell me a joke"
//! must not land here — and Rust's `regex` crate has no lookaround, by design.
//! See `parse::NOT_A_RECIPIENT`.

#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

#[cfg(target_arch = "wasm32")]
use alloc::string::{String, ToString};
#[cfg(target_arch = "wasm32")]
use alloc::vec::Vec;

mod action;
mod matrix;
mod parse;
mod transport;

#[cfg(target_arch = "wasm32")]
use ari_skill_sdk as ari;

/// Services that can go without anybody tapping anything, so the message
/// reaches another person with no human in the loop. These — and only these —
/// get read back and confirmed first.
///
/// Matrix and Slack join this list when they land; they need `http`.
const TRUE_SEND: &[&str] = &["sms", "matrix"];

/// Fallback when the user names no service and hasn't set one. SMS matches
/// what Siri and Google do; the setting exists because SMS is near-dead in
/// much of Europe and normal in the US.
const DEFAULT_SERVICE: &str = "sms";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// Hand to another app. No confirmation — the user's tap is the confirm.
    Compose,
    /// Ari sends it. Read it back first.
    Confirm,
    /// Ari sends it without asking, because the user turned confirmation off.
    Send,
}

/// What to do with a fully-formed request. Pure so both branches are testable
/// before a transport exists to reach them.
pub fn plan(service: &str, confirm: bool, true_send: &[&str]) -> Plan {
    if !true_send.contains(&service) {
        return Plan::Compose;
    }
    if confirm {
        Plan::Confirm
    } else {
        Plan::Send
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(ptr: i32, len: i32) -> f32 {
    let input = unsafe { ari::input(ptr, len) };
    // Matching runs on normalised text by contract, and `raw_input()` is
    // None here anyway — pass the same string for both.
    parse::parse(input, input).map(|r| r.confidence).unwrap_or(0.0)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    ari::respond_action(&dispatch(input))
}

#[cfg(target_arch = "wasm32")]
fn dispatch(input: &str) -> String {
    if let Some(reply) = ari::parse_reply(input) {
        return handle_reply(&reply.context, &reply.text);
    }

    // The body is quoted to another person, so it comes from what the user
    // actually said. Falling back to the normalised text keeps the skill
    // working on a host that predates `raw_input`, just less prettily.
    let raw = ari::raw_input().unwrap_or(input);
    // The address book decides where the name ends: "tell gail marie i'll be
    // late" can only be split by something that knows Gail Marie is one
    // person. `score()` deliberately does none of this.
    let Some(req) = parse::parse_with(input, raw, |name| !ari::contacts_lookup(name).is_empty())
    else {
        return action::say_json(&t("error.not_understood"));
    };

    // Whether the user *named* a service matters: "text mario" must go by SMS
    // even if his WhatsApp thread is open. Only an unspecified service lets
    // the live-thread preference kick in.
    let named_service = req.service.is_some();
    let service = req
        .service
        .unwrap_or_else(|| setting("default_service", DEFAULT_SERVICE));

    let Some(body) = req.body else {
        return action::ask_json(
            &t("ask.body"),
            &action::pack_context("body", &req.recipient, "", &service, ""),
        );
    };

    // Replying into a thread the user already has open beats composing on
    // every count — it's the only genuinely hands-free path, it threads
    // properly, and it needs no address book. So it goes first, unless the
    // user pinned a service or there's nothing live to answer.
    if req.explicit_reply || !named_service {
        if let Some(target) = live_target(&req.recipient, req.explicit_reply) {
            return act_reply(target, &body);
        }
    }
    if req.explicit_reply {
        // They asked to reply and there is nothing to reply to. Say so rather
        // than quietly starting a new conversation they didn't ask for.
        return action::say_json(&t("error.no_live_thread"));
    }

    resolve_and_act(&req.recipient, &service, &body)
}

#[cfg(target_arch = "wasm32")]
fn live_target(recipient: &str, explicit_reply: bool) -> Option<Option<String>> {
    choose_live(&ari::live_conversations(), recipient, explicit_reply)
}

/// Which live conversation to answer, if any. `live` is newest first.
///
/// `Some(None)` means "the newest one" — the user said "reply" and named
/// nobody. `None` means there is nothing here to answer, and the caller
/// composes instead.
pub fn choose_live(
    live: &[String],
    recipient: &str,
    explicit_reply: bool,
) -> Option<Option<String>> {
    if live.is_empty() {
        return None;
    }
    if recipient.trim().is_empty() {
        // Only a deliberate "reply" may take the newest thread. Doing it for
        // an unaddressed message would answer whoever happened to write last.
        return if explicit_reply { Some(None) } else { None };
    }
    let wanted = recipient.trim().to_lowercase();
    live.iter()
        .find(|name| name.to_lowercase() == wanted)
        .or_else(|| live.iter().find(|name| name.to_lowercase().contains(&wanted)))
        .map(|name| Some(name.clone()))
}

/// A reply reaches somebody with nobody having seen it, so it confirms on the
/// same setting as SMS and Matrix.
#[cfg(target_arch = "wasm32")]
fn act_reply(target: Option<String>, body: &str) -> String {
    let name = target.unwrap_or_default();
    if setting("confirm_before_sending", "always") == "never" {
        return action::reply_json(Some(name.as_str()).filter(|s| !s.is_empty()), body);
    }
    let question = if name.is_empty() {
        t_args("reply.confirm_unnamed", &[("body", body)])
    } else {
        t_args("reply.confirm", &[("recipient", &name), ("body", body)])
    };
    action::ask_json(&question, &action::pack_context("replyconfirm", &name, "", "", body))
}

/// Which branch a contact lookup lands on, decided purely from which of the
/// matches are reachable on the service being asked for.
///
/// Split out from [`resolve_and_act`] because that function is wasm-only —
/// it is wrapped in host calls that cannot run in a native test build — and
/// this is the part that was wrong. Same reason `yes_matches` is split out.
#[derive(Debug, PartialEq, Eq)]
enum Candidates {
    /// Nobody by that name at all.
    Nobody,
    /// Somebody by that name, but not reachable this way.
    Unreachable,
    /// Exactly one to act on, at this index into the match list.
    One(usize),
    /// Several to ask about.
    Several,
}

fn candidates(reachable: &[bool]) -> Candidates {
    let mut hits = reachable.iter().enumerate().filter(|(_, r)| **r).map(|(i, _)| i);
    let first = hits.next();
    match (reachable.is_empty(), first, hits.next()) {
        (true, _, _) => Candidates::Nobody,
        (_, None, _) => Candidates::Unreachable,
        (_, Some(i), None) => Candidates::One(i),
        (_, Some(_), Some(_)) => Candidates::Several,
    }
}

/// Turn a spoken name into somebody we can actually address, then act.
///
/// Four outcomes, and they are genuinely different answers to the user:
/// found one, found several, found nobody, or couldn't look at all.
#[cfg(target_arch = "wasm32")]
fn resolve_and_act(recipient: &str, service: &str, body: &str) -> String {
    if !ari::contacts_permission_granted() {
        // Not the same as "nobody by that name", and it must not sound like
        // it — the user can fix this one.
        return act(recipient, None, service, body);
    }

    let matches = ari::contacts_lookup(recipient);
    // Narrow to people reachable on the service being asked for BEFORE
    // counting. Three cards for one name where only one carries a phone
    // number is a single answer and two dead ends — asking "which Keith?"
    // there is a question the user cannot meaningfully answer, since the
    // other two would only fail once chosen.
    let reachable: Vec<bool> = matches
        .iter()
        .map(|c| channel_for(c, service).is_some())
        .collect();

    match candidates(&reachable) {
        Candidates::Nobody => {
            action::say_json(&t_args("error.no_contact", &[("recipient", recipient)]))
        }
        Candidates::Unreachable => {
            // Somebody by that name exists, they just can't be reached this
            // way. Name them when there is only one; with several we would be
            // picking one arbitrarily to apologise about.
            let who = if matches.len() == 1 {
                matches[0].display_name.as_str()
            } else {
                recipient
            };
            action::say_json(&t_args(
                "error.no_channel",
                &[("recipient", who), ("service", service)],
            ))
        }
        Candidates::One(i) => {
            let c = &matches[i];
            act(&c.display_name, channel_for(c, service), service, body)
        }
        Candidates::Several => {
            // More than one reachable Gail is the normal case, not an error.
            let names = matches
                .iter()
                .zip(&reachable)
                .filter(|(_, r)| **r)
                .map(|(c, _)| c.display_name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            action::ask_json(
                &t_args("ask.which", &[("recipient", recipient), ("names", &names)]),
                &action::pack_context("which", recipient, "", service, body),
            )
        }
    }
}

/// The id this contact is reachable on for `service`, if any.
#[cfg(target_arch = "wasm32")]
fn channel_for<'a>(c: &'a ari::Contact, service: &str) -> Option<&'a str> {
    c.channels
        .iter()
        .find(|ch| ch.service == service)
        .map(|ch| ch.id.as_str())
}

#[cfg(target_arch = "wasm32")]
fn act(recipient: &str, recipient_id: Option<&str>, service: &str, body: &str) -> String {
    let confirm = setting("confirm_before_sending", "always") != "never";
    match plan(service, confirm, TRUE_SEND) {
        Plan::Compose => action::compose_json(recipient, recipient_id, service, body),
        Plan::Send => {
            if transport::is_self_sent(service) {
                self_send(recipient, service, body)
            } else {
                action::send_json(recipient, recipient_id, service, body)
            }
        }
        Plan::Confirm => action::ask_json(
            &t_args("send.confirm", &[("recipient", recipient), ("body", body)]),
            &action::pack_context(
                "confirm",
                recipient,
                recipient_id.unwrap_or(""),
                service,
                body,
            ),
        ),
    }
}

#[cfg(target_arch = "wasm32")]
fn handle_reply(context: &str, text: &str) -> String {
    let Some((kind, recipient, recipient_id, service, body)) = action::unpack_context(context)
    else {
        return action::say_json(&t("error.not_understood"));
    };
    let id = Some(recipient_id.as_str()).filter(|s| !s.is_empty());

    match kind.as_str() {
        // The answer to "what do you want to say?" IS the message, so it
        // needs the raw utterance for the same reason the first one did.
        "body" => {
            let raw = ari::raw_input().unwrap_or(text);
            if raw.trim().is_empty() {
                return action::say_json(&t("error.no_body"));
            }
            resolve_and_act(&recipient, &service, raw.trim())
        }
        // "which Gail?" — the answer narrows the lookup, so run the whole
        // resolution again against what they just said rather than trying to
        // index into a list they can't see.
        "which" => resolve_and_act(text.trim(), &service, &body),
        "replyconfirm" => {
            if is_yes(text) {
                action::reply_json(Some(recipient.as_str()).filter(|s| !s.is_empty()), &body)
            } else {
                action::say_json(&t("send.cancelled"))
            }
        }
        "confirm" => {
            if is_yes(text) {
                act_confirmed(&recipient, id, &service, &body)
            } else {
                // Anything that isn't a clear yes is a no. The cost of a
                // false positive here is a message that cannot be recalled.
                action::say_json(&t("send.cancelled"))
            }
        }
        _ => action::say_json(&t("error.not_understood")),
    }
}

/// Post-confirmation send. Deliberately does not re-consult the setting —
/// the user just said yes to this exact message.
#[cfg(target_arch = "wasm32")]
fn act_confirmed(
    recipient: &str,
    recipient_id: Option<&str>,
    service: &str,
    body: &str,
) -> String {
    if transport::is_self_sent(service) {
        return self_send(recipient, service, body);
    }
    action::send_json(recipient, recipient_id, service, body)
}

/// Send it ourselves and say what happened. Nothing is handed to the
/// frontend, so every outcome has to be spoken here.
#[cfg(target_arch = "wasm32")]
fn self_send(recipient: &str, service: &str, body: &str) -> String {
    let outcome = match service {
        "matrix" => matrix::send(
            &setting("matrix_homeserver", ""),
            &setting("matrix_token", ""),
            recipient,
            body,
        ),
        // is_self_sent said yes but nothing handles it — a transport was
        // listed and not wired. Fail loudly rather than silently composing.
        _ => transport::Outcome::Failed(alloc::string::String::from("no transport")),
    };
    action::say_json(&describe(&outcome, recipient, service))
}

#[cfg(target_arch = "wasm32")]
fn describe(outcome: &transport::Outcome, recipient: &str, service: &str) -> String {
    use transport::Outcome as O;
    match outcome {
        O::Sent => t_args("send.done", &[("recipient", recipient)]),
        O::NotConfigured => t_args("error.not_configured", &[("service", service)]),
        O::NoRecipient => t_args("error.no_contact", &[("recipient", recipient)]),
        O::Ambiguous(names) => {
            t_args("error.ambiguous", &[("recipient", recipient), ("names", &names.join(", "))])
        }
        O::Encrypted => t_args("error.encrypted", &[("recipient", recipient)]),
        O::Offline => t("error.offline"),
        O::Failed(reason) => {
            ari::log(ari::LogLevel::Warn, &alloc::format!("{service} send failed: {reason}"));
            t_args("error.send_failed", &[("service", service)])
        }
    }
}

/// Affirmative in the active language. Falls back to English so a locale
/// without the key still works rather than treating every answer as "no".
#[cfg(target_arch = "wasm32")]
fn is_yes(text: &str) -> bool {
    let listed = ari::t("confirm.yes_words", &[]).unwrap_or(YES_WORDS_FALLBACK);
    yes_matches(text, listed)
}

const YES_WORDS_FALLBACK: &str = "yes, yeah, yep, yup, sure, ok, okay, send";

/// Whether `text` is one of the comma-separated affirmatives in `listed`.
///
/// Punctuation is stripped from both ends first. Cloud STT punctuates what it
/// transcribes, so a spoken "yes" arrives as "Yes." — and comparing that
/// verbatim read a clear confirmation as a refusal.
///
/// The list is comma-separated because several affirmatives are phrases:
/// Italian ships "va bene". Splitting on whitespace made "va" and "bene" each
/// count as a yes on their own while the phrase itself matched nothing, which
/// is a poor way to send a message that cannot be recalled.
fn yes_matches(text: &str, listed: &str) -> bool {
    let answer = text
        .trim()
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase();
    listed
        .split(',')
        .map(str::trim)
        .filter(|phrase| !phrase.is_empty())
        .any(|phrase| answer == phrase.to_lowercase())
}

#[cfg(target_arch = "wasm32")]
fn setting(key: &str, fallback: &str) -> String {
    ari::setting_get(key)
        .filter(|v| !v.trim().is_empty())
        .unwrap_or(fallback)
        .to_string()
}

#[cfg(target_arch = "wasm32")]
fn t(key: &str) -> String {
    ari::t(key, &[]).unwrap_or(key).to_string()
}

#[cfg(target_arch = "wasm32")]
fn t_args(key: &str, args: &[(&str, &str)]) -> String {
    ari::t(key, args).unwrap_or(key).to_string()
}

#[cfg(test)]
mod tests {

    #[test]
    fn nobody_by_that_name() {
        assert_eq!(candidates(&[]), Candidates::Nobody);
    }

    #[test]
    fn found_but_not_reachable_this_way() {
        assert_eq!(candidates(&[false]), Candidates::Unreachable);
        assert_eq!(candidates(&[false, false, false]), Candidates::Unreachable);
    }

    #[test]
    fn one_reachable_among_several_is_an_answer_not_a_question() {
        // The bug. Three "Keith Vassallo" cards, one with a phone number:
        // asking which is a question the user can't usefully answer, because
        // the other two would only fail once picked.
        assert_eq!(candidates(&[false, true, false]), Candidates::One(1));
        assert_eq!(candidates(&[true, false, false]), Candidates::One(0));
        assert_eq!(candidates(&[false, false, true]), Candidates::One(2));
    }

    #[test]
    fn the_single_match_case_still_works() {
        assert_eq!(candidates(&[true]), Candidates::One(0));
    }

    #[test]
    fn two_reachable_people_is_still_worth_asking_about() {
        assert_eq!(candidates(&[true, true]), Candidates::Several);
        assert_eq!(candidates(&[true, false, true]), Candidates::Several);
    }

    #[test]
    fn the_index_points_into_the_full_match_list_not_the_filtered_one() {
        // Getting this wrong would text the wrong person rather than fail
        // loudly, so it is worth its own test.
        match candidates(&[false, false, true]) {
            Candidates::One(i) => assert_eq!(i, 2),
            other => panic!("expected One(2), got {other:?}"),
        }
    }
    use super::*;

    #[test]
    fn a_service_we_cannot_send_always_composes() {
        // Regardless of the confirmation setting — there is nothing to
        // confirm when the user is going to tap send themselves.
        assert_eq!(plan("whatsapp", true, &["sms"]), Plan::Compose);
        assert_eq!(plan("whatsapp", false, &["sms"]), Plan::Compose);
    }

    #[test]
    fn a_true_send_service_confirms_by_default() {
        assert_eq!(plan("sms", true, &["sms"]), Plan::Confirm);
    }

    #[test]
    fn confirmation_can_be_turned_off() {
        assert_eq!(plan("sms", false, &["sms"]), Plan::Send);
    }

    fn live(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| String::from(*n)).collect()
    }

    #[test]
    fn a_named_person_with_a_live_thread_is_answered_in_it() {
        let l = live(&["Mario", "Gail"]);
        assert_eq!(choose_live(&l, "gail", false), Some(Some("Gail".into())));
    }

    #[test]
    fn an_exact_thread_beats_a_partial_one() {
        let l = live(&["Gail Marie", "Gail"]);
        assert_eq!(choose_live(&l, "gail", false), Some(Some("Gail".into())));
    }

    #[test]
    fn a_bare_reply_takes_the_newest_thread() {
        assert_eq!(choose_live(&live(&["Mario", "Gail"]), "", true), Some(None));
    }

    #[test]
    fn an_unaddressed_message_never_takes_the_newest_thread() {
        // "send a message saying I am late" must not answer whoever happened
        // to write last. Only a deliberate "reply" may do that.
        assert_eq!(choose_live(&live(&["Mario"]), "", false), None);
    }

    #[test]
    fn nobody_matching_means_compose_instead() {
        assert_eq!(choose_live(&live(&["Mario"]), "gail", false), None);
    }

    #[test]
    fn no_live_threads_means_compose_instead() {
        assert_eq!(choose_live(&[], "gail", true), None);
        assert_eq!(choose_live(&[], "", true), None);
    }

    #[test]
    fn everything_that_can_send_itself_confirms_first() {
        for service in TRUE_SEND {
            assert_eq!(plan(service, true, TRUE_SEND), Plan::Confirm, "{service}");
        }
    }

    #[test]
    fn the_messengers_that_reserve_sending_for_themselves_compose() {
        // No API to send on the user's behalf, so the user taps — and there
        // is nothing to confirm, because they see it before it goes.
        for service in ["whatsapp", "telegram", "signal", "messenger", "slack"] {
            assert_eq!(plan(service, true, TRUE_SEND), Plan::Compose, "{service}");
        }
    }

    #[test]
    fn confirmation_is_on_for_sms_unless_turned_off() {
        // The default matters: this is the one path where nobody sees the
        // message before it reaches another person.
        assert_eq!(plan("sms", true, TRUE_SEND), Plan::Confirm);
        assert_eq!(plan("sms", false, TRUE_SEND), Plan::Send);
    }

    const EN: &str = "yes, yeah, yep, yup, sure, ok, okay, send";
    const IT: &str = "sì, si, certo, va bene, ok, okay, manda, invia, procedi";

    #[test]
    fn a_bare_yes_confirms() {
        assert!(yes_matches("yes", EN));
    }

    #[test]
    fn a_transcribed_yes_confirms() {
        // Cloud STT punctuates: a spoken "yes" arrives as "Yes." and used to
        // be read as a refusal, cancelling the send.
        assert!(yes_matches("Yes.", EN));
        assert!(yes_matches("Yeah!", EN));
        assert!(yes_matches("  Okay...  ", EN));
    }

    #[test]
    fn a_refusal_is_not_a_confirmation() {
        assert!(!yes_matches("no", EN));
        assert!(!yes_matches("No.", EN));
        assert!(!yes_matches("don't", EN));
    }

    #[test]
    fn an_empty_answer_is_not_a_confirmation() {
        assert!(!yes_matches("", EN));
        assert!(!yes_matches("   ", EN));
        assert!(!yes_matches("...", EN));
    }

    #[test]
    fn a_multi_word_affirmative_confirms_as_a_phrase() {
        assert!(yes_matches("va bene", IT));
        assert!(yes_matches("Va bene.", IT));
    }

    #[test]
    fn half_a_phrase_does_not_confirm() {
        // "va" and "bene" were each a yes of their own while the whole phrase
        // matched nothing — the wrong way round for an unrecallable message.
        assert!(!yes_matches("va", IT));
        assert!(!yes_matches("bene", IT));
    }

    #[test]
    fn an_accented_affirmative_confirms() {
        assert!(yes_matches("sì", IT));
        assert!(yes_matches("Sì.", IT));
        assert!(yes_matches("si", IT));
    }

    #[test]
    fn one_language_does_not_answer_for_another() {
        assert!(!yes_matches("yes", IT));
        assert!(!yes_matches("certo", EN));
    }

    #[test]
    fn the_english_fallback_matches_the_shipped_list() {
        // The fallback only runs when the locale lookup fails, so a drift
        // between the two would be invisible until exactly then.
        assert!(yes_matches("yes", YES_WORDS_FALLBACK));
        assert!(!yes_matches("it", YES_WORDS_FALLBACK));
        assert!(!yes_matches("go", YES_WORDS_FALLBACK));
    }
}
