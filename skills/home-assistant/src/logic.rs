// Pure, host-independent logic. Functions are added in later tasks.

use alloc::string::{String, ToString};

#[derive(Debug, PartialEq, Eq)]
pub enum Intent {
    /// "where is <name>" — answered by Ari via a person.* state read.
    PersonLocation { name: String },
    /// Everything else — forwarded verbatim to HA conversation/process.
    Forward,
}

/// A small stop-list of nouns that, after "where is/are", indicate a
/// non-person locator question HA's conversation API should handle instead.
const NON_PERSON_AFTER_WHERE: &[&str] = &[
    "the nearest", "nearest", "my phone", "my keys", "the dog", "the cat",
];

pub fn classify(input: &str) -> Intent {
    let t = input.trim();
    for prefix in ["where is ", "where are "] {
        if let Some(rest) = t.strip_prefix(prefix) {
            let name = rest.trim();
            if name.is_empty() {
                return Intent::Forward;
            }
            if NON_PERSON_AFTER_WHERE.iter().any(|n| name.starts_with(n)) {
                return Intent::Forward;
            }
            return Intent::PersonLocation { name: name.to_string() };
        }
    }
    Intent::Forward
}

#[cfg(test)]
mod classify_tests {
    use super::*;

    #[test]
    fn where_is_person_extracts_name() {
        assert_eq!(classify("where is keith"), Intent::PersonLocation { name: "keith".into() });
        assert_eq!(classify("where is sarah jane"), Intent::PersonLocation { name: "sarah jane".into() });
        assert_eq!(classify("where are the kids"), Intent::PersonLocation { name: "the kids".into() });
    }

    #[test]
    fn control_and_status_forward() {
        assert_eq!(classify("turn on the kitchen lights"), Intent::Forward);
        assert_eq!(classify("set the bedroom to 21 degrees"), Intent::Forward);
        assert_eq!(classify("is the garage door open"), Intent::Forward);
    }

    #[test]
    fn where_is_a_thing_is_not_person_location() {
        assert_eq!(classify("where is the nearest pizza"), Intent::Forward);
    }
}
