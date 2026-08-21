//! Envelope JSON. No typed builder exists for the `message` slot yet, so it
//! is hand-built here, the way `skills/music` hand-builds `media`.

extern crate alloc;

use alloc::string::{String, ToString};

/// Hand the message to another app with the body filled in.
///
/// **No `speak`.** The frontend decides whether it could target the named app
/// or had to fall back to the system chooser, and those are different
/// sentences — "just tap send" versus "pick Mario to send it". Only the
/// frontend knows which happened, so it supplies the line. Setting `speak`
/// here would make the skill claim an outcome it can't observe.
pub fn compose_json(
    recipient: &str,
    recipient_id: Option<&str>,
    service: &str,
    body: &str,
) -> String {
    let mut msg = serde_json::json!({
        "service": service,
        "recipient_label": recipient,
        "text": body,
        "delivery": "compose",
    });
    // Only ever set from a real address-book channel. A spoken name is not an
    // address, and emitting one here would have the frontend try to open a
    // chat with the literal string "Mario".
    if let Some(id) = recipient_id {
        msg["recipient_id"] = serde_json::Value::String(id.to_string());
    }
    serde_json::json!({ "v": 1, "message": msg }).to_string()
}

/// Ask for a true send — no user interaction at all.
///
/// **No `speak`,** for the same reason compose has none: the frontend may not
/// be able to honour it. Without the SMS permission, or without a number, it
/// composes instead — and a skill that had already said "Sent your message to
/// Mario" would be lying about the one action that can't be taken back.
pub fn send_json(
    recipient: &str,
    recipient_id: Option<&str>,
    service: &str,
    body: &str,
) -> String {
    let mut msg = serde_json::json!({
        "service": service,
        "recipient_label": recipient,
        "text": body,
        "delivery": "send",
    });
    if let Some(id) = recipient_id {
        msg["recipient_id"] = serde_json::Value::String(id.to_string());
    }
    serde_json::json!({ "v": 1, "message": msg }).to_string()
}

/// Answer a conversation that already has a live notification.
///
/// **No `speak`** — the frontend knows whether the thread was still there when
/// it fired, and whether notification access was ever granted. A skill that
/// announced a reply it couldn't verify would be claiming the one thing here
/// that can't be taken back.
pub fn reply_json(recipient: Option<&str>, body: &str) -> String {
    let mut r = serde_json::json!({ "text": body });
    if let Some(name) = recipient {
        r["recipient_label"] = serde_json::Value::String(name.to_string());
    }
    serde_json::json!({ "v": 1, "reply": r }).to_string()
}

/// Ask a question and route the answer straight back here. `context` is our
/// own opaque blob — the whole multi-turn mechanism, no session object.
pub fn ask_json(speak: &str, context: &str) -> String {
    serde_json::json!({
        "v": 1,
        "speak": speak,
        "await_reply": { "context": context },
    })
    .to_string()
}

/// Nothing happened and there's nothing to hand off — just say why.
pub fn say_json(speak: &str) -> String {
    serde_json::json!({ "v": 1, "speak": speak }).to_string()
}

/// State carried across a question. Deliberately a flat string rather than
/// JSON: it round-trips through the engine untouched and only this skill
/// ever reads it, so a parser would be ceremony.
pub fn pack_context(
    kind: &str,
    recipient: &str,
    recipient_id: &str,
    service: &str,
    body: &str,
) -> String {
    let mut s = String::from(kind);
    for part in [recipient, recipient_id, service, body] {
        s.push('\u{1f}');
        s.push_str(part);
    }
    s
}

/// `(kind, recipient, recipient_id, service, body)`. An empty
/// `recipient_id` means unresolved, not "id is empty string".
pub fn unpack_context(ctx: &str) -> Option<(String, String, String, String, String)> {
    let mut parts = ctx.split('\u{1f}');
    let kind = parts.next()?.to_string();
    let recipient = parts.next()?.to_string();
    let recipient_id = parts.next()?.to_string();
    let service = parts.next()?.to_string();
    let body = parts.next().unwrap_or("").to_string();
    Some((kind, recipient, recipient_id, service, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_carries_the_body_and_no_speak() {
        let v: serde_json::Value =
            serde_json::from_str(&compose_json("Mario", Some("35699000000"), "whatsapp", "I'll be home soon")).unwrap();
        assert_eq!(v["v"], 1);
        assert_eq!(v["message"]["service"], "whatsapp");
        assert_eq!(v["message"]["text"], "I'll be home soon");
        assert_eq!(v["message"]["recipient_label"], "Mario");
        assert_eq!(v["message"]["recipient_id"], "35699000000");
        assert_eq!(v["message"]["delivery"], "compose");
        assert!(
            v.get("speak").is_none(),
            "the frontend phrases compose — it knows whether it targeted the app or had to \
             open the chooser, and the skill does not",
        );
    }

    #[test]
    fn an_unresolved_recipient_carries_no_id() {
        // A spoken name is not an address. Emitting it as recipient_id would
        // have the frontend try to address a chat by the string "Mario".
        let v: serde_json::Value =
            serde_json::from_str(&compose_json("Mario", None, "sms", "hello")).unwrap();
        assert!(v["message"].get("recipient_id").is_none());
    }

    #[test]
    fn send_asks_but_never_claims_it_happened() {
        let v: serde_json::Value =
            serde_json::from_str(&send_json("Mario", None, "sms", "on my way")).unwrap();
        assert_eq!(v["message"]["delivery"], "send");
        assert!(
            v.get("speak").is_none(),
            "the frontend may have to compose instead — claiming a send here \
             would lie about the one action that can't be taken back",
        );
    }

    #[test]
    fn a_reply_carries_the_body_and_no_speak() {
        let v: serde_json::Value =
            serde_json::from_str(&reply_json(Some("Gail"), "On my way")).unwrap();
        assert_eq!(v["reply"]["text"], "On my way");
        assert_eq!(v["reply"]["recipient_label"], "Gail");
        assert!(v.get("speak").is_none());
        assert!(v.get("message").is_none(), "a reply is not also a new message");
    }

    #[test]
    fn a_reply_to_nobody_names_nobody() {
        // The frontend takes the newest live thread; a placeholder label here
        // would have it announce a name the skill never resolved.
        let v: serde_json::Value = serde_json::from_str(&reply_json(None, "On my way")).unwrap();
        assert!(v["reply"].get("recipient_label").is_none());
    }

    #[test]
    fn ask_emits_no_message_slot() {
        let v: serde_json::Value =
            serde_json::from_str(&ask_json("What do you want to say?", "ctx")).unwrap();
        assert_eq!(v["await_reply"]["context"], "ctx");
        assert!(
            v.get("message").is_none(),
            "a question must not also hand off a half-formed message",
        );
    }

    #[test]
    fn context_round_trips() {
        let packed = pack_context("confirm", "Gail Marie", "35677000000", "sms", "I'll be late");
        let (kind, recipient, id, service, body) = unpack_context(&packed).unwrap();
        assert_eq!(id, "35677000000");
        assert_eq!(kind, "confirm");
        assert_eq!(recipient, "Gail Marie");
        assert_eq!(service, "sms");
        assert_eq!(body, "I'll be late");
    }

    #[test]
    fn context_survives_a_body_containing_spaces_and_punctuation() {
        let body = "Meet me at 8, don't be late!";
        let packed = pack_context("confirm", "Gail", "", "whatsapp", body);
        assert_eq!(unpack_context(&packed).unwrap().4, body);
    }

    #[test]
    fn an_empty_body_round_trips_as_empty() {
        let packed = pack_context("body", "Gail", "", "sms", "");
        assert_eq!(unpack_context(&packed).unwrap().4, "");
    }
}
