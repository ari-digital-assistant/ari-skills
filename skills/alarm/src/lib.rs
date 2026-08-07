#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};

#[cfg(target_arch = "wasm32")]
use ari_skill_sdk as ari;
use ari_skill_sdk::presentation as p;

mod parse;
use parse::{classify, Day, Intent};

#[cfg(target_arch = "wasm32")]
#[inline]
fn t(key: &str, args: &[(&str, &str)]) -> Option<String> {
    ari::t(key, args).map(|s| s.to_string())
}
#[cfg(not(target_arch = "wasm32"))]
#[inline]
fn t(_key: &str, _args: &[(&str, &str)]) -> Option<String> {
    None
}

fn sdk_day(d: Day) -> p::Day {
    match d {
        Day::Mon => p::Day::Mon, Day::Tue => p::Day::Tue, Day::Wed => p::Day::Wed,
        Day::Thu => p::Day::Thu, Day::Fri => p::Day::Fri, Day::Sat => p::Day::Sat,
        Day::Sun => p::Day::Sun,
    }
}

/// Format "7:00" style, honouring the parsed 24h clock. Kept simple for v1.
fn hhmm(hour: u8, minute: u8) -> String {
    format!("{hour}:{minute:02}")
}

/// Current local hour and minute, or `None` off-wasm where the host clock
/// import doesn't exist.
#[cfg(target_arch = "wasm32")]
fn local_hour_minute() -> Option<(u8, u8)> {
    let now = ari::local_now_components();
    Some((now.hour, now.minute))
}
#[cfg(not(target_arch = "wasm32"))]
fn local_hour_minute() -> Option<(u8, u8)> {
    None
}

/// Resolve a bare 12h time to whichever of its two readings comes round
/// first. Asked for "half past five" at 08:30, a person means 17:30 today,
/// not 05:30 tomorrow.
fn next_occurrence(hour: u8, minute: u8, now_hour: u8, now_minute: u8) -> u8 {
    let now = now_hour as u16 * 60 + now_minute as u16;
    let morning = (hour % 12) as u16 * 60 + minute as u16;
    let evening = morning + 12 * 60;
    let delay = |candidate: u16| match (candidate + 24 * 60 - now) % (24 * 60) {
        0 => 24 * 60,
        d => d,
    };
    (if delay(morning) < delay(evening) { morning } else { evening } / 60) as u8
}

pub fn handle(input: &str) -> String {
    handle_at(input, local_hour_minute())
}

fn handle_at(input: &str, now: Option<(u8, u8)>) -> String {
    match classify(input) {
        Intent::Set { mut hour, minute, meridian_known, message, days } => {
            // Recurrence pins the meaning on its own: a bare "half past five
            // every weekday" is the morning wake-up, whatever time it is now.
            if !meridian_known && days.is_empty() {
                if let Some((now_hour, now_minute)) = now {
                    hour = next_occurrence(hour, minute, now_hour, now_minute);
                }
            }
            let mut alarm = p::Alarm::set(hour, minute);
            if let Some(ref m) = message {
                alarm = alarm.message(m.clone());
            }
            if !days.is_empty() {
                let sdk_days: alloc::vec::Vec<p::Day> =
                    days.iter().copied().map(sdk_day).collect();
                alarm = alarm.days(&sdk_days);
            }

            let when = hhmm(hour, minute);
            let speak = t("set_confirm", &[("time", &when)])
                .unwrap_or_else(|| format!("Alarm set for {when}."));
            let card_title =
                t("card_title", &[]).unwrap_or_else(|| "Alarm set".to_string());
            let card = p::Card::new("alarm-confirm")
                .title(card_title)
                .subtitle(when);

            p::Envelope::new().speak(speak).alarm(alarm).card(card).to_json()
        }
        Intent::Show => {
            let speak = t("show_hint", &[]).unwrap_or_else(|| {
                "I can't change alarms directly, but here's your clock app.".to_string()
            });
            p::Envelope::new().speak(speak).alarm(p::Alarm::show()).to_json()
        }
        Intent::NeedTime => {
            let speak = t("need_time", &[])
                .unwrap_or_else(|| "What time should I set the alarm for?".to_string());
            p::Envelope::new().speak(speak).to_json()
        }
        Intent::Unintelligible => {
            let speak = t("unintelligible", &[])
                .unwrap_or_else(|| "Sorry, I didn't catch a time for that alarm.".to_string());
            p::Envelope::new().speak(speak).to_json()
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    0.9
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let input = unsafe { ari::input(ptr, len) };
    ari::respond_action(&handle(input))
}

#[cfg(test)]
mod tests {
    use super::{handle, handle_at, next_occurrence};

    fn v(input: &str) -> serde_json::Value {
        serde_json::from_str(&handle(input)).unwrap()
    }

    fn at(input: &str, now: (u8, u8)) -> serde_json::Value {
        serde_json::from_str(&handle_at(input, Some(now))).unwrap()
    }

    #[test]
    fn set_emits_alarm_and_card() {
        let j = v("set an alarm for 7 am");
        assert_eq!(j["alarm"]["op"], "set");
        assert_eq!(j["alarm"]["hour"], 7);
        assert_eq!(j["alarm"]["minute"], 0);
        assert_eq!(j["alarm"]["skip_ui"], true);
        assert_eq!(j["cards"][0]["title"], "Alarm set");
        assert!(j["speak"].as_str().unwrap().contains("7"));
    }

    #[test]
    fn recurring_set_carries_days() {
        let j = v("set an alarm for 6 30 every weekday");
        assert_eq!(j["alarm"]["days"][0], "mon");
        assert_eq!(j["alarm"]["days"][4], "fri");
    }

    #[test]
    fn show_emits_show_op() {
        let j = v("what alarms do i have");
        assert_eq!(j["alarm"]["op"], "show");
        assert!(j["cards"].get(0).is_none());
    }

    #[test]
    fn need_time_asks_without_alarm_slot() {
        let j = v("set an alarm");
        assert!(j.get("alarm").is_none());
        assert!(j["speak"].as_str().unwrap().to_lowercase().contains("time"));
    }

    #[test]
    fn next_occurrence_picks_the_sooner_reading() {
        // At 08:30, 17:30 is 9 hours off and 05:30 is 21 — afternoon wins.
        assert_eq!(next_occurrence(5, 30, 8, 30), 17);
        // At 02:00 the morning reading is only 3.5 hours off.
        assert_eq!(next_occurrence(5, 30, 2, 0), 5);
        // Past both readings today: wraps to tomorrow morning.
        assert_eq!(next_occurrence(5, 30, 20, 0), 5);
        // Exactly now → the reading 12 hours out, never "in a moment".
        assert_eq!(next_occurrence(5, 30, 5, 30), 17);
        // 12 means noon or midnight, same rule.
        assert_eq!(next_occurrence(12, 0, 8, 0), 12);
        assert_eq!(next_occurrence(12, 0, 13, 0), 0);
    }

    #[test]
    fn ambiguous_time_resolves_against_the_clock() {
        // The reported bug: "half past five" at 08:30 is 17:30, not 05:30.
        let j = at("set an alarm for half past 5", (8, 30));
        assert_eq!(j["alarm"]["hour"], 17);
        assert_eq!(j["alarm"]["minute"], 30);
        assert!(j["speak"].as_str().unwrap().contains("17:30"));
    }

    #[test]
    fn stated_meridian_is_left_alone() {
        let j = at("set an alarm for 5 30 am", (8, 30));
        assert_eq!(j["alarm"]["hour"], 5);
        assert_eq!(j["alarm"]["minute"], 30);
    }

    #[test]
    fn recurring_alarm_keeps_the_spoken_hour() {
        let j = at("set an alarm for 6 30 every weekday", (8, 30));
        assert_eq!(j["alarm"]["hour"], 6);
        assert_eq!(j["alarm"]["days"][0], "mon");
    }

    #[test]
    fn twenty_four_hour_time_is_left_alone() {
        let j = at("set an alarm for 17 30", (20, 0));
        assert_eq!(j["alarm"]["hour"], 17);
    }
}
