//! Ari reminder skill.
//!
//! Parses utterances like "remind me to walk the dog at 5pm" and "add
//! milk to my shopping list" into a tasks-provider insert or
//! calendar-event insert, using only generic host capabilities. The
//! skill does everything itself:
//!
//!   - Intent extraction (parse.rs).
//!   - Local timezone / weekday resolution via `ari::local_now_components`.
//!   - Reading user settings (destination / default list) via
//!     `ari::setting_get`.
//!   - Calling `ari::tasks_insert` / `ari::calendar_insert` directly.
//!   - Emitting a card with `on_cancel: { run_utterance: ... }` for
//!     partial-confidence flows; the frontend bounces the cancel
//!     utterance back through the engine, which routes here and
//!     calls `ari::tasks_delete` / `ari::calendar_delete`.
//!
//! Zero frontend-side reminder-specific code needed. The same WASM
//! module runs on Android today and ari-linux tomorrow.
//!
//! See SKILL.md for the supported utterance shapes.

#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};

#[cfg(target_arch = "wasm32")]
use ari_skill_sdk as ari;

mod layer_c;
mod parse;

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    // custom_score: false in the manifest — engine uses the regex
    // pattern scorer and never calls into here. Returned value is a
    // shrug for completeness.
    0.9
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    let envelope = dispatch(input);
    ari::respond_action(&envelope)
}

/// Entry point. Four branches:
/// 1. **Layer C continuation** — the engine bypasses `normalize_input`
///    and hands us `{"_ari_continuation":{"context":...,"response":...}}`.
///    The skill owns all post-assistant logic (AI's answer in hand;
///    decide between commit, clarification card, or fallback).
/// 2. **Clarification confirm** — Yes button on a clarification card
///    emits `ariconfirmreminder <dest> <epoch_ms> <title_hex>`. Commit
///    with the pre-staged values.
/// 3. **Cancel round-trip** — Keep/Cancel card's `on_cancel` utterance
///    (`aricancelreminder <mode> <id>`). Delete the row that the
///    fallback path inserted.
/// 4. **Normal utterance** — parse, then `High` confidence commits
///    immediately; `Partial`/`Low` emits a `consult_assistant`
///    directive so the engine runs an assistant round-trip and re-
///    enters branch 1.
#[cfg(target_arch = "wasm32")]
pub fn dispatch(input: &str) -> String {
    if let Some(cont) = layer_c::parse_continuation_input(input) {
        ari::log(
            ari::LogLevel::Info,
            &format!(
                "handle_continuation context_len={} response_len={}",
                cont.context.len(),
                cont.response.len()
            ),
        );
        return handle_continuation(cont);
    }
    if let Some(confirm) = layer_c::parse_confirm(input) {
        ari::log(
            ari::LogLevel::Info,
            &format!(
                "handle_confirm destination={} epoch_ms={} title={:?}",
                confirm.destination, confirm.epoch_ms, confirm.title
            ),
        );
        return handle_confirm(confirm);
    }
    if let Some(cancel) = parse_internal_cancel(input) {
        // Log the cancel round-trip at skill-log level so
        // `adb logcat -s AriSkill` shows both the create and the
        // cancel sides of a partial-confidence flow. One line per
        // user-visible state change — not per utterance.
        ari::log(
            ari::LogLevel::Info,
            &format!(
                "handle_cancel mode={} id={}",
                cancel.mode.as_str(),
                cancel.id,
            ),
        );
        return handle_cancel(cancel);
    }
    let parsed = parse::parse(input);
    ari::log(
        ari::LogLevel::Info,
        &format!(
            "parse confidence={} unparsed={:?} title={:?}",
            parsed.confidence.as_envelope_str(),
            parsed.unparsed.as_deref().unwrap_or(""),
            parsed.title,
        ),
    );

    // Confidence-gated routing. Layer C v2 defers the commit when the
    // parser isn't sure: we emit a `consult_assistant` envelope so the
    // engine runs the assistant round-trip, and the continuation
    // handler writes the reminder only once the assistant confirms.
    // High-confidence parses short-circuit to the classic immediate
    // commit.
    match parsed.confidence {
        parse::Confidence::High => handle_create(&parsed),
        parse::Confidence::Partial | parse::Confidence::Low => {
            build_consult_assistant_envelope(input, &parsed)
        }
    }
}

/// Host-side stub so unit tests that test parse() in isolation still
/// link. Production dispatch requires host imports (tasks, calendar,
/// clock, settings) that only exist on wasm32. The parse-level tests
/// in `parse.rs` cover the parsing logic end-to-end.
#[cfg(not(target_arch = "wasm32"))]
pub fn dispatch(_input: &str) -> String {
    String::from(r#"{"v":1,"speak":"reminder skill requires the wasm32 target"}"#)
}

// ── Internal-utterance cancel protocol ────────────────────────────
//
// A partial-confidence card carries an `on_cancel` envelope whose
// `run_utterance` is a deterministic string we recognise as
// "cancel the <mode> row <id> I just inserted". Routing through the
// engine + skill matching keeps the cancel flow entirely
// frontend-independent — any frontend that implements `on_cancel`
// handling + utterance round-trip works.
//
// Format: `aricancelreminder <mode> <id>` — space-separated tokens,
// all alphanumeric. The engine's `normalize_input` strips non-
// alphanumeric characters (underscores, colons) to spaces before
// matching, so earlier formats like `__ari_cancel_reminder:tasks:42`
// got mangled into `ari cancel reminder tasks 42` at routing time
// and the skill's regex never fired. Keeping the prefix as one
// contiguous word dodges the normaliser entirely.

struct InternalCancel {
    mode: Mode,
    id: u64,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Tasks,
    Calendar,
}

impl Mode {
    fn as_str(&self) -> &'static str {
        match self {
            Mode::Tasks => "tasks",
            Mode::Calendar => "calendar",
        }
    }
}

fn parse_internal_cancel(input: &str) -> Option<InternalCancel> {
    // Tolerant of leading whitespace (the engine's normaliser may
    // emit it) and extra trailing tokens, but the first three must
    // be exactly `aricancelreminder <mode> <id>`.
    let mut tokens = input.trim().split_whitespace();
    if tokens.next()? != "aricancelreminder" {
        return None;
    }
    let mode = match tokens.next()? {
        "tasks" => Mode::Tasks,
        "calendar" => Mode::Calendar,
        _ => return None,
    };
    let id: u64 = tokens.next()?.parse().ok()?;
    Some(InternalCancel { mode, id })
}

#[cfg(target_arch = "wasm32")]
fn handle_cancel(cancel: InternalCancel) -> String {
    let deleted = match cancel.mode {
        Mode::Tasks => ari::tasks_delete(cancel.id),
        Mode::Calendar => ari::calendar_delete(cancel.id),
    };
    let speak = if deleted {
        "OK, cancelled that."
    } else {
        "I couldn't find that to cancel — it might have already been removed."
    };
    let mut out = String::from("{\"v\":1,\"speak\":");
    push_json_string(&mut out, speak);
    out.push('}');
    out
}

// ── Layer C phase-1 envelope ──────────────────────────────────────

/// Build the phase-1 envelope the engine intercepts. Deliberately
/// silent — no `speak`, no cards. Most cloud-assistant round-trips
/// finish in a couple of seconds, faster than a TTS ack would even
/// finish playing, so saying "let me check..." just delays the real
/// answer. If the round-trip turns out to be slow (>4s) the engine
/// pushes a delay phrase (`Hang on...`, `One moment...`, etc.) on
/// its own.
///
/// The envelope only carries the `consult_assistant` directive. The
/// engine strips that field before returning, so what reaches the
/// frontend is `{"v":1}` — empty enough that the conversation UI
/// shouldn't render a bubble (see ConversationViewModel's
/// blank-skip).
#[cfg(target_arch = "wasm32")]
fn build_consult_assistant_envelope(utterance: &str, parsed: &parse::Parsed) -> String {
    let prompt = layer_c::compose_prompt(utterance, parsed);
    let mut out = String::from("{\"v\":1,\"consult_assistant\":{\"prompt\":");
    push_json_string(&mut out, &prompt);
    out.push_str(",\"continuation_context\":");
    push_json_string(&mut out, utterance);
    out.push_str("}}");
    out
}

// ── Layer C continuation handler ──────────────────────────────────

/// Called by the engine after the assistant round-trip. Branches:
///
/// - AI confidence `high` + valid JSON: commit per AI's values,
///   confirmation envelope.
/// - Everything else (AI low/partial, invalid JSON, empty response
///   signalling assistant unavailable): fall back to the skill's own
///   optimistic-commit-plus-warn-card flow, parsing the original
///   utterance locally. Same UX as pre-Layer-C Partial/Low handling.
#[cfg(target_arch = "wasm32")]
fn handle_continuation(cont: layer_c::Continuation) -> String {
    // Log the first chunk of the raw response so `adb logcat` shows
    // what the model actually returned — invaluable when tuning the
    // prompt or debugging a cloud assistant that's deviating from the
    // asked-for JSON shape. Cap at 200 chars so the log line stays
    // readable.
    let preview: String = cont.response.chars().take(200).collect();
    ari::log(
        ari::LogLevel::Info,
        &format!("continuation: assistant response preview: {preview:?}"),
    );

    let parsed = parse::parse(&cont.context);

    match layer_c::parse_assistant_response(&cont.response) {
        Some(resp) if resp.confidence.eq_ignore_ascii_case("high") => {
            ari::log(
                ari::LogLevel::Info,
                &format!(
                    "continuation: AI high-confidence commit title={:?} datetime={:?}",
                    resp.title, resp.datetime
                ),
            );
            commit_per_assistant(resp, parse::Confidence::High)
        }
        Some(resp) if resp.is_actionable_yes_no_clarification() => {
            // AI flagged partial AND gave us a yes/no question to
            // put in front of the user. Defer the commit — emit a
            // clarification card whose Yes button commits with the
            // AI's values and whose No button is a no-op.
            ari::log(
                ari::LogLevel::Info,
                &format!(
                    "continuation: AI partial + yes_no clarification — emitting card: {:?}",
                    resp.clarification.as_deref().unwrap_or("")
                ),
            );
            build_clarification_envelope(resp)
        }
        Some(resp) if resp.confidence.eq_ignore_ascii_case("partial") => {
            // AI was partial but didn't give us a usable yes/no
            // question (empty clarification, or follow_up=open_ended
            // which we don't render yet). Commit with the AI's
            // sharpened values + warn-and-commit card so the user
            // can Cancel. Same UX shell as Layer B, sharper content.
            ari::log(
                ari::LogLevel::Info,
                &format!(
                    "continuation: AI partial (no actionable clarification) — warn-and-commit with AI values title={:?} datetime={:?}",
                    resp.title, resp.datetime
                ),
            );
            commit_per_assistant(resp, parse::Confidence::Partial)
        }
        Some(resp) => {
            ari::log(
                ari::LogLevel::Warn,
                &format!(
                    "continuation: AI returned confidence={:?} — falling back to warn-and-commit with skill's first-pass parse",
                    resp.confidence
                ),
            );
            handle_create(&parsed)
        }
        None => {
            ari::log(
                ari::LogLevel::Warn,
                "continuation: assistant unavailable or response unparseable — falling back to warn-and-commit with skill's first-pass parse",
            );
            handle_create(&parsed)
        }
    }
}

/// Yes button on a clarification card fires this. The AI's
/// pre-staged values (destination, epoch_ms, title) are carried in the
/// utterance itself — no stored skill state is required, and the
/// commit happens right here without another assistant round-trip.
#[cfg(target_arch = "wasm32")]
fn handle_confirm(confirm: layer_c::Confirm) -> String {
    let resolved = if confirm.epoch_ms == 0 {
        Resolved::Untimed
    } else {
        // Titled reminders: we stored UTC epoch ms in the utterance,
        // so no further timezone conversion is needed at commit time.
        // `all_day = false` matches the way `commit_per_assistant`
        // packaged this earlier; if the AI intended an all-day date,
        // it sent the time as 00:00 and the frontend's calendar writer
        // infers all-day from that anyway.
        Resolved::At {
            ms: confirm.epoch_ms,
            all_day: false,
        }
    };

    let effective_destination = match &resolved {
        Resolved::Untimed => "tasks".to_string(),
        _ => confirm.destination.clone(),
    };

    let pseudo_parsed = parse::Parsed {
        title: confirm.title,
        when: parse::When::None,
        list_hint: None,
        speak_template: String::new(),
        confidence: parse::Confidence::High,
        unparsed: None,
    };

    let result = match effective_destination.as_str() {
        "tasks" => insert_into_tasks(&pseudo_parsed, &resolved),
        "calendar" => insert_into_calendar(&pseudo_parsed, &resolved),
        "both" => {
            let tasks_outcome = insert_into_tasks(&pseudo_parsed, &resolved);
            let calendar_outcome = insert_into_calendar(&pseudo_parsed, &resolved);
            match &calendar_outcome {
                Outcome::Success { .. } => calendar_outcome,
                _ => tasks_outcome,
            }
        }
        _ => insert_into_tasks(&pseudo_parsed, &resolved),
    };

    build_envelope(&pseudo_parsed, &resolved, &result)
}

/// Build a clarification card envelope with Yes (commits via
/// `ariconfirmreminder ...`) and No (no-op dismiss) actions. Speak
/// the AI's question so the user hears it — TTS is the primary
/// channel for the clarification, the card is the visible / tappable
/// backup.
#[cfg(target_arch = "wasm32")]
fn build_clarification_envelope(resp: layer_c::AssistantResponse) -> String {
    let clarification = resp.clarification.clone().unwrap_or_default();
    let title = resp.title.clone();
    let epoch_ms = datetime_to_epoch_ms(resp.datetime.as_deref());

    // Destination needs to match what the skill would have picked if
    // it had committed optimistically. Re-read the setting here so
    // the Yes path routes to the same list/calendar.
    let destination = ari::setting_get("destination")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "tasks".to_string());
    let effective_destination = if epoch_ms == 0 {
        "tasks".to_string()
    } else {
        destination
    };

    let confirm_utterance =
        layer_c::encode_confirm(&effective_destination, epoch_ms, &title);

    // Unique card id so multiple clarifications in one session don't
    // collide. Epoch ms changes per request, title differs, combined
    // they're effectively unique for a given user's input stream.
    let card_id = format!("reminder-clarify-{}", epoch_ms);

    let mut out = String::from("{\"v\":1,\"speak\":");
    push_json_string(&mut out, &clarification);
    out.push_str(",\"cards\":[{\"id\":");
    push_json_string(&mut out, &card_id);
    out.push_str(",\"title\":\"Is this right?\",\"body\":");
    push_json_string(&mut out, &clarification);
    out.push_str(",\"accent\":\"DEFAULT\",\"actions\":[");
    out.push_str("{\"id\":\"yes\",\"label\":\"Yes\",\"style\":\"PRIMARY\",\"utterance\":");
    push_json_string(&mut out, &confirm_utterance);
    out.push_str("},{\"id\":\"no\",\"label\":\"No\",\"style\":\"DEFAULT\"}");
    out.push_str("]}]}");
    out
}

/// Convert an optional ISO-8601 datetime string to UTC epoch ms, or 0
/// when the input is null/missing/unparseable. The commit path relies
/// on the sentinel 0 to route to the untimed (Tasks) path.
#[cfg(target_arch = "wasm32")]
fn datetime_to_epoch_ms(datetime: Option<&str>) -> i64 {
    let Some(dt_str) = datetime else { return 0 };
    let Some(p) = layer_c::parse_iso_datetime(dt_str) else {
        return 0;
    };
    let local_ms = civil_to_epoch_ms(p.year, p.month, p.day, p.hour, p.minute);
    let now = ari::local_now_components();
    let now_ms = ari::now_ms();
    let offset = tz_offset_ms(&now, now_ms);
    local_ms - offset
}

/// Commit the reminder per the assistant's structured response and
/// emit the confirmation envelope. `confidence_on_output` determines
/// whether the envelope includes a warn-and-commit card: pass
/// `High` when the AI was fully confident (no card, straight
/// confirmation), `Partial` to surface the Keep/Cancel card so the
/// user can roll back if the AI's disambiguation fell the wrong way.
#[cfg(target_arch = "wasm32")]
fn commit_per_assistant(
    resp: layer_c::AssistantResponse,
    confidence_on_output: parse::Confidence,
) -> String {
    let resolved = resolved_from_assistant_datetime(resp.datetime.as_deref());

    let destination = ari::setting_get("destination")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "tasks".to_string());
    let effective_destination = match &resolved {
        Resolved::Untimed => "tasks".to_string(),
        _ => destination,
    };

    let pseudo_parsed = parse::Parsed {
        title: resp.title,
        when: parse::When::None,
        list_hint: None,
        speak_template: String::new(),
        confidence: confidence_on_output,
        unparsed: None,
    };

    let result = match effective_destination.as_str() {
        "tasks" => insert_into_tasks(&pseudo_parsed, &resolved),
        "calendar" => insert_into_calendar(&pseudo_parsed, &resolved),
        "both" => {
            let tasks_outcome = insert_into_tasks(&pseudo_parsed, &resolved);
            let calendar_outcome = insert_into_calendar(&pseudo_parsed, &resolved);
            match &calendar_outcome {
                Outcome::Success { .. } => calendar_outcome,
                _ => tasks_outcome,
            }
        }
        _ => insert_into_tasks(&pseudo_parsed, &resolved),
    };

    build_envelope(&pseudo_parsed, &resolved, &result)
}

/// Convert the assistant's ISO-8601 datetime string to the skill's
/// internal [`Resolved`] representation. Null/missing/unparseable →
/// untimed (the safest degradation — a reminder without a time still
/// lands in the Tasks list).
#[cfg(target_arch = "wasm32")]
fn resolved_from_assistant_datetime(datetime: Option<&str>) -> Resolved {
    let Some(dt_str) = datetime else {
        return Resolved::Untimed;
    };
    let Some(p) = layer_c::parse_iso_datetime(dt_str) else {
        ari::log(
            ari::LogLevel::Warn,
            &format!(
                "continuation: couldn't parse AI datetime {:?}, treating as untimed",
                dt_str
            ),
        );
        return Resolved::Untimed;
    };
    let local_ms = civil_to_epoch_ms(p.year, p.month, p.day, p.hour, p.minute);
    let now = ari::local_now_components();
    let now_ms = ari::now_ms();
    let offset = tz_offset_ms(&now, now_ms);
    Resolved::At {
        ms: local_ms - offset,
        all_day: p.hour == 0 && p.minute == 0,
    }
}

// ── Normal create-reminder flow ───────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn handle_create(parsed: &parse::Parsed) -> String {
    let resolved = resolve_when(&parsed.when);

    let destination = ari::setting_get("destination")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "tasks".to_string());

    let effective_destination = match &resolved {
        Resolved::Untimed => "tasks".to_string(),
        _ => destination,
    };

    let result = match effective_destination.as_str() {
        "tasks" => insert_into_tasks(parsed, &resolved),
        "calendar" => insert_into_calendar(parsed, &resolved),
        "both" => {
            let tasks_outcome = insert_into_tasks(parsed, &resolved);
            let calendar_outcome = insert_into_calendar(parsed, &resolved);
            match &calendar_outcome {
                Outcome::Success { .. } => calendar_outcome,
                _ => tasks_outcome,
            }
        }
        _ => insert_into_tasks(parsed, &resolved),
    };

    build_envelope(parsed, &resolved, &result)
}

#[derive(Debug)]
enum Resolved {
    Untimed,
    At { ms: i64, all_day: bool },
}

#[cfg(target_arch = "wasm32")]
fn resolve_when(when: &parse::When) -> Resolved {
    match when {
        parse::When::None => Resolved::Untimed,
        parse::When::InSeconds(seconds) => {
            let now_ms = ari::now_ms();
            Resolved::At {
                ms: now_ms + (*seconds as i64 * 1000),
                all_day: false,
            }
        }
        parse::When::LocalClock {
            hour,
            minute,
            day_offset,
        } => {
            let now = ari::local_now_components();
            resolve_local_clock(&now, *hour as i64, *minute as i64, *day_offset as i64, false)
        }
        parse::When::LocalClockOnWeekday {
            hour,
            minute,
            weekday,
        } => {
            let now = ari::local_now_components();
            let days_ahead = days_until_weekday(now.weekday, *weekday);
            let hour = *hour as i64;
            let minute = *minute as i64;
            let offset = if days_ahead == 0
                && (hour < now.hour as i64
                    || (hour == now.hour as i64 && minute <= now.minute as i64))
            {
                7
            } else {
                days_ahead
            };
            resolve_local_clock(&now, hour, minute, offset, false)
        }
        parse::When::LocalClockOnDate {
            hour,
            minute,
            month,
            day,
        } => {
            let now = ari::local_now_components();
            let year = choose_year(&now, *month, *day, Some((*hour, *minute)));
            resolve_local_clock_on_date(&now, year, *month, *day, *hour, *minute, false)
        }
        parse::When::DateOnly { day_offset } => {
            let now = ari::local_now_components();
            resolve_local_clock(&now, 0, 0, *day_offset as i64, true)
        }
        parse::When::DateOnlyWeekday { weekday } => {
            let now = ari::local_now_components();
            let days_ahead = days_until_weekday(now.weekday, *weekday);
            resolve_local_clock(&now, 0, 0, days_ahead, true)
        }
        parse::When::DateOnlyDate { month, day } => {
            let now = ari::local_now_components();
            let year = choose_year(&now, *month, *day, None);
            resolve_local_clock_on_date(&now, year, *month, *day, 0, 0, true)
        }
    }
}

fn days_until_weekday(today: u8, target: u8) -> i64 {
    (target as i64 - today as i64 + 7) % 7
}

#[cfg(target_arch = "wasm32")]
fn choose_year(
    now: &ari::LocalTimeComponents,
    month: u8,
    day: u8,
    time: Option<(u8, u8)>,
) -> i32 {
    let today_m = now.month;
    let today_d = now.day;
    let is_future_this_year = if month > today_m {
        true
    } else if month < today_m {
        false
    } else if let Some((h, m)) = time {
        if day > today_d {
            true
        } else if day < today_d {
            false
        } else {
            (h as u32) > now.hour as u32
                || ((h as u32) == now.hour as u32 && (m as u32) > now.minute as u32)
        }
    } else {
        day >= today_d
    };
    if is_future_this_year {
        now.year
    } else {
        now.year + 1
    }
}

#[cfg(target_arch = "wasm32")]
fn resolve_local_clock_on_date(
    now: &ari::LocalTimeComponents,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    all_day: bool,
) -> Resolved {
    let target_local_ms = civil_to_epoch_ms(year, month, day, hour, minute);
    let now_ms = ari::now_ms();
    let offset_ms = tz_offset_ms(now, now_ms);
    Resolved::At {
        ms: target_local_ms - offset_ms,
        all_day,
    }
}

/// Timezone offset from UTC, in ms, at minute precision.
///
/// Computed from the local components vs. `now_ms`. Historically this
/// function subtracted a second-resolution `now_local_ms` from a full-
/// millisecond `now_ms`, which left a 0–999 ms slop in the result —
/// when the same offset was recomputed for display, a *different*
/// 0–999 ms slop tipped the formatted time into the previous minute
/// (insert at 14:00:00.750, display "1:59pm").
///
/// TZ offsets are always whole-minute quantities (no zone uses
/// sub-minute offsets), so truncating both sides to the minute
/// produces the exact offset with no drift. The round-trip
/// `target_ms + offset = target_local_ms` now holds exactly.
fn tz_offset_ms(now: &ari_skill_sdk::LocalTimeComponents, now_ms: i64) -> i64 {
    let now_local_minute_ms =
        civil_to_epoch_ms(now.year, now.month, now.day, now.hour, now.minute);
    let now_ms_minute_floor = (now_ms / 60_000) * 60_000;
    now_local_minute_ms - now_ms_minute_floor
}

#[cfg(target_arch = "wasm32")]
fn resolve_local_clock(
    now: &ari::LocalTimeComponents,
    hour: i64,
    minute: i64,
    day_offset: i64,
    all_day: bool,
) -> Resolved {
    let today_epoch_days = civil_to_days(now.year, now.month, now.day);
    let target_epoch_days = today_epoch_days + day_offset;
    let (y, m, d) = days_to_civil(target_epoch_days);
    resolve_local_clock_on_date(now, y, m, d, hour as u8, minute as u8, all_day)
}

// Civil-date helpers (proleptic Gregorian). Public-domain Howard
// Hinnant algorithms, duplicated here so the skill stays no_std
// friendly without pulling in chrono.

fn civil_to_days(year: i32, month: u8, day: u8) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let y = y as i64;
    let era = if y >= 0 { y / 400 } else { (y - 399) / 400 };
    let yoe = (y - era * 400) as u64;
    let m = month as u64;
    let d = day as u64;
    let doy = ((153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5) + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe as i64 - 719_468
}

fn days_to_civil(z: i64) -> (i32, u8, u8) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp.wrapping_sub(9) };
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m as u8, d as u8)
}

fn civil_to_epoch_ms(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> i64 {
    let days = civil_to_days(year, month, day);
    let secs = days * 86_400 + (hour as i64) * 3600 + (minute as i64) * 60;
    secs * 1000
}

// ── Insert path ────────────────────────────────────────────────────

#[derive(Debug)]
enum Outcome {
    Success {
        mode: Mode,
        destination_name: String,
        row_id: u64,
    },
    Failure {
        message: String,
    },
}

#[cfg(target_arch = "wasm32")]
fn insert_into_tasks(parsed: &parse::Parsed, resolved: &Resolved) -> Outcome {
    if !ari::tasks_provider_installed() {
        return Outcome::Failure {
            message: "I can't add tasks because no tasks app is installed.".to_string(),
        };
    }
    let lists = ari::tasks_list_lists();
    if lists.is_empty() {
        return Outcome::Failure {
            message: "Your tasks app doesn't have any lists set up yet.".to_string(),
        };
    }

    let target = resolve_list_target(
        &lists,
        |l: &ari::TaskList| format!("{}", l.id),
        |l: &ari::TaskList| l.display_name.as_str(),
        ari::setting_get("default_task_list"),
        parsed.list_hint.as_deref(),
    );
    let target_id = target.id;
    let target_name = target.display_name.clone();

    let (due_ms, all_day) = match resolved {
        Resolved::Untimed => (None, false),
        Resolved::At { ms, all_day } => (Some(*ms), *all_day),
    };
    let tz_id = ari::local_timezone_id();
    let row = ari::tasks_insert(&ari::InsertTaskParams {
        list_id: target_id,
        title: &parsed.title,
        due_ms,
        due_all_day: all_day,
        tz_id: if all_day { None } else { Some(tz_id.as_str()) },
    });
    match row {
        Some(id) => Outcome::Success {
            mode: Mode::Tasks,
            destination_name: target_name,
            row_id: id,
        },
        None => Outcome::Failure {
            message: "I couldn't save that task. Check the tasks app has permission."
                .to_string(),
        },
    }
}

#[cfg(target_arch = "wasm32")]
fn insert_into_calendar(parsed: &parse::Parsed, resolved: &Resolved) -> Outcome {
    let ms = match resolved {
        Resolved::At { ms, .. } => *ms,
        Resolved::Untimed => {
            return Outcome::Failure {
                message: "That reminder has no time, so I can't put it on the calendar."
                    .to_string(),
            };
        }
    };
    if !ari::calendar_has_write_permission() {
        return Outcome::Failure {
            message: "I need calendar write access to save that.".to_string(),
        };
    }
    let cals = ari::calendar_list_calendars();
    if cals.is_empty() {
        return Outcome::Failure {
            message: "I couldn't find any writable calendars.".to_string(),
        };
    }
    let target = resolve_list_target(
        &cals,
        |c: &ari::Calendar| format!("{}", c.id),
        |c: &ari::Calendar| c.display_name.as_str(),
        ari::setting_get("default_calendar"),
        parsed.list_hint.as_deref(),
    );
    let target_id = target.id;
    let target_name = target.display_name.clone();
    let tz_id = ari::local_timezone_id();
    let row = ari::calendar_insert(&ari::InsertCalendarEventParams {
        calendar_id: target_id,
        title: &parsed.title,
        start_ms: ms,
        duration_minutes: 30,
        reminder_minutes_before: 5,
        tz_id: tz_id.as_str(),
    });
    match row {
        Some(id) => Outcome::Success {
            mode: Mode::Calendar,
            destination_name: target_name,
            row_id: id,
        },
        None => Outcome::Failure {
            message: "I couldn't add that to the calendar.".to_string(),
        },
    }
}

fn resolve_list_target<'a, T>(
    available: &'a [T],
    by_id: impl Fn(&T) -> String,
    by_name: impl Fn(&T) -> &str,
    stored_default: Option<&str>,
    hint: Option<&str>,
) -> &'a T {
    if let Some(h) = hint {
        let needle = h.trim().to_lowercase();
        if let Some(t) = available
            .iter()
            .find(|t| by_name(t).to_lowercase() == needle)
        {
            return t;
        }
        if let Some(t) = available
            .iter()
            .find(|t| by_name(t).to_lowercase().contains(&needle))
        {
            return t;
        }
    }
    if let Some(def) = stored_default {
        if !def.is_empty() {
            if let Some(t) = available.iter().find(|t| by_id(t) == def) {
                return t;
            }
        }
    }
    &available[0]
}

// ── Envelope construction ─────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn build_envelope(parsed: &parse::Parsed, resolved: &Resolved, result: &Outcome) -> String {
    let mut out = String::from("{\"v\":1");

    match result {
        Outcome::Success {
            mode,
            destination_name,
            row_id,
        } => {
            let speak = format_success_speech(parsed, resolved, destination_name);
            out.push_str(",\"speak\":");
            push_json_string(&mut out, &speak);

            if parsed.confidence != parse::Confidence::High {
                out.push_str(",\"cards\":[");
                out.push_str(&build_partial_card(
                    parsed,
                    resolved,
                    mode,
                    *row_id,
                    destination_name,
                ));
                out.push(']');
            }
        }
        Outcome::Failure { message } => {
            out.push_str(",\"speak\":");
            push_json_string(&mut out, message);
        }
    }

    out.push_str(",\"confidence\":\"");
    out.push_str(parsed.confidence.as_envelope_str());
    out.push('"');
    if let Some(u) = &parsed.unparsed {
        out.push_str(",\"unparsed\":");
        push_json_string(&mut out, u);
    }
    out.push('}');
    out
}

#[cfg(target_arch = "wasm32")]
fn format_success_speech(
    parsed: &parse::Parsed,
    resolved: &Resolved,
    destination_name: &str,
) -> String {
    let when_phrase = match resolved {
        Resolved::Untimed => String::from("your list"),
        Resolved::At { ms, all_day } => format_when_phrase(*ms, *all_day),
    };
    if parsed.confidence == parse::Confidence::High {
        if matches!(resolved, Resolved::Untimed) {
            format!("Added {} to your {} list.", parsed.title, destination_name)
        } else {
            format!("Set {} for {}.", parsed.title, when_phrase)
        }
    } else {
        let preface = if matches!(resolved, Resolved::Untimed) {
            format!(
                "I've added {} to your {} list",
                parsed.title, destination_name
            )
        } else {
            format!("I've set this for {}", when_phrase)
        };
        let aside = match &parsed.unparsed {
            Some(u) => format!(
                ". I wasn't sure what \"{}\" meant — tap Cancel on the card if that was important.",
                u
            ),
            None => String::from(". Tap Cancel on the card if that's not what you meant."),
        };
        preface + &aside
    }
}

#[cfg(target_arch = "wasm32")]
fn format_when_phrase(ms: i64, _all_day: bool) -> String {
    let now = ari::local_now_components();
    let now_ms = ari::now_ms();
    let offset_ms = tz_offset_ms(&now, now_ms);
    let target_local_ms = ms + offset_ms;

    let total_secs = target_local_ms.div_euclid(1000);
    let days = total_secs.div_euclid(86_400);
    let secs_of_day = total_secs.rem_euclid(86_400);
    let hour = (secs_of_day / 3600) as u8;
    let minute = ((secs_of_day % 3600) / 60) as u8;
    let (_year, month, day) = days_to_civil(days);

    let today_days = civil_to_days(now.year, now.month, now.day);
    let day_label = if days == today_days {
        String::from("today")
    } else if days == today_days + 1 {
        String::from("tomorrow")
    } else {
        let month_name = match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "unknown",
        };
        format!("{} {}", day, month_name)
    };

    let (h12, ampm) = if hour == 0 {
        (12, "am")
    } else if hour < 12 {
        (hour, "am")
    } else if hour == 12 {
        (12, "pm")
    } else {
        (hour - 12, "pm")
    };
    if minute == 0 {
        format!("{} at {}{}", day_label, h12, ampm)
    } else {
        format!("{} at {}:{:02}{}", day_label, h12, minute, ampm)
    }
}

#[cfg(target_arch = "wasm32")]
fn build_partial_card(
    parsed: &parse::Parsed,
    resolved: &Resolved,
    mode: &Mode,
    row_id: u64,
    destination_name: &str,
) -> String {
    let when_phrase = match resolved {
        Resolved::Untimed => String::from("no specific time"),
        Resolved::At { ms, all_day } => format_when_phrase(*ms, *all_day),
    };
    let subtitle = format!("{} · {}", destination_name, when_phrase);
    let body = match &parsed.unparsed {
        Some(u) => format!(
            "I didn't understand \"{}\". Tap Cancel if that was important.",
            u
        ),
        None => String::from("Tap Cancel if that's not what you meant."),
    };
    let accent = match parsed.confidence {
        parse::Confidence::Low => "WARNING",
        _ => "DEFAULT",
    };
    let cancel_utterance = format!("aricancelreminder {} {}", mode.as_str(), row_id);

    let mut card = String::from("{\"id\":");
    let card_id = format!("reminder-partial-{}", row_id);
    push_json_string(&mut card, &card_id);
    card.push_str(",\"title\":");
    push_json_string(&mut card, parsed.title.as_str());
    card.push_str(",\"subtitle\":");
    push_json_string(&mut card, &subtitle);
    card.push_str(",\"body\":");
    push_json_string(&mut card, &body);
    card.push_str(",\"accent\":\"");
    card.push_str(accent);
    card.push_str("\",\"actions\":[");
    card.push_str("{\"id\":\"keep\",\"label\":\"Keep\",\"style\":\"DEFAULT\"},");
    card.push_str("{\"id\":\"cancel\",\"label\":\"Cancel\",\"style\":\"DESTRUCTIVE\"}");
    card.push_str("],\"on_cancel\":{\"v\":1,\"run_utterance\":");
    push_json_string(&mut card, &cancel_utterance);
    card.push_str("}}");
    card
}

// ── JSON string escape ────────────────────────────────────────────

fn push_json_string(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_cancel_parses() {
        let c = parse_internal_cancel("aricancelreminder tasks 42").unwrap();
        assert_eq!(c.id, 42);
        assert!(matches!(c.mode, Mode::Tasks));

        let c = parse_internal_cancel("aricancelreminder calendar 17").unwrap();
        assert_eq!(c.id, 17);
        assert!(matches!(c.mode, Mode::Calendar));
    }

    #[test]
    fn internal_cancel_rejects_malformed() {
        assert!(parse_internal_cancel("remind me at 5pm").is_none());
        assert!(parse_internal_cancel("aricancelreminder tasks").is_none());
        assert!(parse_internal_cancel("aricancelreminder garbage 42").is_none());
        assert!(parse_internal_cancel("aricancelreminder tasks notanumber").is_none());
    }

    #[test]
    fn internal_cancel_survives_engine_normalisation() {
        // Regression for the bug that caused Cancel to no-op: the
        // engine normalises underscores and colons into spaces
        // before the skill sees the input. Our prefix must be one
        // contiguous alphanumeric token so normalisation leaves it
        // alone. This test simulates that: lowercase + minimum
        // whitespace collapsed.
        let c = parse_internal_cancel("aricancelreminder tasks 42").unwrap();
        assert_eq!(c.id, 42);
        // Also tolerate leading / trailing whitespace and extra spaces
        // between tokens.
        let c = parse_internal_cancel("  aricancelreminder  tasks  42  ").unwrap();
        assert_eq!(c.id, 42);
    }

    #[test]
    fn days_to_civil_round_trip() {
        for days in [0i64, 1, 100, 10_000, 20_000, 25_000] {
            let (y, m, d) = days_to_civil(days);
            assert_eq!(civil_to_days(y, m, d), days);
        }
    }

    #[test]
    fn civil_to_epoch_ms_known_values() {
        // Unix epoch.
        assert_eq!(civil_to_epoch_ms(1970, 1, 1, 0, 0), 0);
        // 2026-04-22 00:00 UTC.
        // 56 years × 365 + 14 leap days (1972..=2024) = 20454 days from
        // 1970-04-22; 1970-04-22 is day 111 from the epoch; so
        // 2026-04-22 = day 20565. × 86_400_000 ms = 1_776_816_000_000.
        assert_eq!(civil_to_epoch_ms(2026, 4, 22, 0, 0), 1_776_816_000_000);
        // Time-of-day adds to the day-start.
        assert_eq!(
            civil_to_epoch_ms(2026, 4, 22, 3, 45),
            1_776_816_000_000 + 3 * 3_600_000 + 45 * 60_000
        );
    }

    #[test]
    fn days_until_weekday_logic() {
        assert_eq!(days_until_weekday(2, 2), 0);
        assert_eq!(days_until_weekday(0, 4), 4);
        assert_eq!(days_until_weekday(4, 0), 3);
        assert_eq!(days_until_weekday(6, 0), 1);
    }

    /// Shorthand for constructing a `LocalTimeComponents`. Tests only
    /// vary the time portion; year/month/day/weekday are arbitrary.
    fn lc(hour: u8, minute: u8, second: u8) -> ari_skill_sdk::LocalTimeComponents {
        ari_skill_sdk::LocalTimeComponents {
            year: 2026,
            month: 4,
            day: 23,
            hour,
            minute,
            second,
            weekday: 3, // Thursday
            tz_id: "Europe/London".to_string(),
        }
    }

    #[test]
    fn tz_offset_ms_exact_on_second_boundary() {
        // UTC+1, now_ms on an exact second boundary → offset is a
        // whole hour.
        let now = lc(11, 45, 33);
        let now_ms = civil_to_epoch_ms(2026, 4, 23, 10, 45) + 33 * 1000;
        assert_eq!(tz_offset_ms(&now, now_ms), 3_600_000);
    }

    #[test]
    fn tz_offset_ms_exact_with_sub_second_noise() {
        // UTC+1, now_ms has 750 ms of sub-second noise. Old buggy
        // code returned 3_599_250. Minute-truncated offset stays
        // 3_600_000 exactly.
        let now = lc(11, 45, 33);
        let now_ms = civil_to_epoch_ms(2026, 4, 23, 10, 45) + 33 * 1000 + 750;
        assert_eq!(tz_offset_ms(&now, now_ms), 3_600_000);
    }

    #[test]
    fn tz_offset_ms_works_for_utc() {
        let now = lc(10, 45, 33);
        let now_ms = civil_to_epoch_ms(2026, 4, 23, 10, 45) + 33 * 1000 + 250;
        assert_eq!(tz_offset_ms(&now, now_ms), 0);
    }

    #[test]
    fn tz_offset_ms_works_for_half_hour_zone() {
        // Simulate IST (UTC+5:30). Whole-minute offsets include half-
        // hour zones — the math must handle them without dropping the
        // 30-minute remainder.
        let now = lc(16, 15, 33);
        let now_ms = civil_to_epoch_ms(2026, 4, 23, 10, 45) + 33 * 1000 + 500;
        assert_eq!(tz_offset_ms(&now, now_ms), 5 * 3_600_000 + 30 * 60_000);
    }

    #[test]
    fn round_trip_target_local_ms_is_stable() {
        // The specific bug: set a reminder for 14:00 local, format
        // the resolved ms back to local components, confirm we get
        // 14:00 exactly. Old code returned 13:59.
        let now = lc(11, 45, 33);
        let now_ms = civil_to_epoch_ms(2026, 4, 23, 10, 45) + 33 * 1000 + 750;

        // Resolve: 14:00 local → stored UTC ms.
        let target_local_ms = civil_to_epoch_ms(2026, 4, 23, 14, 0);
        let offset = tz_offset_ms(&now, now_ms);
        let stored_utc_ms = target_local_ms - offset;

        // Format later (simulate a few ms passing + a different
        // sub-second fraction). Offset should still round-trip to the
        // same local instant.
        let now_later = lc(11, 45, 33);
        let now_ms_later = now_ms + 200; // 200 ms later
        let display_offset = tz_offset_ms(&now_later, now_ms_later);
        let displayed_local_ms = stored_utc_ms + display_offset;

        assert_eq!(displayed_local_ms, target_local_ms, "14:00 should still display as 14:00");
    }
}
