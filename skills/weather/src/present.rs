#![allow(dead_code)] // consumed by lib.rs wire-up (later task)

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use ari_skill_sdk::presentation as p;
use crate::forecast::{Forecast, Source};
use crate::router::{When, Facet};
use crate::units::{System, c_to_f, ms_to_kmh, ms_to_mph};
use crate::facets::{wind_band, rain_band, uv_band};

/// Host-call seam: the wasm impl wraps `ari::t` / `ari::format_number` and a
/// weekday formatter; tests inject a fake.
pub trait L10n {
    fn t(&self, key: &str, args: &[(&str, &str)]) -> String;
    fn num(&self, v: f64) -> String;
    /// Localised short weekday for an ISO "YYYY-MM-DD" date.
    fn day_label(&self, iso_date: &str) -> String;
}

fn attribution(src: Source) -> &'static str {
    match src {
        Source::MetNorway => "Weather data from MET Norway",
        Source::OpenMeteo => "Weather data by Open-Meteo.com",
    }
}

fn temp(sys: System, c: f64, l: &dyn L10n) -> String {
    let v = if sys == System::Imperial { c_to_f(c) } else { c };
    l.num(v)
}
fn wind(sys: System, ms: f64, l: &dyn L10n) -> String {
    // Append the speed unit — unlike temperature ("14 degrees"), a bare wind
    // number is meaningless, and the unit varies by system so the strings
    // templates can't carry it.
    let (v, unit) = if sys == System::Imperial { (ms_to_mph(ms), "mph") } else { (ms_to_kmh(ms), "km/h") };
    let mut s = l.num(v);
    s.push(' ');
    s.push_str(unit);
    s
}
fn when_key(w: When) -> &'static str {
    match w {
        When::Now => "when.now",
        When::Today => "when.today",
        When::Tomorrow => "when.tomorrow",
        When::ThisWeek => "when.this_week",
    }
}
fn target_index(w: When) -> usize { match w { When::Tomorrow => 1, _ => 0 } }

/// The forecast day a request targets (Tomorrow→[1], else→[0]), clamped to
/// the available range so the index can never go out of bounds. Caller must
/// ensure `f.daily` is non-empty (every call site is guarded by that check).
fn day_at(f: &Forecast, when: When) -> &crate::forecast::DailyConditions {
    &f.daily[target_index(when).min(f.daily.len() - 1)]
}

/// Compact current-conditions card (also used as the facet card).
fn current_card(f: &Forecast, place: &Option<String>, sys: System, l: &dyn L10n) -> p::Card {
    let cond_label = l.t(f.current.condition.label_key(), &[]);
    let title = place.clone().unwrap_or_else(|| l.t("card.current_location", &[]));
    let mut lines: Vec<String> = Vec::new();
    lines.push(l.t("card.feels_like", &[("temp", &temp(sys, f.current.feels_like_c, l))]));
    lines.push(l.t("card.wind", &[("speed", &wind(sys, f.current.wind_speed_ms, l))]));
    if let Some(h) = f.current.humidity_pct {
        lines.push(l.t("card.humidity", &[("pct", &l.num(h))]));
    }
    lines.push(attribution(f.source).to_string());
    p::Card::new("weather_current")
        .title(title)
        .subtitle(cond_label)
        .body(lines.join("\n"))
        .icon(p::Asset::new(f.current.condition.icon(f.current.is_day)))
}

/// Multi-day forecast card: one row per day + attribution.
fn forecast_card(f: &Forecast, place: &Option<String>, sys: System, l: &dyn L10n) -> p::Card {
    let title = place.clone().unwrap_or_else(|| l.t("card.current_location", &[]));
    let mut lines: Vec<String> = Vec::new();
    for day in f.daily.iter().take(7) {
        let cond = l.t(day.condition.label_key(), &[]);
        lines.push(l.t("card.daily_row", &[
            ("day", &l.day_label(&day.date)),
            ("hi", &temp(sys, day.temp_max_c, l)),
            ("lo", &temp(sys, day.temp_min_c, l)),
            ("cond", &cond),
        ]));
    }
    lines.push(attribution(f.source).to_string());
    let icon_cond = f.daily.first().map(|d| d.condition).unwrap_or(crate::conditions::Condition::Unknown);
    p::Card::new("weather_forecast")
        .title(title)
        .body(lines.join("\n"))
        .icon(p::Asset::new(icon_cond.icon(true)))
}

fn facet_speak(f: &Forecast, when: When, facet: Facet, sys: System, l: &dyn L10n) -> String {
    match facet {
        Facet::Wind => {
            let band = l.t(wind_band(f.current.wind_speed_ms), &[]);
            l.t("speak.wind", &[("band", &band), ("speed", &wind(sys, f.current.wind_speed_ms, l))])
        }
        Facet::Uv => {
            let uv = if when != When::Now && !f.daily.is_empty() {
                day_at(f, when).uv_index_max
            } else { f.current.uv_index };
            match uv {
                Some(u) => l.t("speak.uv", &[("band", &l.t(uv_band(u), &[])), ("value", &l.num(u))]),
                None => l.t("speak.uv_unknown", &[]),
            }
        }
        Facet::Rain => {
            let (prob, mm) = if when != When::Now && !f.daily.is_empty() {
                let d = day_at(f, when);
                (d.precip_probability, d.precip_mm)
            } else { (f.current.precip_probability, f.current.precip_mm) };
            let band = l.t(rain_band(prob), &[]);
            l.t("speak.rain", &[("band", &band), ("mm", &l.num(mm))])
        }
        Facet::None => String::new(),
    }
}

/// Build the v:1 envelope JSON for a resolved request.
pub fn build(f: &Forecast, when: When, facet: Facet, sys: System, _locale: &str, l: &dyn L10n) -> String {
    let place = f.place_label.clone();

    // Facet answer takes precedence — spoken band + a current-conditions card.
    if facet != Facet::None {
        let speak = facet_speak(f, when, facet, sys, l);
        return p::Envelope::new().speak(speak).card(current_card(f, &place, sys, l)).to_json();
    }

    // Multi-day forecast.
    if when != When::Now && !f.daily.is_empty() {
        let day = day_at(f, when);
        let cond = l.t(day.condition.label_key(), &[]);
        let wk = l.t(when_key(when), &[]);
        let speak = match &place {
            Some(pl) => l.t("speak.forecast_place", &[("when", &wk), ("place", pl),
                ("hi", &temp(sys, day.temp_max_c, l)), ("lo", &temp(sys, day.temp_min_c, l)), ("cond", &cond)]),
            None => l.t("speak.forecast_no_place", &[("when", &wk),
                ("hi", &temp(sys, day.temp_max_c, l)), ("lo", &temp(sys, day.temp_min_c, l)), ("cond", &cond)]),
        };
        return p::Envelope::new().speak(speak).card(forecast_card(f, &place, sys, l)).to_json();
    }

    // Current conditions (default).
    let cond_label = l.t(f.current.condition.label_key(), &[]);
    let temp_s = temp(sys, f.current.temp_c, l);
    let feels_s = temp(sys, f.current.feels_like_c, l);
    let speak = match &place {
        Some(pl) => l.t("speak.current_place",
            &[("place", pl), ("temp", &temp_s), ("cond", &cond_label), ("feels", &feels_s)]),
        None => l.t("speak.current_no_place",
            &[("temp", &temp_s), ("cond", &cond_label), ("feels", &feels_s)]),
    };
    p::Envelope::new().speak(speak).card(current_card(f, &place, sys, l)).to_json()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forecast::*;
    use crate::conditions::Condition;
    use crate::units::System;
    use crate::router::{When, Facet};
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;

    // Fake i18n: echo "key" plus its arg values (so nested band/row keys
    // passed as args surface in the rendered string), round numbers, echo
    // the date for day labels.
    struct Fakes;
    impl L10n for Fakes {
        fn t(&self, key: &str, args: &[(&str, &str)]) -> String {
            let mut s = key.to_string();
            for (_k, v) in args {
                s.push(' ');
                s.push_str(v);
            }
            s
        }
        fn num(&self, v: f64) -> String { alloc::format!("{}", v.round() as i64) }
        fn day_label(&self, iso_date: &str) -> String { iso_date.to_string() }
    }

    fn current_only() -> Forecast {
        Forecast { place_label: Some("Valletta".to_string()), source: Source::OpenMeteo,
            current: Conditions { temp_c: 14.0, feels_like_c: 13.0, condition: Condition::PartlyCloudy,
                is_day: true, wind_speed_ms: 8.0, wind_gust_ms: Some(12.0), precip_mm: 0.0,
                precip_probability: Some(20.0), humidity_pct: Some(60.0), uv_index: Some(5.0) },
            daily: Vec::new() }
    }

    fn with_daily() -> Forecast {
        let mut f = current_only();
        f.daily = alloc::vec![
            DailyConditions { date: "2026-06-17".to_string(), temp_min_c: 20.0, temp_max_c: 30.0,
                condition: Condition::Clear, precip_mm: 0.0, precip_probability: Some(5.0), uv_index_max: Some(7.0) },
            DailyConditions { date: "2026-06-18".to_string(), temp_min_c: 21.0, temp_max_c: 31.0,
                condition: Condition::Rain, precip_mm: 4.0, precip_probability: Some(80.0), uv_index_max: Some(6.0) },
        ];
        f
    }

    #[test]
    fn current_envelope_has_speak_card_and_attribution() {
        let env = build(&current_only(), When::Now, Facet::None, System::Metric, "en", &Fakes);
        assert!(env.contains("\"speak\""));
        assert!(env.contains("Valletta"));
        assert!(env.contains("Open-Meteo"));            // attribution footer
        assert!(env.contains("asset:icons/"));          // icon attached
    }

    #[test]
    fn uv_facet_speaks_band() {
        let env = build(&current_only(), When::Now, Facet::Uv, System::Metric, "en", &Fakes);
        assert!(env.contains("uv.moderate"));           // uv 5 → moderate band key surfaced
    }

    #[test]
    fn forecast_envelope_lists_days() {
        let env = build(&with_daily(), When::ThisWeek, Facet::None, System::Metric, "en", &Fakes);
        assert!(env.contains("2026-06-17"));            // day label (faked = the date) present
        assert!(env.contains("2026-06-18"));
        assert!(env.contains("card.daily_row"));        // daily row template key used
    }

    #[test]
    fn uv_facet_on_forecast_uses_daily_max() {
        // when=Tomorrow → daily[1].uv_index_max = 6.0 → "high" band (6..8)
        let env = build(&with_daily(), When::Tomorrow, Facet::Uv, System::Metric, "en", &Fakes);
        assert!(env.contains("uv.high"));
    }
}
