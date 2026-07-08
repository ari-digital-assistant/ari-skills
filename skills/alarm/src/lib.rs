#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};

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

pub fn handle(input: &str) -> String {
    match classify(input) {
        Intent::Set { hour, minute, message, days } => {
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
    use super::handle;

    fn v(input: &str) -> serde_json::Value {
        serde_json::from_str(&handle(input)).unwrap()
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
}
