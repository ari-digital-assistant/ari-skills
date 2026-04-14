#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use ari_skill_sdk as ari;

mod action;
mod parse;
mod state;

use action::{Envelope, Event};
use parse::Intent;
use state::{State, Timer};

#[cfg(target_arch = "wasm32")]
use state::STATE_KEY;

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

/// Plain-Rust entry point for unit tests. `wall_clock_ms` is the "now" used
/// for all relative time computation — tests inject a fixed value; the WASM
/// entry point passes `ari::now_ms()`.
#[cfg(any(test, not(target_arch = "wasm32")))]
pub fn handle_with_clock(input: &str, now_ms: i64, state_json: &str) -> (String, String) {
    let mut state = State::load(state_json);
    let (speak, events) = dispatch(input, now_ms, &mut state);
    let envelope = Envelope::new(speak, events, &state);
    (envelope.to_json(), state.serialise())
}

/// WASM-side entry: reads state from storage_kv, dispatches, writes state
/// back, returns the envelope JSON.
#[cfg(target_arch = "wasm32")]
fn handle(input: &str) -> String {
    let now = ari::now_ms();
    let raw = ari::storage_get(STATE_KEY).unwrap_or("");
    let mut state = State::load(raw);

    let (speak, events) = dispatch(input, now, &mut state);

    let serialised = state.serialise();
    if !ari::storage_set(STATE_KEY, &serialised) {
        ari::log(
            ari::LogLevel::Warn,
            "timer: storage_set failed; state not persisted",
        );
    }
    let envelope = Envelope::new(speak, events, &state);
    envelope.to_json()
}

fn dispatch(input: &str, now_ms: i64, state: &mut State) -> (String, Vec<Event>) {
    // Prune expired timers and surface each one as a cancel event so the
    // frontend can dismiss stale cards/notifications on the next utterance.
    let mut events: Vec<Event> = state
        .prune_expired(now_ms)
        .into_iter()
        .map(|id| Event::Cancel { id })
        .collect();

    match parse::classify(input) {
        Intent::Create(segments) => handle_create(segments, now_ms, state, &mut events),
        Intent::Query(name) => handle_query(name, now_ms, state, &mut events),
        Intent::Cancel(name) => handle_cancel(name, state, &mut events),
        Intent::CancelAll => handle_cancel_all(state, &mut events),
        Intent::List => handle_list(now_ms, state, &mut events),
        Intent::Unintelligible => (
            "Sorry, I couldn't work out what timer you meant.".to_string(),
            events,
        ),
    }
}

fn handle_create(
    segments: Vec<(Option<String>, u64)>,
    now_ms: i64,
    state: &mut State,
    events: &mut Vec<Event>,
) -> (String, Vec<Event>) {
    if segments.is_empty() {
        return (
            "I need a duration like '5 minutes' to set a timer.".to_string(),
            core::mem::take(events),
        );
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
        events.push(Event::Create {
            id: id.clone(),
            name: name.clone(),
            duration_ms,
            end_ts_ms,
            created_ts_ms: now_ms,
        });
        state.timers.push(timer);
        created_phrases.push(format!(
            "{} timer for {}",
            name.as_deref().unwrap_or("a"),
            describe_duration(duration_ms),
        ));
    }

    let speak = match created_phrases.len() {
        1 => format!("Set {}.", capitalise(&created_phrases[0])),
        _ => format!("Set {}.", join_with_and(&created_phrases)),
    };

    (speak, core::mem::take(events))
}

fn handle_query(
    name: Option<String>,
    now_ms: i64,
    state: &mut State,
    events: &mut Vec<Event>,
) -> (String, Vec<Event>) {
    events.push(Event::Ack);

    if state.timers.is_empty() {
        return ("No timers running.".to_string(), core::mem::take(events));
    }

    match name {
        Some(n) => match state.find_by_name(&n) {
            Some(t) => {
                let remaining = (t.end_ts_ms - now_ms).max(0) as u64;
                (
                    format!(
                        "{} timer has {} left.",
                        capitalise(&n),
                        describe_duration(remaining)
                    ),
                    core::mem::take(events),
                )
            }
            None => (
                format!("I couldn't find a timer called {}.", n),
                core::mem::take(events),
            ),
        },
        None => {
            if state.timers.len() == 1 {
                let t = &state.timers[0];
                let remaining = (t.end_ts_ms - now_ms).max(0) as u64;
                let prefix = match &t.name {
                    Some(n) => format!("{} timer", capitalise(n)),
                    None => "Your timer".to_string(),
                };
                (
                    format!("{} has {} left.", prefix, describe_duration(remaining)),
                    core::mem::take(events),
                )
            } else {
                // Ambiguous — list them for the user to disambiguate.
                (list_sentence(now_ms, state), core::mem::take(events))
            }
        }
    }
}

fn handle_cancel(
    name: Option<String>,
    state: &mut State,
    events: &mut Vec<Event>,
) -> (String, Vec<Event>) {
    match name {
        Some(n) => match state.remove_by_name(&n) {
            Some(id) => {
                events.push(Event::Cancel { id });
                (format!("Cancelled the {} timer.", n), core::mem::take(events))
            }
            None => {
                events.push(Event::Ack);
                (
                    format!("No {} timer to cancel.", n),
                    core::mem::take(events),
                )
            }
        },
        None => {
            if state.timers.len() == 1 {
                let id = state.timers.remove(0).id;
                events.push(Event::Cancel { id });
                ("Cancelled your timer.".to_string(), core::mem::take(events))
            } else if let Some(id) = state.remove_only_anonymous() {
                events.push(Event::Cancel { id });
                (
                    "Cancelled the anonymous timer.".to_string(),
                    core::mem::take(events),
                )
            } else {
                events.push(Event::Ack);
                (
                    "You have several timers. Which one should I cancel?".to_string(),
                    core::mem::take(events),
                )
            }
        }
    }
}

fn handle_cancel_all(
    state: &mut State,
    events: &mut Vec<Event>,
) -> (String, Vec<Event>) {
    if state.timers.is_empty() {
        events.push(Event::Ack);
        return ("No timers to cancel.".to_string(), core::mem::take(events));
    }
    let n = state.timers.len();
    state.timers.clear();
    events.push(Event::CancelAll);
    let phrase = if n == 1 { "1 timer" } else { "every timer" };
    (format!("Cancelled {}.", phrase), core::mem::take(events))
}

fn handle_list(
    now_ms: i64,
    state: &mut State,
    events: &mut Vec<Event>,
) -> (String, Vec<Event>) {
    events.push(Event::Ack);
    (list_sentence(now_ms, state), core::mem::take(events))
}

fn list_sentence(now_ms: i64, state: &State) -> String {
    if state.timers.is_empty() {
        return "No timers running.".to_string();
    }
    let phrases: Vec<String> = state
        .timers
        .iter()
        .map(|t| {
            let remaining = (t.end_ts_ms - now_ms).max(0) as u64;
            match &t.name {
                Some(n) => format!("{} ({} left)", n, describe_duration(remaining)),
                None => format!("an anonymous timer ({} left)", describe_duration(remaining)),
            }
        })
        .collect();
    format!("You have {}.", join_with_and(&phrases))
}

/// Human-friendly rendering like "3 minutes", "1 minute 30 seconds", "2
/// hours 15 minutes". Zero comes out as "0 seconds" rather than nothing.
fn describe_duration(ms: u64) -> String {
    let total_secs = ms / 1000;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    let mut parts: Vec<String> = Vec::new();
    if hours > 0 {
        parts.push(format!("{} {}", hours, plural("hour", hours)));
    }
    if minutes > 0 {
        parts.push(format!("{} {}", minutes, plural("minute", minutes)));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{} {}", seconds, plural("second", seconds)));
    }
    parts.join(" ")
}

fn plural(stem: &str, n: u64) -> String {
    if n == 1 {
        stem.to_string()
    } else {
        format!("{stem}s")
    }
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
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        2 => format!("{} and {}", items[0], items[1]),
        _ => {
            let head = items[..items.len() - 1].join(", ");
            format!("{}, and {}", head, items.last().unwrap())
        }
    }
}

/// 20-char id: "t_" + 16 hex chars from `rand_u64`. 64 bits is plenty for
/// uniqueness within any one user's timer set.
#[cfg(target_arch = "wasm32")]
fn new_id() -> String {
    let r = ari::rand_u64();
    format!("t_{:016x}", r)
}

/// Host-test equivalent. `std::time::SystemTime` + a process-local counter
/// is enough for uniqueness within a single test binary run.
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
    use std::string::ToString;

    fn handle_once(input: &str, now_ms: i64, state_json: &str) -> (serde_json::Value, String) {
        let (envelope_json, state_json_out) = handle_with_clock(input, now_ms, state_json);
        let value: serde_json::Value = serde_json::from_str(&envelope_json).unwrap();
        (value, state_json_out)
    }

    #[test]
    fn create_emits_create_event_and_persists() {
        let (env, state_json) = handle_once("set a pasta timer for 8 minutes", 1_000_000, "");
        assert_eq!(env["action"], "timer");
        assert_eq!(env["speak"], "Set Pasta timer for 8 minutes.");
        let events = env["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["kind"], "create");
        assert_eq!(events[0]["name"], "pasta");
        assert_eq!(events[0]["duration_ms"], 480_000);
        assert_eq!(events[0]["end_ts_ms"], 1_480_000);

        // Timers array has one entry, state JSON round-trips.
        assert_eq!(env["timers"].as_array().unwrap().len(), 1);
        let reparsed: serde_json::Value = serde_json::from_str(&state_json).unwrap();
        assert_eq!(reparsed["timers"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn create_adjective_form_parses_same_as_prepositional() {
        let (env_adj, _) = handle_once("set a 4 minute pasta timer", 0, "");
        let (env_prep, _) = handle_once("set a pasta timer for 4 minutes", 0, "");
        assert_eq!(
            env_adj["events"][0]["duration_ms"],
            env_prep["events"][0]["duration_ms"]
        );
        assert_eq!(env_adj["events"][0]["name"], env_prep["events"][0]["name"]);
        assert_eq!(env_adj["events"][0]["duration_ms"], 240_000);
        assert_eq!(env_adj["events"][0]["name"], "pasta");
    }

    #[test]
    fn multi_create_emits_two_events() {
        let (env, state_json) = handle_once(
            "set a timer for 5 minutes and another for 15 minutes",
            0,
            "",
        );
        let events = env["events"].as_array().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["kind"], "create");
        assert_eq!(events[1]["kind"], "create");
        assert_eq!(events[0]["duration_ms"], 300_000);
        assert_eq!(events[1]["duration_ms"], 900_000);
        assert_eq!(env["timers"].as_array().unwrap().len(), 2);
        let state: serde_json::Value = serde_json::from_str(&state_json).unwrap();
        assert_eq!(state["timers"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn query_returns_remaining_in_speak_field() {
        let (_, state_after_create) = handle_once("set a pasta timer for 8 minutes", 0, "");
        let (env, _) = handle_once(
            "how much time is left on my pasta timer",
            300_000, // 5 minutes in
            &state_after_create,
        );
        assert_eq!(env["events"][0]["kind"], "ack");
        assert_eq!(env["speak"], "Pasta timer has 3 minutes left.");
    }

    #[test]
    fn cancel_named_removes_from_state() {
        let (_, s1) = handle_once("set a pasta timer for 8 minutes", 0, "");
        let (_, s2) = handle_once("set an egg timer for 3 minutes", 0, &s1);
        let (env, s3) = handle_once("cancel my pasta timer", 0, &s2);

        assert_eq!(env["events"][0]["kind"], "cancel");
        assert_eq!(env["speak"], "Cancelled the pasta timer.");
        let state: serde_json::Value = serde_json::from_str(&s3).unwrap();
        let timers = state["timers"].as_array().unwrap();
        assert_eq!(timers.len(), 1);
        assert_eq!(timers[0]["name"], "egg");
    }

    #[test]
    fn cancel_all_empties_state() {
        let (_, s1) = handle_once("set a pasta timer for 8 minutes", 0, "");
        let (_, s2) = handle_once("set an egg timer for 3 minutes", 0, &s1);
        let (env, s3) = handle_once("cancel all timers", 0, &s2);

        assert_eq!(env["events"][0]["kind"], "cancel_all");
        let state: serde_json::Value = serde_json::from_str(&s3).unwrap();
        assert!(state["timers"].as_array().unwrap().is_empty());
    }

    #[test]
    fn list_enumerates_active_timers() {
        let (_, s1) = handle_once("set a pasta timer for 8 minutes", 0, "");
        let (_, s2) = handle_once("set an egg timer for 3 minutes", 0, &s1);
        let (env, _) = handle_once("what timers do i have", 60_000, &s2);
        assert_eq!(env["events"][0]["kind"], "ack");
        // Ordering is insertion order: pasta, then egg.
        assert!(env["speak"]
            .as_str()
            .unwrap()
            .contains("pasta (7 minutes left)"));
        assert!(env["speak"]
            .as_str()
            .unwrap()
            .contains("egg (2 minutes left)"));
    }

    #[test]
    fn expired_timers_are_pruned_and_reported_as_cancels() {
        let (_, s1) = handle_once("set a pasta timer for 1 minute", 0, "");
        // Way past expiry — the prune pass should emit a cancel for the
        // pasta timer BEFORE handling the new utterance.
        let (env, _) = handle_once("what timers do i have", 120_000, &s1);
        let events = env["events"].as_array().unwrap();
        let cancels: Vec<&serde_json::Value> =
            events.iter().filter(|e| e["kind"] == "cancel").collect();
        assert_eq!(cancels.len(), 1, "expected one prune-cancel event");
        // And the list reflects the empty state.
        assert_eq!(env["speak"], "No timers running.");
    }

    #[test]
    fn storage_state_survives_garbage() {
        // A corrupt storage_kv blob must not brick the skill.
        let (env, _) = handle_once("set a timer for 30 seconds", 0, "not json");
        assert_eq!(env["action"], "timer");
        assert_eq!(env["events"][0]["kind"], "create");
    }

    #[test]
    fn compound_duration_sums_correctly() {
        let (env, _) = handle_once("set a timer for 1 hour and 30 minutes", 0, "");
        assert_eq!(env["events"][0]["duration_ms"], 5_400_000);
        assert_eq!(env["speak"], "Set A timer for 1 hour 30 minutes.");
    }
}
