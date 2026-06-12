#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use ari_skill_sdk as ari;
use ari_skill_sdk::presentation as p;

mod parse;
mod state;

use parse::Intent;
use state::{State, Timer};

#[cfg(target_arch = "wasm32")]
use state::STATE_KEY;

// ---------------------------------------------------------------------------
// Thin i18n shim — on wasm32 we go through the host's strings table;
// on native (unit tests) we always return None so every call site falls
// back to the English literal it already carries.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
#[inline]
fn t(key: &str, args: &[(&str, &str)]) -> Option<&'static str> {
    ari::t(key, args)
}

#[cfg(not(target_arch = "wasm32"))]
#[inline]
fn t(_key: &str, _args: &[(&str, &str)]) -> Option<&'static str> {
    None
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    // custom_score: false in the manifest → engine uses the pattern scorer
    // and this entry point is never called. Returned value is a shrug for
    // completeness if a future config ever enables custom scoring.
    0.9
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    let json = handle(input);
    ari::respond_action(&json)
}

/// Plain-Rust entry point for unit tests. `now_ms` is injected so tests
/// don't depend on wall-clock; the WASM entry point passes `ari::now_ms()`.
#[cfg(any(test, not(target_arch = "wasm32")))]
pub fn handle_with_clock(input: &str, now_ms: i64, state_json: &str) -> (String, String) {
    let mut state = State::load(state_json);
    let envelope_json = dispatch(input, now_ms, &mut state);
    (envelope_json, state.serialise())
}

/// WASM-side entry: reads state from storage_kv, dispatches, writes back,
/// returns the envelope JSON for `respond_action`.
#[cfg(target_arch = "wasm32")]
fn handle(input: &str) -> String {
    let now = ari::now_ms();
    let raw = ari::storage_get(STATE_KEY).unwrap_or("");
    let mut state = State::load(raw);

    let envelope_json = dispatch(input, now, &mut state);

    let serialised = state.serialise();
    if !ari::storage_set(STATE_KEY, &serialised) {
        ari::log(
            ari::LogLevel::Warn,
            "timer: storage_set failed; state not persisted",
        );
    }
    envelope_json
}

fn dispatch(input: &str, now_ms: i64, state: &mut State) -> String {
    // Prune expired timers and surface dismissals for any cards/notifs/alerts
    // we previously asked the frontend to show. Self-healing across app kills
    // and missed alarm fires.
    let pruned_ids: Vec<String> = state.prune_expired(now_ms);

    let mut envelope = p::Envelope::new();
    for id in &pruned_ids {
        envelope = envelope
            .dismiss_card(card_id_for(id))
            .dismiss_notification(notif_id_for(id))
            .dismiss_alert(alert_id_for(id));
    }

    match parse::classify(input) {
        Intent::Create(segments) => handle_create(segments, now_ms, state, envelope),
        Intent::Query(name) => handle_query(name, now_ms, state, envelope),
        Intent::Cancel(name) => handle_cancel(name, state, envelope),
        Intent::CancelAll => handle_cancel_all(state, envelope),
        Intent::List => handle_list(now_ms, state, envelope),
        Intent::Unintelligible => envelope
            .speak(
                t("error.unintelligible", &[])
                    .unwrap_or("Sorry, I couldn't work out what timer you meant.")
                    .to_string(),
            )
            .to_json(),
    }
}

fn handle_create(
    segments: Vec<(Option<String>, u64)>,
    now_ms: i64,
    state: &mut State,
    mut envelope: p::Envelope,
) -> String {
    if segments.is_empty() {
        return envelope
            .speak(
                t("error.no_duration", &[])
                    .unwrap_or("I need a duration like '5 minutes' to set a timer.")
                    .to_string(),
            )
            .to_json();
    }

    let mut created_phrases: Vec<String> = Vec::new();
    for (name, duration_ms) in segments {
        let id = new_id();
        let end_ts_ms = now_ms.saturating_add(duration_ms as i64);
        let timer = Timer {
            id: id.clone(),
            name: name.clone(),
            end_ts_ms,
            created_ts_ms: now_ms,
        };
        envelope = envelope
            .card(build_card(&id, &name, end_ts_ms, now_ms))
            .notification(build_notification(&id, &name, end_ts_ms));
        state.timers.push(timer);
        let dur = describe_duration(duration_ms);
        let phrase = match &name {
            Some(n) => t("create.phrase_named", &[("name", n), ("time", &dur)])
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{} timer for {}", n, dur)),
            None => t("create.phrase_anonymous", &[("time", &dur)])
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("a timer for {}", dur)),
        };
        created_phrases.push(phrase);
    }

    let raw_phrase = match created_phrases.len() {
        1 => capitalise(&created_phrases[0]),
        _ => join_with_and(&created_phrases),
    };
    let speak = t("create.success", &[("phrase", &raw_phrase)])
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Set {}.", raw_phrase));
    envelope.speak(speak).to_json()
}

fn handle_query(
    name: Option<String>,
    now_ms: i64,
    state: &State,
    envelope: p::Envelope,
) -> String {
    if state.timers.is_empty() {
        return envelope
            .speak(
                t("query.none", &[])
                    .unwrap_or("No timers running.")
                    .to_string(),
            )
            .to_json();
    }

    let speak = match name {
        Some(n) => match state.find_by_name(&n) {
            Some(ti) => {
                let remaining = (ti.end_ts_ms - now_ms).max(0) as u64;
                let dur = describe_duration(remaining);
                // Named query: "{Name} timer has {time} left."
                // We build the full name prefix and pass it as `name`.
                let title = t("query.named_title", &[("name", &capitalise(&n))])
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{} timer", capitalise(&n)));
                t("query.remaining", &[("name", &title), ("time", &dur)])
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{} has {} left.", title, dur))
            }
            None => t("query.not_found", &[("name", &n)])
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("I couldn't find a timer called {}.", n)),
        },
        None => {
            if state.timers.len() == 1 {
                let ti = &state.timers[0];
                let remaining = (ti.end_ts_ms - now_ms).max(0) as u64;
                let dur = describe_duration(remaining);
                let prefix = match &ti.name {
                    Some(n) => t("query.named_title", &[("name", &capitalise(n))])
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("{} timer", capitalise(n))),
                    None => t("query.anonymous_prefix", &[])
                        .unwrap_or("Your timer")
                        .to_string(),
                };
                t("query.remaining", &[("name", &prefix), ("time", &dur)])
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{} has {} left.", prefix, dur))
            } else {
                list_sentence(now_ms, state)
            }
        }
    };
    envelope.speak(speak).to_json()
}

fn handle_cancel(
    name: Option<String>,
    state: &mut State,
    mut envelope: p::Envelope,
) -> String {
    let speak = match name {
        Some(n) => match state.remove_by_name(&n) {
            Some(id) => {
                envelope = dismiss_all_for(envelope, &id);
                t("cancel.named_success", &[("name", &n)])
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("Cancelled the {} timer.", n))
            }
            None => t("cancel.named_not_found", &[("name", &n)])
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("No {} timer to cancel.", n)),
        },
        None => {
            if state.timers.len() == 1 {
                let id = state.timers.remove(0).id;
                envelope = dismiss_all_for(envelope, &id);
                t("cancel.anonymous_success", &[])
                    .unwrap_or("Cancelled your timer.")
                    .to_string()
            } else if let Some(id) = state.remove_only_anonymous() {
                envelope = dismiss_all_for(envelope, &id);
                t("cancel.the_anonymous", &[])
                    .unwrap_or("Cancelled the anonymous timer.")
                    .to_string()
            } else {
                t("cancel.ambiguous", &[])
                    .unwrap_or("You have several timers. Which one should I cancel?")
                    .to_string()
            }
        }
    };
    envelope.speak(speak).to_json()
}

fn handle_cancel_all(state: &mut State, mut envelope: p::Envelope) -> String {
    if state.timers.is_empty() {
        return envelope
            .speak(
                t("cancel.none", &[])
                    .unwrap_or("No timers to cancel.")
                    .to_string(),
            )
            .to_json();
    }
    let n = state.timers.len();
    let ids: Vec<String> = state.timers.iter().map(|ti| ti.id.clone()).collect();
    state.timers.clear();
    for id in &ids {
        envelope = dismiss_all_for(envelope, id);
    }
    let summary = if n == 1 {
        t("cancel.all.one", &[]).unwrap_or("1 timer").to_string()
    } else {
        t("cancel.all.many", &[]).unwrap_or("every timer").to_string()
    };
    let speak = t("cancel.all_success", &[("summary", &summary)])
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Cancelled {}.", summary));
    envelope.speak(speak).to_json()
}

fn handle_list(now_ms: i64, state: &State, envelope: p::Envelope) -> String {
    envelope.speak(list_sentence(now_ms, state)).to_json()
}

fn dismiss_all_for(envelope: p::Envelope, timer_id: &str) -> p::Envelope {
    envelope
        .dismiss_card(card_id_for(timer_id))
        .dismiss_notification(notif_id_for(timer_id))
        .dismiss_alert(alert_id_for(timer_id))
}

fn build_card(timer_id: &str, name: &Option<String>, end_ts_ms: i64, started_ts_ms: i64) -> p::Card {
    let title = match name.as_deref() {
        Some(n) => t("query.named_title", &[("name", &capitalise(n))])
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{} timer", capitalise(n))),
        None => t("card.anonymous_title", &[])
            .unwrap_or("Timer")
            .to_string(),
    };
    p::Card::new(card_id_for(timer_id))
        .title(title)
        .icon(p::Asset::new("timer_icon.png"))
        .countdown_to(end_ts_ms)
        .started_at(started_ts_ms)
        .action(
            p::Action::new(
                "cancel",
                t("action.cancel_label", &[]).unwrap_or("Cancel"),
            )
            .utterance(cancel_utterance(name))
            .destructive(),
        )
        .on_complete(
            p::OnComplete::new()
                .alert(build_alert(timer_id, name))
                .dismiss_card(true)
                // Dismiss the paired ongoing shade notification at the
                // same instant the alert fires — without this the
                // notification ticks past zero (counting up) until the
                // next user utterance prunes it.
                .dismiss_notification(notif_id_for(timer_id)),
        )
}

fn build_alert(timer_id: &str, name: &Option<String>) -> p::Alert {
    let title = match name.as_deref() {
        Some(n) => t("alert.named_done", &[("name", &capitalise(n))])
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{} timer done", capitalise(n))),
        None => t("alert.anonymous_done", &[])
            .unwrap_or("Timer done")
            .to_string(),
    };
    let speech = name
        .as_deref()
        .map(|n| {
            t("query.named_title", &[("name", &capitalise(n))])
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{} timer", capitalise(n)))
        });
    let mut alert = p::Alert::new(alert_id_for(timer_id))
        .title(title)
        .urgency(p::Urgency::Critical)
        .sound(p::Sound::asset("timer.mp3"))
        .auto_stop_ms(120_000)
        .max_cycles(12)
        .full_takeover(true)
        .icon(p::Asset::new("timer_icon.png"))
        .action(
            p::Action::new(
                "stop_alert",
                t("action.stop_label", &[]).unwrap_or("Stop"),
            )
            .primary(),
        );
    if let Some(s) = speech {
        alert = alert.speech_loop(s);
    }
    alert
}

fn build_notification(timer_id: &str, name: &Option<String>, end_ts_ms: i64) -> p::Notification {
    let title = match name.as_deref() {
        Some(n) => t("query.named_title", &[("name", &capitalise(n))])
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{} timer", capitalise(n))),
        None => t("card.anonymous_title", &[])
            .unwrap_or("Timer")
            .to_string(),
    };
    p::Notification::new(notif_id_for(timer_id))
        .title(title)
        .body(
            t("notification.running", &[])
                .unwrap_or("Running…")
                .to_string(),
        )
        .importance(p::Importance::Default)
        .sticky(true)
        .countdown_to(end_ts_ms)
        .action(
            p::Action::new(
                "cancel",
                t("action.cancel_label", &[]).unwrap_or("Cancel"),
            )
            .utterance(cancel_utterance(name)),
        )
}

fn cancel_utterance(name: &Option<String>) -> String {
    match name {
        Some(n) => t("action.cancel_named_utterance", &[("name", n)])
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("cancel my {} timer", n)),
        None => t("action.cancel_anonymous_utterance", &[])
            .unwrap_or("cancel my timer")
            .to_string(),
    }
}

fn card_id_for(timer_id: &str) -> String {
    format!("card_{timer_id}")
}

fn notif_id_for(timer_id: &str) -> String {
    format!("notif_{timer_id}")
}

fn alert_id_for(timer_id: &str) -> String {
    format!("alert_{timer_id}")
}

fn list_sentence(now_ms: i64, state: &State) -> String {
    if state.timers.is_empty() {
        return t("query.none", &[])
            .unwrap_or("No timers running.")
            .to_string();
    }
    let phrases: Vec<String> = state
        .timers
        .iter()
        .map(|ti| {
            let remaining = (ti.end_ts_ms - now_ms).max(0) as u64;
            let dur = describe_duration(remaining);
            match &ti.name {
                Some(n) => t("list.named_entry", &[("name", n), ("time", &dur)])
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{} ({} left)", n, dur)),
                None => t("list.anonymous_entry", &[("time", &dur)])
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("an anonymous timer ({} left)", dur)),
            }
        })
        .collect();
    let items = join_with_and(&phrases);
    t("list.sentence", &[("items", &items)])
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("You have {}.", items))
}

/// "3 minutes", "1 minute 30 seconds", "2 hours 15 minutes".
/// Zero comes out as "0 seconds" rather than nothing.
fn describe_duration(ms: u64) -> String {
    let total_secs = ms / 1000;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    let mut parts: Vec<String> = Vec::new();
    if hours > 0 {
        parts.push(unit(
            hours,
            "duration.hour_singular",
            "duration.hour_plural",
            "hour",
            "hours",
        ));
    }
    if minutes > 0 {
        parts.push(unit(
            minutes,
            "duration.minute_singular",
            "duration.minute_plural",
            "minute",
            "minutes",
        ));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(unit(
            seconds,
            "duration.second_singular",
            "duration.second_plural",
            "second",
            "seconds",
        ));
    }
    parts.join(" ")
}

/// Format a count with its localised singular or plural unit word.
fn unit(n: u64, sing_key: &str, plur_key: &str, sing_fb: &str, plur_fb: &str) -> String {
    let word = if n == 1 {
        t(sing_key, &[]).unwrap_or(sing_fb)
    } else {
        t(plur_key, &[]).unwrap_or(plur_fb)
    };
    format!("{} {}", n, word)
}

fn capitalise(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => {
            let mut out = c.to_uppercase().collect::<String>();
            out.push_str(chars.as_str());
            out
        }
        None => String::new(),
    }
}

fn join_with_and(items: &[String]) -> String {
    let joiner = t("list.join", &[]).unwrap_or(" and ");
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        2 => format!("{}{}{}", items[0], joiner, items[1]),
        _ => {
            let head = items[..items.len() - 1].join(", ");
            format!("{},{}{}", head, joiner, items.last().unwrap())
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn new_id() -> String {
    let r = ari::rand_u64();
    format!("t_{:016x}", r)
}

#[cfg(not(target_arch = "wasm32"))]
fn new_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    format!("t_{:016x}", nanos ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15))
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use serde_json::Value;

    fn handle_once(input: &str, now_ms: i64, state_json: &str) -> (Value, String) {
        let (envelope_json, state_json_out) = handle_with_clock(input, now_ms, state_json);
        let value: Value = serde_json::from_str(&envelope_json).unwrap();
        (value, state_json_out)
    }

    #[test]
    fn envelope_carries_v_1() {
        let (env, _) = handle_once("set a timer for 30 seconds", 0, "");
        assert_eq!(env["v"], 1);
    }

    #[test]
    fn create_emits_card_with_countdown_and_on_complete_alert() {
        let (env, state_json) = handle_once("set a pasta timer for 8 minutes", 1_000_000, "");
        assert_eq!(env["speak"], "Set Pasta timer for 8 minutes.");
        let cards = env["cards"].as_array().unwrap();
        assert_eq!(cards.len(), 1);
        let card = &cards[0];
        // id mapping
        let timer_id = serde_json::from_str::<Value>(&state_json).unwrap()["timers"][0]["id"]
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!(card["id"], format!("card_{timer_id}"));
        assert_eq!(card["title"], "Pasta timer");
        assert_eq!(card["icon"], "asset:timer_icon.png");
        assert_eq!(card["countdown_to_ts_ms"], 1_480_000);
        assert_eq!(card["started_at_ts_ms"], 1_000_000);
        // cancel action
        assert_eq!(card["actions"][0]["id"], "cancel");
        assert_eq!(card["actions"][0]["utterance"], "cancel my pasta timer");
        assert_eq!(card["actions"][0]["style"], "destructive");
        // on_complete.alert
        let alert = &card["on_complete"]["alert"];
        assert_eq!(alert["id"], format!("alert_{timer_id}"));
        assert_eq!(alert["urgency"], "critical");
        assert_eq!(alert["sound"], "asset:timer.mp3");
        assert_eq!(alert["speech_loop"], "Pasta timer");
        assert_eq!(alert["full_takeover"], true);
        // Glyph for the takeover screen — same asset the card uses, so
        // the timer reads as "the timer" wherever it surfaces.
        assert_eq!(alert["icon"], "asset:timer_icon.png");
        assert_eq!(alert["actions"][0]["id"], "stop_alert");
        assert_eq!(card["on_complete"]["dismiss_card"], true);
        // The card's on_complete must dismiss the matching ongoing
        // notification at expiry — without this the shade entry ticks
        // past zero (counting up) until the user prompts again.
        assert_eq!(
            card["on_complete"]["dismiss_notifications"][0],
            format!("notif_{timer_id}"),
        );
    }

    #[test]
    fn create_emits_matching_notification() {
        let (env, _) = handle_once("set a pasta timer for 8 minutes", 1_000_000, "");
        let notifs = env["notifications"].as_array().unwrap();
        assert_eq!(notifs.len(), 1);
        let n = &notifs[0];
        assert_eq!(n["title"], "Pasta timer");
        assert_eq!(n["countdown_to_ts_ms"], 1_480_000);
        assert_eq!(n["sticky"], true);
        assert_eq!(n["actions"][0]["utterance"], "cancel my pasta timer");
    }

    #[test]
    fn anonymous_timer_omits_speech_loop() {
        let (env, _) = handle_once("set a timer for 30 seconds", 0, "");
        let alert = &env["cards"][0]["on_complete"]["alert"];
        assert!(
            alert.get("speech_loop").is_none(),
            "anonymous timer alert must not have a speech_loop"
        );
    }

    #[test]
    fn create_adjective_form_parses_same_as_prepositional() {
        let (env_adj, _) = handle_once("set a 4 minute pasta timer", 0, "");
        let (env_prep, _) = handle_once("set a pasta timer for 4 minutes", 0, "");
        assert_eq!(
            env_adj["cards"][0]["countdown_to_ts_ms"],
            env_prep["cards"][0]["countdown_to_ts_ms"],
        );
        assert_eq!(env_adj["cards"][0]["title"], "Pasta timer");
        assert_eq!(env_adj["cards"][0]["countdown_to_ts_ms"], 240_000);
    }

    #[test]
    fn multi_create_emits_two_cards_two_notifications() {
        let (env, _) = handle_once(
            "set a timer for 5 minutes and another for 15 minutes",
            0,
            "",
        );
        assert_eq!(env["cards"].as_array().unwrap().len(), 2);
        assert_eq!(env["notifications"].as_array().unwrap().len(), 2);
        assert_eq!(env["cards"][0]["countdown_to_ts_ms"], 300_000);
        assert_eq!(env["cards"][1]["countdown_to_ts_ms"], 900_000);
    }

    #[test]
    fn query_emits_speak_only_no_cards() {
        let (_, s1) = handle_once("set a pasta timer for 8 minutes", 0, "");
        let (env, _) = handle_once("how much time is left on my pasta timer", 300_000, &s1);
        assert_eq!(env["speak"], "Pasta timer has 3 minutes left.");
        // Query is read-only: no cards/notifications/alerts/dismissals.
        assert!(env.get("cards").is_none());
        assert!(env.get("notifications").is_none());
        assert!(env.get("dismiss").is_none());
    }

    #[test]
    fn cancel_dismisses_all_three_id_forms() {
        let (_, s1) = handle_once("set a pasta timer for 8 minutes", 0, "");
        let timer_id = serde_json::from_str::<Value>(&s1).unwrap()["timers"][0]["id"]
            .as_str()
            .unwrap()
            .to_string();
        let (env, s2) = handle_once("cancel my pasta timer", 0, &s1);
        assert_eq!(env["speak"], "Cancelled the pasta timer.");
        let dismiss = &env["dismiss"];
        assert_eq!(dismiss["cards"][0], format!("card_{timer_id}"));
        assert_eq!(dismiss["notifications"][0], format!("notif_{timer_id}"));
        assert_eq!(dismiss["alerts"][0], format!("alert_{timer_id}"));
        // State now empty.
        let state: Value = serde_json::from_str(&s2).unwrap();
        assert!(state["timers"].as_array().unwrap().is_empty());
    }

    #[test]
    fn cancel_all_dismisses_every_tracked_id() {
        let (_, s1) = handle_once("set a pasta timer for 8 minutes", 0, "");
        let (_, s2) = handle_once("set an egg timer for 3 minutes", 0, &s1);
        let (env, _) = handle_once("cancel all timers", 0, &s2);
        let dismiss = &env["dismiss"];
        assert_eq!(dismiss["cards"].as_array().unwrap().len(), 2);
        assert_eq!(dismiss["notifications"].as_array().unwrap().len(), 2);
        assert_eq!(dismiss["alerts"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn list_emits_speak_only() {
        let (_, s1) = handle_once("set a pasta timer for 8 minutes", 0, "");
        let (_, s2) = handle_once("set an egg timer for 3 minutes", 0, &s1);
        let (env, _) = handle_once("what timers do i have", 60_000, &s2);
        let speak = env["speak"].as_str().unwrap();
        assert!(speak.contains("pasta (7 minutes left)"));
        assert!(speak.contains("egg (2 minutes left)"));
        // Cards aren't re-emitted on list — they already exist from create.
        assert!(env.get("cards").is_none());
    }

    #[test]
    fn expired_prune_emits_dismiss() {
        let (_, s1) = handle_once("set a pasta timer for 1 minute", 0, "");
        let timer_id = serde_json::from_str::<Value>(&s1).unwrap()["timers"][0]["id"]
            .as_str()
            .unwrap()
            .to_string();
        // Way past expiry — prune fires before any handler.
        let (env, _) = handle_once("what timers do i have", 120_000, &s1);
        let dismiss_cards = env["dismiss"]["cards"].as_array().unwrap();
        assert_eq!(dismiss_cards.len(), 1);
        assert_eq!(dismiss_cards[0], format!("card_{timer_id}"));
        assert_eq!(env["speak"], "No timers running.");
    }

    #[test]
    fn storage_state_survives_garbage() {
        // Corrupt storage_kv blob must not brick the skill.
        let (env, _) = handle_once("set a timer for 30 seconds", 0, "not json");
        assert_eq!(env["v"], 1);
        assert_eq!(env["cards"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn compound_duration_countdown_matches_end_ts() {
        let (env, _) = handle_once("set a timer for 1 hour and 30 minutes", 0, "");
        // 1h30m = 5400000 ms
        assert_eq!(env["cards"][0]["countdown_to_ts_ms"], 5_400_000);
        assert_eq!(env["speak"], "Set A timer for 1 hour 30 minutes.");
    }
}
