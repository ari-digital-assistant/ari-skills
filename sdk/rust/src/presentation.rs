//! Presentation primitives.
//!
//! Skills emit a unified envelope describing *what* the user should see —
//! cards, alerts, notifications, app launches, search queries — and the
//! frontend decides *how* to render on the current OS. Wire format is versioned
//! via `v` at the envelope root; `v: 1` is the current schema documented in
//! `ari-skills/docs/action-responses.md`.
//!
//! ```rust,ignore
//! use ari_skill_sdk::presentation as p;
//!
//! let json = p::Envelope::new()
//!     .speak("Pasta timer set for 8 minutes.")
//!     .card(
//!         p::Card::new("card_t_01HZ")
//!             .title("Pasta timer")
//!             .countdown_to(end_ts_ms)
//!             .started_at(created_ts_ms)
//!             .icon(p::Asset::new("timer_icon.png"))
//!             .action(p::Action::new("cancel", "Cancel").utterance("cancel my pasta timer"))
//!             .on_complete(
//!                 p::OnComplete::new().alert(
//!                     p::Alert::new("alert_t_01HZ")
//!                         .title("Pasta timer done")
//!                         .urgency(p::Urgency::Critical)
//!                         .sound(p::Sound::asset("timer.mp3"))
//!                         .speech_loop("Pasta timer")
//!                         .full_takeover(true)
//!                         .action(p::Action::new("stop_alert", "Stop").primary()),
//!                 ),
//!             ),
//!     )
//!     .to_json();
//! ```

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Serialize, Serializer};

pub const ENVELOPE_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Envelope
// ---------------------------------------------------------------------------

#[derive(Serialize, Default)]
pub struct Envelope {
    v: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    speak: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cards: Vec<Card>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    alerts: Vec<Alert>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notifications: Vec<Notification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    launch_app: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    open_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clipboard: Option<Clipboard>,
    #[serde(skip_serializing_if = "Dismiss::is_empty")]
    dismiss: Dismiss,
}

impl Envelope {
    pub fn new() -> Self {
        Envelope { v: ENVELOPE_VERSION, ..Default::default() }
    }

    pub fn speak(mut self, s: impl Into<String>) -> Self {
        self.speak = Some(s.into());
        self
    }

    pub fn card(mut self, card: Card) -> Self {
        self.cards.push(card);
        self
    }

    pub fn alert(mut self, alert: Alert) -> Self {
        self.alerts.push(alert);
        self
    }

    pub fn notification(mut self, notif: Notification) -> Self {
        self.notifications.push(notif);
        self
    }

    pub fn launch_app(mut self, target: impl Into<String>) -> Self {
        self.launch_app = Some(target.into());
        self
    }

    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.search = Some(query.into());
        self
    }

    pub fn open_url(mut self, url: impl Into<String>) -> Self {
        self.open_url = Some(url.into());
        self
    }

    pub fn clipboard(mut self, text: impl Into<String>) -> Self {
        self.clipboard = Some(Clipboard { text: text.into() });
        self
    }

    pub fn dismiss_card(mut self, id: impl Into<String>) -> Self {
        self.dismiss.cards.push(id.into());
        self
    }

    pub fn dismiss_notification(mut self, id: impl Into<String>) -> Self {
        self.dismiss.notifications.push(id.into());
        self
    }

    pub fn dismiss_alert(mut self, id: impl Into<String>) -> Self {
        self.dismiss.alerts.push(id.into());
        self
    }

    /// Serialize to the JSON string the host expects. Returns `{"v":1}` if
    /// serde somehow fails — impossible in practice for these types but a
    /// valid envelope is cheaper than unwinding the skill.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{\"v\":1}".to_string())
    }
}

#[derive(Serialize)]
struct Clipboard {
    text: String,
}

#[derive(Serialize, Default)]
struct Dismiss {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cards: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notifications: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    alerts: Vec<String>,
}

impl Dismiss {
    fn is_empty(&self) -> bool {
        self.cards.is_empty() && self.notifications.is_empty() && self.alerts.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Card
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct Card {
    id: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    countdown_to_ts_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    started_at_ts_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    progress: Option<Progress>,
    #[serde(skip_serializing_if = "Accent::is_default")]
    accent: Accent,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    actions: Vec<Action>,
    #[serde(skip_serializing_if = "Option::is_none")]
    on_complete: Option<OnComplete>,
}

impl Card {
    pub fn new(id: impl Into<String>) -> Self {
        Card {
            id: id.into(),
            title: String::new(),
            subtitle: None,
            body: None,
            icon: None,
            countdown_to_ts_ms: None,
            started_at_ts_ms: None,
            progress: None,
            accent: Accent::Default,
            actions: Vec::new(),
            on_complete: None,
        }
    }

    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = t.into();
        self
    }

    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }

    pub fn body(mut self, b: impl Into<String>) -> Self {
        self.body = Some(b.into());
        self
    }

    pub fn icon(mut self, asset: Asset) -> Self {
        self.icon = Some(asset.0);
        self
    }

    pub fn countdown_to(mut self, end_ts_ms: i64) -> Self {
        self.countdown_to_ts_ms = Some(end_ts_ms);
        self
    }

    pub fn started_at(mut self, start_ts_ms: i64) -> Self {
        self.started_at_ts_ms = Some(start_ts_ms);
        self
    }

    pub fn progress(mut self, value: f32) -> Self {
        self.progress = Some(Progress { value: value.clamp(0.0, 1.0) });
        self
    }

    pub fn accent(mut self, accent: Accent) -> Self {
        self.accent = accent;
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    pub fn on_complete(mut self, oc: OnComplete) -> Self {
        self.on_complete = Some(oc);
        self
    }
}

#[derive(Serialize)]
struct Progress {
    value: f32,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    Default,
    Warning,
    Success,
    Critical,
}

impl Accent {
    fn is_default(&self) -> bool {
        matches!(self, Accent::Default)
    }
}

// ---------------------------------------------------------------------------
// Alert
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct Alert {
    id: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    urgency: Urgency,
    sound: Sound,
    #[serde(skip_serializing_if = "Option::is_none")]
    speech_loop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_stop_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_cycles: Option<u32>,
    #[serde(skip_serializing_if = "core::ops::Not::not")]
    full_takeover: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    actions: Vec<Action>,
}

impl Alert {
    pub fn new(id: impl Into<String>) -> Self {
        Alert {
            id: id.into(),
            title: String::new(),
            body: None,
            urgency: Urgency::Normal,
            sound: Sound::SystemNotification,
            speech_loop: None,
            auto_stop_ms: None,
            max_cycles: None,
            full_takeover: false,
            icon: None,
            actions: Vec::new(),
        }
    }

    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = t.into();
        self
    }

    pub fn body(mut self, b: impl Into<String>) -> Self {
        self.body = Some(b.into());
        self
    }

    pub fn urgency(mut self, u: Urgency) -> Self {
        self.urgency = u;
        self
    }

    pub fn sound(mut self, s: Sound) -> Self {
        self.sound = s;
        self
    }

    pub fn speech_loop(mut self, text: impl Into<String>) -> Self {
        self.speech_loop = Some(text.into());
        self
    }

    pub fn auto_stop_ms(mut self, ms: u64) -> Self {
        self.auto_stop_ms = Some(ms);
        self
    }

    pub fn max_cycles(mut self, n: u32) -> Self {
        self.max_cycles = Some(n);
        self
    }

    pub fn full_takeover(mut self, enabled: bool) -> Self {
        self.full_takeover = enabled;
        self
    }

    /// Skill-bundled glyph rendered on the full-takeover UI. Optional;
    /// frontends fall back to a generic alarm-bell icon when omitted.
    /// Path is resolved against the emitting skill's `assets/` directory.
    pub fn icon(mut self, asset: Asset) -> Self {
        self.icon = Some(asset.0);
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    Normal,
    High,
    Critical,
}

/// Either a named system token or a skill-bundled asset reference.
/// Serialises as a plain string per the wire format.
pub enum Sound {
    SystemAlarm,
    SystemNotification,
    SystemSilent,
    Asset(String),
}

impl Sound {
    pub fn asset(path: impl Into<String>) -> Self {
        Sound::Asset(path.into())
    }

    fn as_wire(&self) -> String {
        match self {
            Sound::SystemAlarm => "system.alarm".to_string(),
            Sound::SystemNotification => "system.notification".to_string(),
            Sound::SystemSilent => "system.silent".to_string(),
            Sound::Asset(path) => {
                let mut s = String::with_capacity(path.len() + 6);
                s.push_str("asset:");
                s.push_str(path);
                s
            }
        }
    }
}

impl Serialize for Sound {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&self.as_wire())
    }
}

// ---------------------------------------------------------------------------
// Notification
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct Notification {
    id: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    importance: Importance,
    #[serde(skip_serializing_if = "core::ops::Not::not")]
    sticky: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    countdown_to_ts_ms: Option<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    actions: Vec<Action>,
}

impl Notification {
    pub fn new(id: impl Into<String>) -> Self {
        Notification {
            id: id.into(),
            title: String::new(),
            body: None,
            importance: Importance::Default,
            sticky: false,
            countdown_to_ts_ms: None,
            actions: Vec::new(),
        }
    }

    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = t.into();
        self
    }

    pub fn body(mut self, b: impl Into<String>) -> Self {
        self.body = Some(b.into());
        self
    }

    pub fn importance(mut self, i: Importance) -> Self {
        self.importance = i;
        self
    }

    pub fn sticky(mut self, stuck: bool) -> Self {
        self.sticky = stuck;
        self
    }

    pub fn countdown_to(mut self, end_ts_ms: i64) -> Self {
        self.countdown_to_ts_ms = Some(end_ts_ms);
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Importance {
    Min,
    Low,
    Default,
    High,
}

// ---------------------------------------------------------------------------
// Action buttons
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct Action {
    id: String,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    utterance: Option<String>,
    #[serde(skip_serializing_if = "Style::is_default")]
    style: Style,
}

impl Action {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Action {
            id: id.into(),
            label: label.into(),
            utterance: None,
            style: Style::Default,
        }
    }

    /// Attach an utterance the frontend will route through `engine.processInput`
    /// when the button is tapped. Reserved ids (`stop_alert`,
    /// `dismiss_notification`) short-circuit this; for them, pass `None`.
    pub fn utterance(mut self, u: impl Into<String>) -> Self {
        self.utterance = Some(u.into());
        self
    }

    pub fn primary(mut self) -> Self {
        self.style = Style::Primary;
        self
    }

    pub fn destructive(mut self) -> Self {
        self.style = Style::Destructive;
        self
    }
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Style {
    Default,
    Primary,
    Destructive,
}

impl Style {
    fn is_default(&self) -> bool {
        matches!(self, Style::Default)
    }
}

// ---------------------------------------------------------------------------
// Card.on_complete
// ---------------------------------------------------------------------------

#[derive(Serialize, Default)]
pub struct OnComplete {
    #[serde(skip_serializing_if = "Option::is_none")]
    alert: Option<Alert>,
    /// Default true in the wire format when an `on_complete` is present. We
    /// serialise explicitly to avoid ambiguity in parsers that aren't
    /// strictly schema-aware.
    dismiss_card: bool,
    /// Notification ids to dismiss when this card's countdown fires. Use
    /// when the skill emitted a paired ongoing notification with the card
    /// (the typical timer pattern) so the shade entry vanishes at the
    /// same instant the alert fires, instead of ticking past zero.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    dismiss_notifications: Vec<String>,
}

impl OnComplete {
    pub fn new() -> Self {
        OnComplete {
            alert: None,
            dismiss_card: true,
            dismiss_notifications: Vec::new(),
        }
    }

    pub fn alert(mut self, a: Alert) -> Self {
        self.alert = Some(a);
        self
    }

    pub fn dismiss_card(mut self, dismiss: bool) -> Self {
        self.dismiss_card = dismiss;
        self
    }

    pub fn dismiss_notification(mut self, id: impl Into<String>) -> Self {
        self.dismiss_notifications.push(id.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Asset helper
// ---------------------------------------------------------------------------

/// Asset URI wrapper. `Asset::new("icons/timer.png")` produces the string
/// `"asset:icons/timer.png"`. Small type-safety layer over passing raw URIs
/// to the `icon` / `sound` builders so typos turn into compile errors on
/// the skill side.
pub struct Asset(String);

impl Asset {
    pub fn new(path: impl AsRef<str>) -> Self {
        let path = path.as_ref();
        let mut s = String::with_capacity(path.len() + 6);
        s.push_str("asset:");
        s.push_str(path);
        Asset(s)
    }
}

// ---------------------------------------------------------------------------
// Tests (host-side only)
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn value(env: &Envelope) -> Value {
        serde_json::from_str(&env.to_json()).unwrap()
    }

    #[test]
    fn empty_envelope_just_version() {
        let v = value(&Envelope::new());
        assert_eq!(v, json!({ "v": 1 }));
    }

    #[test]
    fn speak_serialises() {
        let v = value(&Envelope::new().speak("hello"));
        assert_eq!(v["speak"], "hello");
    }

    #[test]
    fn omitted_optionals_are_absent_from_json() {
        // Don't want noise like "subtitle":null cluttering the wire.
        let env = Envelope::new().card(Card::new("c1").title("hi"));
        let v = value(&env);
        let card = &v["cards"][0];
        assert!(card.get("subtitle").is_none());
        assert!(card.get("body").is_none());
        assert!(card.get("icon").is_none());
        assert!(card.get("countdown_to_ts_ms").is_none());
        assert!(card.get("progress").is_none());
        // default accent is elided too
        assert!(card.get("accent").is_none());
    }

    #[test]
    fn card_countdown_envelope_matches_spec() {
        let env = Envelope::new()
            .speak("Pasta timer set for 8 minutes.")
            .card(
                Card::new("card_t_pasta")
                    .title("Pasta timer")
                    .countdown_to(1_000_480_000)
                    .started_at(1_000_000_000)
                    .icon(Asset::new("icons/timer.png"))
                    .action(Action::new("cancel", "Cancel").utterance("cancel my pasta timer"))
                    .on_complete(
                        OnComplete::new().alert(
                            Alert::new("alert_t_pasta")
                                .title("Pasta timer done")
                                .urgency(Urgency::Critical)
                                .sound(Sound::asset("timer_ding.wav"))
                                .speech_loop("Pasta timer")
                                .auto_stop_ms(120_000)
                                .max_cycles(12)
                                .full_takeover(true)
                                .action(Action::new("stop_alert", "Stop").primary()),
                        ),
                    ),
            );
        let v = value(&env);
        assert_eq!(v["v"], 1);
        assert_eq!(v["speak"], "Pasta timer set for 8 minutes.");
        let card = &v["cards"][0];
        assert_eq!(card["id"], "card_t_pasta");
        assert_eq!(card["countdown_to_ts_ms"], 1_000_480_000_i64);
        assert_eq!(card["started_at_ts_ms"], 1_000_000_000_i64);
        assert_eq!(card["icon"], "asset:icons/timer.png");
        assert_eq!(card["actions"][0]["id"], "cancel");
        assert_eq!(card["actions"][0]["utterance"], "cancel my pasta timer");
        let alert = &card["on_complete"]["alert"];
        assert_eq!(alert["urgency"], "critical");
        assert_eq!(alert["sound"], "asset:timer_ding.wav");
        assert_eq!(alert["speech_loop"], "Pasta timer");
        assert_eq!(alert["auto_stop_ms"], 120_000);
        assert_eq!(alert["max_cycles"], 12);
        assert_eq!(alert["full_takeover"], true);
        assert_eq!(alert["actions"][0]["id"], "stop_alert");
        assert_eq!(alert["actions"][0]["style"], "primary");
        assert_eq!(card["on_complete"]["dismiss_card"], true);
    }

    #[test]
    fn on_complete_dismiss_notifications_round_trip() {
        let env = Envelope::new().card(
            Card::new("c").title("t").countdown_to(1).on_complete(
                OnComplete::new()
                    .dismiss_notification("notif_a")
                    .dismiss_notification("notif_b"),
            ),
        );
        let v = value(&env);
        assert_eq!(
            v["cards"][0]["on_complete"]["dismiss_notifications"],
            json!(["notif_a", "notif_b"]),
        );
    }

    #[test]
    fn on_complete_dismiss_notifications_elided_when_empty() {
        // Don't pollute the envelope with an empty array. Most cards'
        // on_complete blocks won't dismiss anything.
        let env = Envelope::new().card(
            Card::new("c").title("t").countdown_to(1).on_complete(OnComplete::new()),
        );
        let v = value(&env);
        let oc = &v["cards"][0]["on_complete"];
        assert!(oc.get("dismiss_notifications").is_none());
    }

    #[test]
    fn alert_icon_serialises_as_asset_uri() {
        // Optional skill-bundled glyph for the takeover screen. The
        // builder accepts an Asset; the wire format is the `asset:<path>`
        // URI string the frontend's resolver expects.
        let env = Envelope::new().alert(
            Alert::new("a")
                .title("Done")
                .urgency(Urgency::Critical)
                .sound(Sound::asset("ding.wav"))
                .icon(Asset::new("icons/timer.png")),
        );
        let v = value(&env);
        assert_eq!(v["alerts"][0]["icon"], "asset:icons/timer.png");
    }

    #[test]
    fn alert_icon_elided_when_unset() {
        // Skills that don't ship an icon get a generic alarm-bell on
        // the frontend. Don't add a noise field to the envelope.
        let env = Envelope::new().alert(
            Alert::new("a").title("Done").urgency(Urgency::Critical).sound(Sound::SystemAlarm),
        );
        let v = value(&env);
        assert!(v["alerts"][0].get("icon").is_none());
    }

    #[test]
    fn sound_tokens_serialise_correctly() {
        assert_eq!(
            serde_json::to_string(&Sound::SystemAlarm).unwrap(),
            "\"system.alarm\""
        );
        assert_eq!(
            serde_json::to_string(&Sound::SystemNotification).unwrap(),
            "\"system.notification\""
        );
        assert_eq!(
            serde_json::to_string(&Sound::SystemSilent).unwrap(),
            "\"system.silent\""
        );
        assert_eq!(
            serde_json::to_string(&Sound::asset("ding.wav")).unwrap(),
            "\"asset:ding.wav\""
        );
    }

    #[test]
    fn dismiss_collects_ids() {
        let env = Envelope::new()
            .dismiss_card("card_1")
            .dismiss_card("card_2")
            .dismiss_notification("notif_1")
            .dismiss_alert("alert_1");
        let v = value(&env);
        assert_eq!(v["dismiss"]["cards"], json!(["card_1", "card_2"]));
        assert_eq!(v["dismiss"]["notifications"], json!(["notif_1"]));
        assert_eq!(v["dismiss"]["alerts"], json!(["alert_1"]));
    }

    #[test]
    fn empty_dismiss_block_is_elided() {
        let v = value(&Envelope::new().speak("hi"));
        assert!(v.get("dismiss").is_none());
    }

    #[test]
    fn single_shot_slots_elide_when_absent() {
        let v = value(&Envelope::new());
        for field in ["launch_app", "search", "open_url", "clipboard"] {
            assert!(v.get(field).is_none(), "{field} should be absent");
        }
    }

    #[test]
    fn launch_app_slot() {
        let v = value(&Envelope::new().launch_app("Spotify"));
        assert_eq!(v["launch_app"], "Spotify");
        assert!(v.get("speak").is_none());
    }

    #[test]
    fn clipboard_primitive() {
        let v = value(&Envelope::new().clipboard("copied text"));
        assert_eq!(v["clipboard"]["text"], "copied text");
    }

    #[test]
    fn notification_countdown() {
        let env = Envelope::new().notification(
            Notification::new("notif_pasta")
                .title("Pasta timer")
                .body("Running…")
                .importance(Importance::Default)
                .sticky(true)
                .countdown_to(1_000_480_000)
                .action(Action::new("cancel", "Cancel").utterance("cancel my pasta timer")),
        );
        let v = value(&env);
        let n = &v["notifications"][0];
        assert_eq!(n["id"], "notif_pasta");
        assert_eq!(n["importance"], "default");
        assert_eq!(n["sticky"], true);
        assert_eq!(n["countdown_to_ts_ms"], 1_000_480_000_i64);
    }

    #[test]
    fn action_style_omitted_when_default() {
        let env = Envelope::new()
            .card(Card::new("c1").title("hi").action(Action::new("ok", "OK")));
        let v = value(&env);
        let action = &v["cards"][0]["actions"][0];
        assert!(action.get("style").is_none());
    }

    #[test]
    fn progress_clamps() {
        let v = value(&Envelope::new().card(Card::new("c").title("t").progress(1.5)));
        assert_eq!(v["cards"][0]["progress"]["value"], 1.0);
        let v = value(&Envelope::new().card(Card::new("c").title("t").progress(-0.3)));
        assert_eq!(v["cards"][0]["progress"]["value"], 0.0);
    }
}
