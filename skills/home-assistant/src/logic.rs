#![allow(dead_code)]
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
        assert_eq!(classify("where is my phone"), Intent::Forward);
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

pub fn build_conversation_request(
    base_url: &str,
    token: &str,
    text: &str,
    language: &str,
    agent_id: Option<&str>,
) -> HaRequest {
    // Order-sensitive body: callers assert the exact string `{"text":...,"language":...}`
    // (plus an optional trailing `"agent_id":...`). serde_json sorts object keys without
    // the `preserve_order` feature, so build the body by hand. `serde_json::Value::String`
    // gives correct escaping/quoting.
    let mut body = alloc::format!(
        "{{\"text\":{},\"language\":{}",
        serde_json::Value::String(text.to_string()),
        serde_json::Value::String(language.to_string()),
    );
    // Optional: pin a specific HA conversation agent entity (e.g.
    // `conversation.openai_conversation`). When blank/absent, HA's
    // `/api/conversation/process` uses its built-in default (local) agent —
    // NOT the UI's preferred agent — so this is the only way to reach an LLM
    // agent like ChatGPT.
    if let Some(id) = agent_id {
        let id = id.trim();
        if !id.is_empty() {
            body.push_str(&alloc::format!(
                ",\"agent_id\":{}",
                serde_json::Value::String(id.to_string())
            ));
        }
    }
    body.push('}');
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
        let r = build_conversation_request("http://hass.local:8123", "tok123", "turn on lights", "en", None);
        assert_eq!(r.method, "POST");
        assert_eq!(r.url, "http://hass.local:8123/api/conversation/process");
        assert_eq!(r.auth_header(), ("Authorization".to_string(), "Bearer tok123".to_string()));
        assert_eq!(r.body, r#"{"text":"turn on lights","language":"en"}"#);
    }

    #[test]
    fn conversation_request_trims_trailing_slash_on_base_url() {
        let r = build_conversation_request("http://hass.local:8123/", "t", "hi", "it", None);
        assert_eq!(r.url, "http://hass.local:8123/api/conversation/process");
        assert_eq!(r.body, r#"{"text":"hi","language":"it"}"#);
    }

    #[test]
    fn conversation_request_escapes_body_text() {
        let r = build_conversation_request("http://h:8123", "t", "say \"hi\"\nbye", "en", None);
        assert_eq!(r.body, r#"{"text":"say \"hi\"\nbye","language":"en"}"#);
    }

    #[test]
    fn conversation_request_includes_agent_id_when_set() {
        let r = build_conversation_request("http://h:8123", "t", "hi", "en", Some("conversation.chatgpt"));
        assert_eq!(r.body, r#"{"text":"hi","language":"en","agent_id":"conversation.chatgpt"}"#);
    }

    #[test]
    fn conversation_request_omits_blank_agent_id() {
        let r = build_conversation_request("http://h:8123", "t", "hi", "en", Some("   "));
        assert_eq!(r.body, r#"{"text":"hi","language":"en"}"#);
    }
}

use alloc::vec::Vec;
use serde::Deserialize;

#[derive(Deserialize)]
struct ConvEnvelope {
    #[serde(default)]
    continue_conversation: bool,
    response: ConvResponse,
}
#[derive(Deserialize)]
struct ConvResponse {
    #[serde(default)]
    response_type: String,
    #[serde(default)]
    data: ConvData,
    #[serde(default)]
    speech: ConvSpeech,
}
#[derive(Deserialize, Default)]
struct ConvData {
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    success: Vec<ConvTarget>,
}
#[derive(Deserialize)]
struct ConvTarget {
    #[serde(default)]
    name: String,
}
#[derive(Deserialize, Default)]
struct ConvSpeech {
    #[serde(default)]
    plain: ConvPlain,
}
#[derive(Deserialize, Default)]
struct ConvPlain {
    #[serde(default)]
    speech: Option<String>,
}

/// Distilled, host-agnostic view of a conversation/process reply.
pub struct ConversationResult {
    pub speech: Option<String>,
    pub success_names: Vec<String>,
    pub continue_conversation: bool,
    pub no_match: bool,
}

pub fn parse_conversation_response(json: &str) -> Option<ConversationResult> {
    let env: ConvEnvelope = serde_json::from_str(json).ok()?;
    let no_match = env.response.response_type == "error"
        && env.response.data.code.as_deref() == Some("no_intent_match");
    Some(ConversationResult {
        speech: env.response.speech.plain.speech,
        success_names: env.response.data.success.into_iter().map(|t| t.name).collect(),
        continue_conversation: env.continue_conversation,
        no_match,
    })
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    const ACTION_DONE: &str = r#"{"continue_conversation":false,"conversation_id":"01J","response":{"response_type":"action_done","language":"en","data":{"targets":[{"type":"area","name":"Kitchen","id":"kitchen"}],"success":[{"type":"entity","name":"Kitchen Light","id":"light.kitchen"}],"failed":[]},"speech":{"plain":{"speech":"Turned on the kitchen light"}}}}"#;

    const NO_MATCH: &str = r#"{"continue_conversation":false,"conversation_id":"01K","response":{"response_type":"error","language":"en","data":{"code":"no_intent_match"},"speech":{"plain":{"speech":"Sorry, I couldn't understand that"}}}}"#;

    #[test]
    fn parses_action_done() {
        let r = parse_conversation_response(ACTION_DONE).unwrap();
        assert_eq!(r.speech.as_deref(), Some("Turned on the kitchen light"));
        assert_eq!(r.success_names, vec!["Kitchen Light".to_string()]);
        assert_eq!(r.continue_conversation, false);
        assert_eq!(r.no_match, false);
    }

    #[test]
    fn detects_no_intent_match() {
        let r = parse_conversation_response(NO_MATCH).unwrap();
        assert_eq!(r.no_match, true);
        assert_eq!(r.speech.as_deref(), Some("Sorry, I couldn't understand that"));
    }

    #[test]
    fn malformed_json_is_error() {
        assert!(parse_conversation_response("not json").is_none());
    }
}

/// Build the action envelope for a forwarded command. `card_title` is the
/// localized title the host passes in (e.g. "Done"). On no-match we tag the
/// envelope with `_ari_no_match` so the engine fallback tier can fall through
/// to the assistant; no card is shown.
pub fn build_conversation_envelope(result: &ConversationResult, card_title: &str) -> String {
    let speak = result.speech.clone().unwrap_or_default();
    let mut env = serde_json::json!({ "v": 1, "speak": speak });
    if result.no_match {
        env["_ari_no_match"] = serde_json::Value::Bool(true);
        return env.to_string();
    }
    if !result.success_names.is_empty() {
        let subtitle = result.success_names.join(", ");
        env["cards"] = serde_json::json!([{
            "id": "ha_result",
            "title": card_title,
            "subtitle": subtitle,
            "accent": "success"
        }]);
    }
    env.to_string()
}

/// A minimal speak-only envelope. Never carries `_ari_no_match` (these are
/// genuine, user-facing messages, not fall-through signals).
pub fn error_envelope(speak: &str) -> String {
    serde_json::json!({ "v": 1, "speak": speak }).to_string()
}

#[cfg(test)]
mod envelope_tests {
    use super::*;

    #[test]
    fn speaks_and_cards_on_success() {
        let r = ConversationResult {
            speech: Some("Turned on the kitchen light".into()),
            success_names: vec!["Kitchen Light".into()],
            continue_conversation: false,
            no_match: false,
        };
        let v: serde_json::Value = serde_json::from_str(&build_conversation_envelope(&r, "Done")).unwrap();
        assert_eq!(v["v"], 1);
        assert_eq!(v["speak"], "Turned on the kitchen light");
        assert_eq!(v["cards"][0]["title"], "Done");
        assert_eq!(v["cards"][0]["subtitle"], "Kitchen Light");
        assert!(v.get("_ari_no_match").is_none());
    }

    #[test]
    fn no_match_tags_envelope_and_uses_fallback_speech() {
        let r = ConversationResult {
            speech: Some("Sorry".into()),
            success_names: vec![],
            continue_conversation: false,
            no_match: true,
        };
        let v: serde_json::Value = serde_json::from_str(&build_conversation_envelope(&r, "Done")).unwrap();
        assert_eq!(v["_ari_no_match"], true);
        assert_eq!(v["speak"], "Sorry");
        assert!(v.get("cards").is_none());
    }
}

pub struct Person {
    pub name: String,
    pub state: String,
}

/// HA Jinja template that prints one `entity_id|friendly_name|state` line per
/// person entity. Newlines separate rows.
const PERSON_TEMPLATE: &str =
    "{% for p in states.person %}{{ p.entity_id }}|{{ p.attributes.friendly_name }}|{{ p.state }}\n{% endfor %}";

pub fn build_person_template_request(base_url: &str, token: &str) -> HaRequest {
    let body = serde_json::json!({ "template": PERSON_TEMPLATE }).to_string();
    HaRequest {
        method: "POST",
        url: alloc::format!("{}/api/template", base(base_url)),
        token: token.to_string(),
        body,
    }
}

pub fn parse_people(raw: &str) -> Vec<Person> {
    raw.lines()
        .filter_map(|line| {
            let mut it = line.splitn(3, '|');
            let _entity = it.next()?;
            let name = it.next()?.trim();
            let state = it.next()?.trim();
            if name.is_empty() {
                return None;
            }
            Some(Person { name: name.to_string(), state: state.to_string() })
        })
        .collect()
}

/// Exact (case-insensitive) name match is preferred; otherwise the FIRST
/// substring match in HA's person-iteration order wins (a known v1 limitation
/// for ambiguous short names).
pub fn match_person<'a>(people: &'a [Person], spoken: &str) -> Option<&'a Person> {
    let want = spoken.trim().to_ascii_lowercase();
    people
        .iter()
        .find(|p| p.name.to_ascii_lowercase() == want)
        .or_else(|| people.iter().find(|p| p.name.to_ascii_lowercase().contains(&want)))
}

/// Build the localized person-location envelope. `at_tmpl`/`home_tmpl`/
/// `away_tmpl` are localized strings with `{name}`/`{place}` slots supplied by
/// the host via `ari::t`. `home_label`/`away_label` localize the card subtitle.
#[allow(clippy::too_many_arguments)]
pub fn build_person_envelope(
    person: &Person,
    at_tmpl: &str,
    home_tmpl: &str,
    away_tmpl: &str,
    name_for_card: &str,
    home_label: &str,
    away_label: &str,
) -> String {
    let st = person.state.as_str();
    let (speak, subtitle) = match st {
        "home" => (home_tmpl.replace("{name}", &person.name), home_label.to_string()),
        "not_home" | "away" | "unknown" | "unavailable" => {
            (away_tmpl.replace("{name}", &person.name), away_label.to_string())
        }
        zone => (
            at_tmpl.replace("{name}", &person.name).replace("{place}", zone),
            zone.to_string(),
        ),
    };
    serde_json::json!({
        "v": 1,
        "speak": speak,
        "cards": [{ "id": "ha_person", "title": name_for_card, "subtitle": subtitle, "accent": "default" }]
    })
    .to_string()
}

#[cfg(test)]
mod person_tests {
    use super::*;

    #[test]
    fn person_template_request_targets_template_endpoint() {
        let r = build_person_template_request("http://hass.local:8123/", "tok");
        assert_eq!(r.method, "POST");
        assert_eq!(r.url, "http://hass.local:8123/api/template");
        assert!(r.body.contains("states.person"));
    }

    #[test]
    fn parses_person_lines() {
        let raw = "person.keith|Keith|Work\nperson.sarah|Sarah Jane|home\n";
        let people = parse_people(raw);
        assert_eq!(people.len(), 2);
        assert_eq!(people[0].name, "Keith");
        assert_eq!(people[0].state, "Work");
        assert_eq!(people[1].name, "Sarah Jane");
    }

    #[test]
    fn matches_person_case_insensitively() {
        let people = parse_people("person.keith|Keith|Work\nperson.sarah|Sarah Jane|home\n");
        assert_eq!(match_person(&people, "keith").unwrap().state, "Work");
        assert_eq!(match_person(&people, "sarah jane").unwrap().state, "home");
        assert!(match_person(&people, "bob").is_none());
    }

    #[test]
    fn person_envelope_home_away_zone() {
        let p = Person { name: "Keith".into(), state: "Work".into() };
        let v: serde_json::Value = serde_json::from_str(&build_person_envelope(
            &p, "{name} is at {place}.", "{name} is home.", "{name} is away.", "Keith", "Home", "Away",
        )).unwrap();
        assert_eq!(v["speak"], "Keith is at Work.");
        assert_eq!(v["cards"][0]["subtitle"], "Work");
    }

    #[test]
    fn person_envelope_not_home_is_away() {
        let p = Person { name: "Keith".into(), state: "not_home".into() };
        let v: serde_json::Value = serde_json::from_str(&build_person_envelope(
            &p, "{name} is at {place}.", "{name} is home.", "{name} is away.", "Keith", "Home", "Away",
        )).unwrap();
        assert_eq!(v["speak"], "Keith is away.");
        assert_eq!(v["cards"][0]["subtitle"], "Away");
    }

    #[test]
    fn person_envelope_unknown_is_away() {
        let p = Person { name: "Keith".into(), state: "unknown".into() };
        let v: serde_json::Value = serde_json::from_str(&build_person_envelope(
            &p, "{name} is at {place}.", "{name} is home.", "{name} is away.", "Keith", "Home", "Away",
        )).unwrap();
        assert_eq!(v["speak"], "Keith is away.");
    }
}

/// Build a `GET /api/states` request used at settings-time to enumerate
/// available `conversation.*` agents for the agent-picker dropdown.
pub fn build_states_request(base_url: &str, token: &str) -> HaRequest {
    HaRequest {
        method: "GET",
        url: alloc::format!("{}/api/states", base(base_url)),
        token: token.to_string(),
        body: String::new(),
    }
}

/// Parse `/api/states` JSON into `(entity_id, friendly_name)` pairs for every
/// `conversation.*` entity. Falls back to the entity_id when no (or empty)
/// friendly_name is present. Malformed JSON yields an empty list.
pub fn parse_conversation_agents(body: &str) -> Vec<(String, String)> {
    #[derive(Deserialize, Default)]
    struct Attrs {
        #[serde(default)]
        friendly_name: Option<String>,
    }
    #[derive(Deserialize)]
    struct State {
        entity_id: String,
        #[serde(default)]
        attributes: Attrs,
    }
    let states: Vec<State> = match serde_json::from_str(body) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    states
        .into_iter()
        .filter(|s| s.entity_id.starts_with("conversation."))
        .map(|s| {
            let label = s
                .attributes
                .friendly_name
                .filter(|f| !f.is_empty())
                .unwrap_or_else(|| s.entity_id.clone());
            (s.entity_id, label)
        })
        .collect()
}

#[cfg(test)]
mod agents_tests {
    use super::*;

    #[test]
    fn states_request_targets_states_endpoint() {
        let r = build_states_request("http://h:8123/", "tok");
        assert_eq!(r.method, "GET");
        assert_eq!(r.url, "http://h:8123/api/states");
    }

    #[test]
    fn parses_conversation_agents_from_states() {
        let body = r#"[
          {"entity_id":"light.kitchen","state":"on","attributes":{"friendly_name":"Kitchen"}},
          {"entity_id":"conversation.home_assistant","state":"unknown","attributes":{"friendly_name":"Home Assistant"}},
          {"entity_id":"conversation.chatgpt","state":"unknown","attributes":{"friendly_name":"ChatGPT"}}
        ]"#;
        let agents = parse_conversation_agents(body);
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0], ("conversation.home_assistant".to_string(), "Home Assistant".to_string()));
        assert_eq!(agents[1], ("conversation.chatgpt".to_string(), "ChatGPT".to_string()));
    }

    #[test]
    fn agent_without_friendly_name_falls_back_to_entity_id() {
        let body = r#"[{"entity_id":"conversation.x","state":"unknown","attributes":{}}]"#;
        let agents = parse_conversation_agents(body);
        assert_eq!(agents[0], ("conversation.x".to_string(), "conversation.x".to_string()));
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Unreachable,
    UnreachableLan,
    Unauthorized,
}

/// Map an HTTP `status` (0 == transport failure) to an error kind, or `None`
/// when the response is usable (2xx). `private_base` adds the home-network
/// hint to transport failures.
pub fn http_error_kind(status: u16, private_base: bool) -> Option<ErrorKind> {
    match status {
        200..=299 => None,
        0 => Some(if private_base { ErrorKind::UnreachableLan } else { ErrorKind::Unreachable }),
        401 | 403 => Some(ErrorKind::Unauthorized),
        _ => Some(ErrorKind::Unreachable),
    }
}

pub fn is_private_base_url(base_url: &str) -> bool {
    match url::Url::parse(base_url) {
        Ok(u) => match u.host() {
            Some(url::Host::Domain(d)) => {
                let d = d.to_ascii_lowercase();
                d == "localhost" || d.ends_with(".local") || d.ends_with(".lan")
            }
            Some(url::Host::Ipv4(ip)) => ip.is_private() || ip.is_loopback(),
            Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
            None => false,
        },
        Err(_) => false,
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn maps_statuses_to_kinds() {
        assert_eq!(http_error_kind(0, true), Some(ErrorKind::UnreachableLan));
        assert_eq!(http_error_kind(0, false), Some(ErrorKind::Unreachable));
        assert_eq!(http_error_kind(401, false), Some(ErrorKind::Unauthorized));
        assert_eq!(http_error_kind(403, false), Some(ErrorKind::Unauthorized));
        assert_eq!(http_error_kind(200, false), None);
        assert_eq!(http_error_kind(500, false), Some(ErrorKind::Unreachable));
    }

    #[test]
    fn detects_private_base_url() {
        assert!(is_private_base_url("http://homeassistant.local:8123"));
        assert!(is_private_base_url("http://192.168.1.5:8123"));
        assert!(!is_private_base_url("https://abcd.ui.nabu.casa"));
    }
}
