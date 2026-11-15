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

/// `home_address` is the user's own setting, empty when unset.
pub fn handle(input: &str, mode: &str, home_address: &str) -> String {
    match classify(input) {
        Intent::Navigate { destination } => {
            let speak = t("navigate_confirm", &[("destination", &destination)])
                .unwrap_or_else(|| format!("Taking you to {destination}."));
            let nav = p::Navigate::to(destination).mode(mode);
            p::Envelope::new().speak(speak).navigate(nav).to_json()
        }
        Intent::Home { said } => {
            let speak = t("navigate_home", &[]).unwrap_or_else(|| "Taking you home.".to_string());
            // An address the user typed works with any maps app. Without one,
            // name the saved home for a maps app that keeps one (Google Maps,
            // turn-by-turn), and search for the word they said everywhere else.
            let nav = match home_address.trim() {
                "" => p::Navigate::to(said).mode(mode).saved_home(),
                address => p::Navigate::to(address).mode(mode),
            };
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
    let home_address = ari::setting_get("home_address").unwrap_or("");
    ari::respond_action(&handle(input, mode, home_address))
}

#[cfg(test)]
mod tests {
    use super::handle;

    fn v(input: &str, mode: &str) -> serde_json::Value {
        serde_json::from_str(&handle(input, mode, "")).unwrap()
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
        assert_eq!(j["navigate"]["place"], "home");
        assert_eq!(j["speak"], "Taking you home.");
    }

    #[test]
    fn italian_home_keeps_the_word_said() {
        let j = v("portami a casa", "turn_by_turn");
        assert_eq!(j["navigate"]["destination"], "casa");
        assert_eq!(j["navigate"]["place"], "home");
        assert_eq!(j["navigate"]["mode"], "turn_by_turn");
    }

    #[test]
    fn a_home_address_setting_wins_over_the_saved_place() {
        let j: serde_json::Value =
            serde_json::from_str(&handle("take me home", "turn_by_turn", " 10 Downing Street, London ")).unwrap();
        assert_eq!(j["navigate"]["destination"], "10 Downing Street, London");
        assert!(j["navigate"].get("place").is_none());
    }

    #[test]
    fn other_destinations_carry_no_place() {
        let j = v("take me to mcdonalds", "turn_by_turn");
        assert!(j["navigate"].get("place").is_none());
    }

    #[test]
    fn no_destination_asks_without_navigate_slot() {
        let j = v("take me to", "default_app");
        assert!(j.get("navigate").is_none());
        assert!(j["speak"].as_str().unwrap().to_lowercase().contains("where"));
    }
}
