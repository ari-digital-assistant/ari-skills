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

/// A ready-to-send HTTP request: method, absolute URL, bearer token, JSON body.
pub struct HaRequest {
    pub method: &'static str,
    pub url: String,
    pub token: String,
    pub body: String,
}

impl HaRequest {
    pub fn auth_header(&self) -> (String, String) {
        ("Authorization".to_string(), alloc::format!("Bearer {}", self.token))
    }
}

fn base(base_url: &str) -> &str {
    base_url.trim_end_matches('/')
}

pub fn build_conversation_request(base_url: &str, token: &str, text: &str, language: &str) -> HaRequest {
    // Order-sensitive body: callers assert the exact string `{"text":...,"language":...}`.
    // serde_json sorts object keys without the `preserve_order` feature, so build
    // the body by hand. `serde_json::Value::String` gives us correct escaping/quoting.
    let body = alloc::format!(
        "{{\"text\":{},\"language\":{}}}",
        serde_json::Value::String(text.to_string()),
        serde_json::Value::String(language.to_string()),
    );
    HaRequest {
        method: "POST",
        url: alloc::format!("{}/api/conversation/process", base(base_url)),
        token: token.to_string(),
        body,
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;

    #[test]
    fn conversation_request_shapes_url_and_body() {
        let r = build_conversation_request("http://hass.local:8123", "tok123", "turn on lights", "en");
        assert_eq!(r.method, "POST");
        assert_eq!(r.url, "http://hass.local:8123/api/conversation/process");
        assert_eq!(r.auth_header(), ("Authorization".to_string(), "Bearer tok123".to_string()));
        assert_eq!(r.body, r#"{"text":"turn on lights","language":"en"}"#);
    }

    #[test]
    fn conversation_request_trims_trailing_slash_on_base_url() {
        let r = build_conversation_request("http://hass.local:8123/", "t", "hi", "it");
        assert_eq!(r.url, "http://hass.local:8123/api/conversation/process");
        assert_eq!(r.body, r#"{"text":"hi","language":"it"}"#);
    }
}
