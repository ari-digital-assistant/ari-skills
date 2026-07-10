#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};

#[cfg(target_arch = "wasm32")]
use ari_skill_sdk as ari;
use ari_skill_sdk::presentation as p;

mod parse;
use parse::{classify, Intent};

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

pub fn handle(input: &str, mode: &str) -> String {
    match classify(input) {
        Intent::Navigate { destination } => {
            let speak = t("navigate_confirm", &[("destination", &destination)])
                .unwrap_or_else(|| format!("Taking you to {destination}."));
            let nav = p::Navigate::to(destination).mode(mode);
            p::Envelope::new().speak(speak).navigate(nav).to_json()
        }
        Intent::NeedDestination => {
            let speak = t("need_destination", &[])
                .unwrap_or_else(|| "Where would you like to go?".to_string());
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
    // Read the user's navigation-style preference; default to the neutral
    // default-maps-app behaviour when unset.
    let mode = ari::setting_get("navigation_mode").unwrap_or("default_app");
    ari::respond_action(&handle(input, mode))
}

#[cfg(test)]
mod tests {
    use super::handle;

    fn v(input: &str, mode: &str) -> serde_json::Value {
        serde_json::from_str(&handle(input, mode)).unwrap()
    }

    #[test]
    fn navigate_emits_slot_and_mode() {
        let j = v("take me to mcdonalds", "default_app");
        assert_eq!(j["navigate"]["destination"], "mcdonalds");
        assert_eq!(j["navigate"]["mode"], "default_app");
        assert!(j["speak"].as_str().unwrap().to_lowercase().contains("mcdonalds"));
    }

    #[test]
    fn turn_by_turn_mode_passes_through() {
        let j = v("navigate to asda", "turn_by_turn");
        assert_eq!(j["navigate"]["destination"], "asda");
        assert_eq!(j["navigate"]["mode"], "turn_by_turn");
    }

    #[test]
    fn home_phrase_navigates_home() {
        let j = v("take me home", "default_app");
        assert_eq!(j["navigate"]["destination"], "home");
    }

    #[test]
    fn no_destination_asks_without_navigate_slot() {
        let j = v("take me to", "default_app");
        assert!(j.get("navigate").is_none());
        assert!(j["speak"].as_str().unwrap().to_lowercase().contains("where"));
    }
}
