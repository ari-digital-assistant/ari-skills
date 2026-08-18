//! Weather-condition normaliser: maps Open-Meteo WMO codes and MET Norway
//! `symbol_code` strings into one internal [`Condition`], plus localised
//! label keys and bundled icon asset paths. Pure logic, `no_std`-friendly.
//!
//! The public items are consumed by the backend parser modules (added in
//! later cycles); until then they are only exercised by the unit tests.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    Clear, PartlyCloudy, Cloudy, Fog, Drizzle,
    LightRain, Rain, HeavyRain, Sleet,
    LightSnow, Snow, HeavySnow, Showers, Thunder, Unknown,
}

/// WMO weather interpretation codes (Open-Meteo). See
/// https://open-meteo.com/en/docs (WMO code table).
pub fn condition_from_wmo(code: u16) -> Condition {
    match code {
        0 => Condition::Clear,
        1 => Condition::Clear,         // mainly clear
        2 => Condition::PartlyCloudy,
        3 => Condition::Cloudy,
        45 | 48 => Condition::Fog,
        51 | 53 | 55 | 56 | 57 => Condition::Drizzle,
        61 => Condition::LightRain,
        63 => Condition::Rain,
        65 => Condition::HeavyRain,
        66 | 67 => Condition::Sleet,   // freezing rain — icy, not just wet
        71 => Condition::LightSnow,
        73 => Condition::Snow,
        75 => Condition::HeavySnow,
        77 => Condition::LightSnow,    // snow grains
        80 => Condition::Showers,
        81 => Condition::Showers,
        82 => Condition::HeavyRain,    // violent showers
        85 => Condition::Snow,         // snow showers
        86 => Condition::HeavySnow,
        95 => Condition::Thunder,
        96 | 99 => Condition::Thunder, // thunder w/ hail
        _ => Condition::Unknown,
    }
}

impl Condition {
    /// Strings-table key for the localised label.
    pub fn label_key(self) -> &'static str {
        match self {
            Condition::Clear => "cond.clear",
            Condition::PartlyCloudy => "cond.partly_cloudy",
            Condition::Cloudy => "cond.cloudy",
            Condition::Fog => "cond.fog",
            Condition::Drizzle => "cond.drizzle",
            Condition::LightRain => "cond.light_rain",
            Condition::Rain => "cond.rain",
            Condition::HeavyRain => "cond.heavy_rain",
            Condition::Sleet => "cond.sleet",
            Condition::LightSnow => "cond.light_snow",
            Condition::Snow => "cond.snow",
            Condition::HeavySnow => "cond.heavy_snow",
            Condition::Showers => "cond.showers",
            Condition::Thunder => "cond.thunder",
            Condition::Unknown => "cond.unknown",
        }
    }

    /// Bundled icon asset path (relative; the SDK's `Asset::new` prepends
    /// `asset:`). Conditions with a day/night distinction pick by `is_day`;
    /// others return a single icon. Icon files come from the MET Norway
    /// weathericons set (MIT) bundled under `assets/icons/`.
    pub fn icon(self, is_day: bool) -> &'static str {
        match self {
            Condition::Clear => if is_day { "icons/clearsky_day.webp" } else { "icons/clearsky_night.webp" },
            Condition::PartlyCloudy => if is_day { "icons/partlycloudy_day.webp" } else { "icons/partlycloudy_night.webp" },
            Condition::Showers => if is_day { "icons/rainshowers_day.webp" } else { "icons/rainshowers_night.webp" },
            Condition::Cloudy => "icons/cloudy.webp",
            Condition::Fog => "icons/fog.webp",
            Condition::Drizzle => "icons/lightrain.webp",
            Condition::LightRain => "icons/lightrain.webp",
            Condition::Rain => "icons/rain.webp",
            Condition::HeavyRain => "icons/heavyrain.webp",
            Condition::Sleet => "icons/sleet.webp",
            Condition::LightSnow => "icons/lightsnow.webp",
            Condition::Snow => "icons/snow.webp",
            Condition::HeavySnow => "icons/heavysnow.webp",
            Condition::Thunder => "icons/rainandthunder.webp",
            Condition::Unknown => "icons/cloudy.webp",
        }
    }

    /// Bundled full-bleed background image for this condition (opaque PNG under
    /// `assets/heroes/`). Day/night variants for clear & partly-cloudy; the wet
    /// conditions collapse to `rain`, the frozen ones to `snow`.
    pub fn hero(self, is_day: bool) -> &'static str {
        match self {
            Condition::Clear => if is_day { "heroes/clear_day.webp" } else { "heroes/clear_night.webp" },
            Condition::PartlyCloudy => if is_day { "heroes/partly_cloudy_day.webp" } else { "heroes/partly_cloudy_night.webp" },
            Condition::Cloudy => "heroes/cloudy.webp",
            Condition::Fog => "heroes/fog.webp",
            Condition::Drizzle | Condition::LightRain | Condition::Rain
                | Condition::HeavyRain | Condition::Showers => "heroes/rain.webp",
            Condition::Sleet | Condition::LightSnow | Condition::Snow | Condition::HeavySnow => "heroes/snow.webp",
            Condition::Thunder => "heroes/thunder.webp",
            Condition::Unknown => "heroes/cloudy.webp",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wmo_codes_map_to_conditions() {
        assert_eq!(condition_from_wmo(0), Condition::Clear);
        assert_eq!(condition_from_wmo(2), Condition::PartlyCloudy);
        assert_eq!(condition_from_wmo(3), Condition::Cloudy);
        assert_eq!(condition_from_wmo(45), Condition::Fog);
        assert_eq!(condition_from_wmo(63), Condition::Rain);
        assert_eq!(condition_from_wmo(65), Condition::HeavyRain);
        assert_eq!(condition_from_wmo(75), Condition::HeavySnow);
        assert_eq!(condition_from_wmo(81), Condition::Showers);
        assert_eq!(condition_from_wmo(95), Condition::Thunder);
        assert_eq!(condition_from_wmo(66), Condition::Sleet);   // freezing rain
        assert_eq!(condition_from_wmo(999), Condition::Unknown);
    }
    #[test]
    fn condition_label_keys() {
        assert_eq!(Condition::Clear.label_key(), "cond.clear");
        assert_eq!(Condition::HeavyRain.label_key(), "cond.heavy_rain");
        assert_eq!(Condition::Unknown.label_key(), "cond.unknown");
    }
    #[test]
    fn condition_icon_picks_day_night() {
        assert_eq!(Condition::Clear.icon(true), "icons/clearsky_day.webp");
        assert_eq!(Condition::Clear.icon(false), "icons/clearsky_night.webp");
        assert_eq!(Condition::Cloudy.icon(true), "icons/cloudy.webp"); // no day/night variant
        assert_eq!(Condition::Cloudy.icon(false), "icons/cloudy.webp");
    }
    #[test]
    fn condition_backgrounds() {
        assert_eq!(Condition::Clear.hero(true), "heroes/clear_day.webp");
        assert_eq!(Condition::Clear.hero(false), "heroes/clear_night.webp");
        assert_eq!(Condition::PartlyCloudy.hero(false), "heroes/partly_cloudy_night.webp");
        assert_eq!(Condition::Rain.hero(true), "heroes/rain.webp");
        assert_eq!(Condition::Showers.hero(true), "heroes/rain.webp");
        assert_eq!(Condition::Snow.hero(true), "heroes/snow.webp");
        assert_eq!(Condition::Sleet.hero(true), "heroes/snow.webp");
        assert_eq!(Condition::Thunder.hero(true), "heroes/thunder.webp");
        assert_eq!(Condition::Fog.hero(true), "heroes/fog.webp");
        assert_eq!(Condition::Unknown.hero(true), "heroes/cloudy.webp");
    }
}
