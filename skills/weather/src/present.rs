use alloc::string::String;
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

/// Current-conditions stat card: current temp leads, condition as caption,
/// feels-like pill, wind/humidity metrics, condition background, attribution.
fn current_card(f: &Forecast, place: &Option<String>, sys: System, l: &dyn L10n) -> p::Card {
    let cond_label = l.t(f.current.condition.label_key(), &[]);
    let title = place.clone().unwrap_or_else(|| l.t("card.current_location", &[]));
    let headline = alloc::format!("{}°", temp(sys, f.current.temp_c, l));

    let mut stat = p::Stat::new(headline)
        .caption(cond_label)
        .pill(p::IconText::new(l.t("card.feels_like", &[("temp", &temp(sys, f.current.feels_like_c, l))]))
            .icon(p::Asset::new("ui/thermometer.webp")))
        .metric(p::IconText::new(l.t("card.metric_wind", &[("speed", &wind(sys, f.current.wind_speed_ms, l))]))
            .icon(p::Asset::new("ui/wind.webp")))
        .background(p::Asset::new(f.current.condition.hero(f.current.is_day)))
        .footer(p::IconText::new(attribution(f.source)).icon(p::Asset::new("ui/shield.webp")));
    if let Some(h) = f.current.humidity_pct {
        stat = stat.metric(p::IconText::new(l.t("card.metric_humidity", &[("pct", &l.num(h))]))
            .icon(p::Asset::new("ui/droplet.webp")));
    }
    p::Card::new("weather_current").title(title).icon(p::Asset::new("ui/pin.webp")).stat(stat)
}

/// Multi-day list card: summary chip (week hi/lo + dominant condition) + one
/// row per day (weekday, icon, condition, hi/lo, rain-chance badge).
fn forecast_card(f: &Forecast, place: &Option<String>, sys: System, l: &dyn L10n) -> p::Card {
    let title = place.clone().unwrap_or_else(|| l.t("card.current_location", &[]));
    let subtitle = l.t("card.forecast_subtitle", &[]);
    let (max_hi, min_lo) = f.week_extremes();
    let dom = f.dominant_daily_condition();
    let summary = p::IconText::new(l.t("card.forecast_summary", &[
        ("hi", &temp(sys, max_hi, l)), ("lo", &temp(sys, min_lo, l)),
        ("cond", &l.t(dom.label_key(), &[])),
    ])).icon(p::Asset::new(dom.icon(true)));

    let mut list = p::ListCard::new().summary(summary);
    for day in f.daily.iter().take(7) {
        let mut row = p::ListRow::new(l.day_label(&day.date))
            .icon(p::Asset::new(day.condition.icon(true)))
            .text(l.t(day.condition.label_key(), &[]))
            .trailing(l.t("card.row_temps", &[
                ("hi", &temp(sys, day.temp_max_c, l)), ("lo", &temp(sys, day.temp_min_c, l))]));
        if let Some(prob) = day.precip_probability {
            if prob >= 20.0 {
                row = row.badge(p::IconText::new(l.t("card.row_badge", &[("pct", &l.num(prob))]))
                    .icon(p::Asset::new("ui/droplet.webp")));
            }
        }
        list = list.row(row);
    }
    list = list.footer(p::IconText::new(attribution(f.source)).icon(p::Asset::new("ui/shield.webp")));
    p::Card::new("weather_forecast").title(title).subtitle(subtitle).list(list)
}

fn facet_speak(f: &Forecast, when: When, facet: Facet, sys: System, l: &dyn L10n) -> String {
    match facet {
        Facet::Wind => {
            let band = l.t(wind_band(f.current.wind_speed_ms), &[]);
            let speed = wind(sys, f.current.wind_speed_ms, l);
            match f.current.wind_gust_ms {
                Some(g) if g > f.current.wind_speed_ms + 2.0 =>
                    l.t("speak.wind_gust", &[("band", &band), ("speed", &speed), ("gust", &wind(sys, g, l))]),
                _ => l.t("speak.wind", &[("band", &band), ("speed", &speed)]),
            }
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
            match prob {
                // A probability (Open-Meteo daily) → likelihood phrasing.
                Some(_) => l.t("speak.rain", &[("band", &l.t(rain_band(prob), &[])), ("mm", &l.num(mm))]),
                // No probability (MET, or a current snapshot) → answer from the
                // amount so we never say "[blank] — about 0 millimetres".
                None if mm >= 0.1 => l.t("speak.rain_amount", &[("mm", &l.num(mm))]),
                None => l.t("speak.rain_none", &[]),
            }
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
        let wk = l.t(when_key(when), &[]);
        // "This week" summarises the whole week (max high, min low, dominant
        // condition) so the spoken line matches the card's summary chip. A
        // single day (today/tomorrow) speaks that day's own figures.
        let (hi_c, lo_c, cond) = if when == When::ThisWeek {
            let (max_hi, min_lo) = f.week_extremes();
            let dom = l.t(f.dominant_daily_condition().label_key(), &[]);
            (max_hi, min_lo, l.t("speak.mostly", &[("cond", &dom)]))
        } else {
            let day = day_at(f, when);
            (day.temp_max_c, day.temp_min_c, l.t(day.condition.label_key(), &[]))
        };
        let speak = match &place {
            Some(pl) => l.t("speak.forecast_place", &[("when", &wk), ("place", pl),
                ("hi", &temp(sys, hi_c, l)), ("lo", &temp(sys, lo_c, l)), ("cond", &cond)]),
            None => l.t("speak.forecast_no_place", &[("when", &wk),
                ("hi", &temp(sys, hi_c, l)), ("lo", &temp(sys, lo_c, l)), ("cond", &cond)]),
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
    }

    #[test]
    fn current_envelope_is_a_stat_card() {
        let env = build(&current_only(), When::Now, Facet::None, System::Metric, "en", &Fakes);
        assert!(env.contains("\"stat\""));
        assert!(env.contains("\"headline\""));
        assert!(env.contains("asset:ui/wind.webp"));
        assert!(env.contains("asset:heroes/"));        // a background was set
        assert!(env.contains("Open-Meteo"));           // footer attribution
    }

    #[test]
    fn forecast_envelope_is_a_list_card() {
        let env = build(&with_daily(), When::ThisWeek, Facet::None, System::Metric, "en", &Fakes);
        assert!(env.contains("\"list\""));
        assert!(env.contains("\"rows\""));
        assert!(env.contains("\"leading\""));
        assert!(env.contains("card.row_temps"));        // trailing temps key surfaced by Fakes
    }

    #[test]
    fn this_week_speak_summarises_the_week_to_match_the_card() {
        // Regression: the spoken "this week" line used to read a single day
        // (day_at → today), disagreeing with the card's week summary. It must
        // now use the week summary ("mostly {dominant}") so speak == card.
        // with_daily(): max-high 31, min-low 20 across the two days.
        let env = build(&with_daily(), When::ThisWeek, Facet::None, System::Metric, "en", &Fakes);
        assert!(env.contains("speak.mostly"));       // week-summary phrasing, not a single day's condition
        // Both the speak and the card summary surface the week extremes via Fakes.
        assert!(env.contains("card.forecast_summary"));
    }

    #[test]
    fn rain_facet_no_probability_uses_amount() {
        let mut wet = current_only();
        wet.current.precip_probability = None;
        wet.current.precip_mm = 3.0;
        let env = build(&wet, When::Now, Facet::Rain, System::Metric, "en", &Fakes);
        assert!(env.contains("speak.rain_amount"));
        let mut dry = current_only();
        dry.current.precip_probability = None;
        dry.current.precip_mm = 0.0;
        let env2 = build(&dry, When::Now, Facet::Rain, System::Metric, "en", &Fakes);
        assert!(env2.contains("speak.rain_none"));
    }

    #[test]
    fn rain_facet_with_probability_uses_band() {
        // Tomorrow → daily[1] precip_probability 80 → very_likely band.
        let env = build(&with_daily(), When::Tomorrow, Facet::Rain, System::Metric, "en", &Fakes);
        assert!(env.contains("rain.very_likely"));
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
        assert!(env.contains("card.row_temps"));        // per-day temps row template key used
    }

    #[test]
    fn wind_facet_reports_gusts_when_present() {
        let env = build(&current_only(), When::Now, Facet::Wind, System::Metric, "en", &Fakes);
        assert!(env.contains("speak.wind_gust"));
    }

    #[test]
    fn uv_facet_on_forecast_uses_daily_max() {
        // when=Tomorrow → daily[1].uv_index_max = 6.0 → "high" band (6..8)
        let env = build(&with_daily(), When::Tomorrow, Facet::Uv, System::Metric, "en", &Fakes);
        assert!(env.contains("uv.high"));
    }
}
