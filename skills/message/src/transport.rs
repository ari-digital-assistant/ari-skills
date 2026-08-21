//! Services the skill sends itself, over HTTPS, with no other app involved.
//!
//! Distinct from the frontend's true send (SMS): there the skill asks and the
//! platform does it, here the skill does it and then reports. Both share the
//! same rule — nobody sees the message before the recipient does, so both are
//! read back and confirmed first.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// What happened. Each variant is a different sentence to the user, and a
/// different thing for them to do about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Sent,
    /// No server or no token — the user has something to fill in.
    NotConfigured,
    /// The directory knows nobody by that name.
    NoRecipient,
    /// Several people match; guessing is exactly what we won't do.
    Ambiguous(Vec<String>),
    /// The room is end-to-end encrypted and we can't write to it. Honest
    /// refusal beats a message the recipient's client flags as suspect.
    Encrypted,
    /// The request never left. Not an HTTP status — its own sentence,
    /// because "check your connection" is useful and "server error" isn't.
    Offline,
    Failed(String),
}

/// Services the skill sends itself rather than handing to the frontend.
pub const SELF_SENT: &[&str] = &["matrix"];

pub fn is_self_sent(service: &str) -> bool {
    SELF_SENT.contains(&service)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_is_sent_by_the_skill_sms_is_not() {
        // SMS is a true send too, but the frontend performs it — the skill
        // only asks. Confusing the two would have the skill try to POST an
        // SMS somewhere.
        assert!(is_self_sent("matrix"));
        assert!(!is_self_sent("sms"));
        assert!(!is_self_sent("whatsapp"));
    }
}
