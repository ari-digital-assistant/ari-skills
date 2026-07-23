//! Presentation primitives.
//!
//! Skills emit a unified envelope describing *what* the user should see —
//! cards, alerts, notifications, app launches, search queries — and the
//! frontend decides *how* to render on the current OS. Wire format is versioned
//! via `v` at the envelope root; `v: 1` is the current schema documented in
//! `ari-skills/docs/reference-actions.md`.
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
    #[serde(skip_serializing_if = "Option::is_none")]
    alarm: Option<Alarm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    navigate: Option<Navigate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    await_reply: Option<AwaitReply>,
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

    pub fn alarm(mut self, alarm: Alarm) -> Self {
        self.alarm = Some(alarm);
        self
    }

    pub fn navigate(mut self, navigate: Navigate) -> Self {
        self.navigate = Some(navigate);
        self
    }

    /// Signal that the engine should await the user's spoken reply to this
    /// envelope's question and deliver it to this skill's `execute_reply`.
    /// `context` is opaque, skill-defined, engine-stored (never round-tripped
    /// through an utterance — so it can be arbitrarily large JSON).
    pub fn await_reply(mut self, context: impl Into<String>) -> Self {
        self.await_reply = Some(AwaitReply { context: context.into() });
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

/// A day-of-week code for alarm recurrence. Serialises to its 3-letter code.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Day {
    Mon, Tue, Wed, Thu, Fri, Sat, Sun,
}

impl Day {
    pub fn code(&self) -> &'static str {
        match self {
            Day::Mon => "mon", Day::Tue => "tue", Day::Wed => "wed",
            Day::Thu => "thu", Day::Fri => "fri", Day::Sat => "sat",
            Day::Sun => "sun",
        }
    }
}

/// An alarm command. `op:"set"` creates a device alarm; `op:"show"` opens the
/// alarm list. Semantic only — carries no platform intent knowledge.
#[derive(Serialize, Default)]
pub struct Alarm {
    op: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    hour: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    minute: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    days: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skip_ui: Option<bool>,
}

impl Alarm {
    pub fn set(hour: u8, minute: u8) -> Self {
        Alarm {
            op: "set",
            hour: Some(hour),
            minute: Some(minute),
            skip_ui: Some(true),
            ..Default::default()
        }
    }

    pub fn show() -> Self {
        Alarm { op: "show", ..Default::default() }
    }

    pub fn message(mut self, m: impl Into<String>) -> Self {
        self.message = Some(m.into());
        self
    }

    pub fn days(mut self, days: &[Day]) -> Self {
        self.days = days.iter().map(|d| d.code()).collect();
        self
    }
}

/// A navigation command. Semantic only — carries no platform intent knowledge.
/// `mode` is a frontend-neutral hint: `default_app` opens the destination in the
/// user's default maps app; `turn_by_turn` starts turn-by-turn navigation.
#[derive(Serialize, Default)]
pub struct Navigate {
    destination: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
}

impl Navigate {
    pub fn to(destination: impl Into<String>) -> Self {
        Navigate { destination: destination.into(), mode: None }
    }

    pub fn mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = Some(mode.into());
        self
    }
}

#[derive(Serialize, Default)]
struct AwaitReply {
    context: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    stat: Option<Stat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    list: Option<ListCard>,
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
            stat: None,
            list: None,
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

    pub fn stat(mut self, stat: Stat) -> Self {
        self.stat = Some(stat);
        self
    }

    pub fn list(mut self, list: ListCard) -> Self {
        self.list = Some(list);
        self
    }
}

/// One row of a generic "list" card.
#[derive(Serialize)]
pub struct ListRow {
    leading: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trailing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    badge: Option<IconText>,
}

impl ListRow {
    pub fn new(leading: impl Into<String>) -> Self {
        ListRow { leading: leading.into(), icon: None, text: None, trailing: None, badge: None }
    }
    pub fn icon(mut self, asset: Asset) -> Self { self.icon = Some(asset.0); self }
    pub fn text(mut self, t: impl Into<String>) -> Self { self.text = Some(t.into()); self }
    pub fn trailing(mut self, t: impl Into<String>) -> Self { self.trailing = Some(t.into()); self }
    pub fn badge(mut self, b: IconText) -> Self { self.badge = Some(b); self }
}

/// Generic "list" card body: an optional summary chip, a column of rows, and a
/// footer. Any skill can populate it.
#[derive(Serialize)]
pub struct ListCard {
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<IconText>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    rows: Vec<ListRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<IconText>,
}

impl ListCard {
    pub fn new() -> Self { ListCard { summary: None, rows: Vec::new(), footer: None } }
    pub fn summary(mut self, s: IconText) -> Self { self.summary = Some(s); self }
    pub fn row(mut self, r: ListRow) -> Self { self.rows.push(r); self }
    pub fn footer(mut self, f: IconText) -> Self { self.footer = Some(f); self }
}

impl Default for ListCard { fn default() -> Self { Self::new() } }

/// Generic "stat" card body: a big headline value, a secondary caption, an
/// optional emphasised pill, a row of metrics, an opaque full-bleed background
/// image, and a low-emphasis footer. Any skill can populate it.
#[derive(Serialize)]
pub struct Stat {
    headline: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pill: Option<IconText>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    metrics: Vec<IconText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<IconText>,
}

impl Stat {
    pub fn new(headline: impl Into<String>) -> Self {
        Stat { headline: headline.into(), caption: None, pill: None,
               metrics: Vec::new(), background: None, footer: None }
    }
    pub fn caption(mut self, c: impl Into<String>) -> Self { self.caption = Some(c.into()); self }
    pub fn pill(mut self, p: IconText) -> Self { self.pill = Some(p); self }
    pub fn metric(mut self, m: IconText) -> Self { self.metrics.push(m); self }
    pub fn background(mut self, asset: Asset) -> Self { self.background = Some(asset.0); self }
    pub fn footer(mut self, f: IconText) -> Self { self.footer = Some(f); self }
}

/// A styled icon + label, used by stat/list card slots (pill, metric, footer,
/// summary, row badge). `icon` is an `asset:` reference.
#[derive(Serialize)]
pub struct IconText {
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    text: String,
}

impl IconText {
    pub fn new(text: impl Into<String>) -> Self {
        IconText { icon: None, text: text.into() }
    }
    pub fn icon(mut self, asset: Asset) -> Self {
        self.icon = Some(asset.0);
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

    #[test]
    fn icon_text_serializes_with_and_without_icon() {
        let with = IconText::new("Wind 18 km/h").icon(Asset::new("ui/wind.png"));
        let j = serde_json::to_string(&with).unwrap();
        assert_eq!(j, r#"{"icon":"asset:ui/wind.png","text":"Wind 18 km/h"}"#);
        let plain = IconText::new("just text");
        let j2 = serde_json::to_string(&plain).unwrap();
        assert_eq!(j2, r#"{"text":"just text"}"#);
    }

    #[test]
    fn stat_card_serializes() {
        let stat = Stat::new("21°")
            .caption("cloudy")
            .pill(IconText::new("Feels like 20°").icon(Asset::new("ui/thermometer.png")))
            .metric(IconText::new("Wind 18 km/h").icon(Asset::new("ui/wind.png")))
            .metric(IconText::new("Humidity 69%").icon(Asset::new("ui/droplet.png")))
            .background(Asset::new("heroes/cloudy.png"))
            .footer(IconText::new("Weather data by Open-Meteo.com").icon(Asset::new("ui/shield.png")));
        let json = Envelope::new()
            .card(Card::new("weather_current").title("London").stat(stat))
            .to_json();
        assert!(json.contains(r#""stat":{"#));
        assert!(json.contains(r#""headline":"21°""#));
        assert!(json.contains(r#""caption":"cloudy""#));
        assert!(json.contains(r#""background":"asset:heroes/cloudy.png""#));
        assert!(json.contains(r#""metrics":[{"icon":"asset:ui/wind.png","text":"Wind 18 km/h"}"#));
    }

    #[test]
    fn envelope_await_reply_serializes_context() {
        let json = Envelope::new().speak("which service?").await_reply("Q1").to_json();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["await_reply"]["context"], "Q1");
        assert_eq!(v["speak"], "which service?");
    }

    #[test]
    fn envelope_without_await_reply_omits_field() {
        let json = Envelope::new().speak("hi").to_json();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("await_reply").is_none());
    }

    #[test]
    fn list_card_serializes() {
        let list = ListCard::new()
            .summary(IconText::new("High 31° · Low 17° · Mostly cloudy").icon(Asset::new("icons/cloudy.png")))
            .row(ListRow::new("Wed").icon(Asset::new("icons/cloudy.png")).text("cloudy")
                    .trailing("24° / 18°").badge(IconText::new("43%").icon(Asset::new("ui/droplet.png"))))
            .row(ListRow::new("Thu").icon(Asset::new("icons/cloudy.png")).text("cloudy").trailing("28° / 17°"))
            .footer(IconText::new("Weather data by Open-Meteo.com").icon(Asset::new("ui/shield.png")));
        let json = Envelope::new()
            .card(Card::new("weather_forecast").title("London").subtitle("7-day forecast").list(list))
            .to_json();
        assert!(json.contains(r#""list":{"#));
        assert!(json.contains(r#""rows":[{"#));
        assert!(json.contains(r#""leading":"Wed""#));
        assert!(json.contains(r#""trailing":"24° / 18°""#));
        assert!(json.contains(r#""badge":{"icon":"asset:ui/droplet.png","text":"43%"}"#));
    }

    #[test]
    fn alarm_set_serialises_full() {
        let env = Envelope::new()
            .speak("Alarm set for 7am on weekdays.")
            .alarm(
                Alarm::set(7, 0)
                    .message("Wake up")
                    .days(&[Day::Mon, Day::Tue, Day::Wed, Day::Thu, Day::Fri]),
            );
        let v: serde_json::Value = serde_json::from_str(&env.to_json()).unwrap();
        assert_eq!(v["v"], 1);
        assert_eq!(v["alarm"]["op"], "set");
        assert_eq!(v["alarm"]["hour"], 7);
        assert_eq!(v["alarm"]["minute"], 0);
        assert_eq!(v["alarm"]["message"], "Wake up");
        assert_eq!(v["alarm"]["days"][0], "mon");
        assert_eq!(v["alarm"]["days"][4], "fri");
        assert_eq!(v["alarm"]["skip_ui"], true);
    }

    #[test]
    fn alarm_set_omits_optional_fields() {
        let env = Envelope::new().alarm(Alarm::set(6, 30));
        let v: serde_json::Value = serde_json::from_str(&env.to_json()).unwrap();
        assert_eq!(v["alarm"]["hour"], 6);
        assert_eq!(v["alarm"]["minute"], 30);
        assert!(v["alarm"].get("message").is_none());
        assert!(v["alarm"].get("days").is_none());
    }

    #[test]
    fn alarm_show_serialises_minimal() {
        let env = Envelope::new().alarm(Alarm::show());
        let v: serde_json::Value = serde_json::from_str(&env.to_json()).unwrap();
        assert_eq!(v["alarm"]["op"], "show");
        assert!(v["alarm"].get("hour").is_none());
    }

    #[test]
    fn navigate_serialises_with_mode() {
        let env = Envelope::new()
            .speak("Taking you to mcdonalds.")
            .navigate(Navigate::to("mcdonalds").mode("default_app"));
        let v: serde_json::Value = serde_json::from_str(&env.to_json()).unwrap();
        assert_eq!(v["v"], 1);
        assert_eq!(v["navigate"]["destination"], "mcdonalds");
        assert_eq!(v["navigate"]["mode"], "default_app");
    }

    #[test]
    fn navigate_omits_mode_when_unset() {
        let env = Envelope::new().navigate(Navigate::to("asda"));
        let v: serde_json::Value = serde_json::from_str(&env.to_json()).unwrap();
        assert_eq!(v["navigate"]["destination"], "asda");
        assert!(v["navigate"].get("mode").is_none());
    }
}
