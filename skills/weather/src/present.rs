use alloc::string::String;
use ari_skill_sdk::presentation as p;
use crate::forecast::Forecast;
use crate::router::{When, Facet};
use crate::units::{System, c_to_f, ms_to_kmh, ms_to_mph};
use crate::facets::{wind_band, rain_band, uv_band, humidity_band, compass_point};

/// Host-call seam: the wasm impl wraps `ari::t` / `ari::format_number` and a
/// weekday formatter; tests inject a fake.
pub trait L10n {
    fn t(&self, key: &str, args: &[(&str, &str)]) -> String;
    fn num(&self, v: f64) -> String;
    /// Localised short weekday for an ISO "YYYY-MM-DD" date.
    fn day_label(&self, iso_date: &str) -> String;
    /// Localised clock label for an hour of the day — "3pm" in English,
    /// "le 15" in Italian. The host has no time-of-day formatter, so this is
    /// assembled from strings templates: each language's `time.hour` picks
    /// whichever of the 12- and 24-hour arguments it actually uses.
    fn hour_label(&self, hour24: u8) -> String {
        let h12 = alloc::format!("{}", if hour24.is_multiple_of(12) { 12 } else { hour24 % 12 });
        let h24 = alloc::format!("{hour24}");
        let ampm = self.t(if hour24 < 12 { "time.am" } else { "time.pm" }, &[]);
        self.t("time.hour", &[("h12", &h12), ("h24", &h24), ("ampm", &ampm)])
    }
}

const ATTRIBUTION: &str = "Weather data by Open-Meteo.com";

/// Under this the rain isn't worth a sentence — nobody needs telling about a
/// 6% chance.
const RAIN_MENTION_PCT: f64 = 20.0;
/// Under this the air is still enough that quoting a bearing is noise.
const WIND_MENTION_MS: f64 = 1.0;

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

/// The `(date, first hour)` slice of the hourly series a request is about.
/// Today starts at the current hour — a downpour that already happened isn't
/// an answer to "will it rain". Later days start at midnight.
fn hourly_window(f: &Forecast, when: When) -> Option<(&str, u8)> {
    if f.daily.is_empty() { return None; }
    let idx = target_index(when).min(f.daily.len() - 1);
    let from = if idx == 0 { f.local_hour() } else { 0 };
    Some((&f.daily[idx].date, from))
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
        .footer(p::IconText::new(ATTRIBUTION).icon(p::Asset::new("ui/shield.webp")));
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
    list = list.footer(p::IconText::new(ATTRIBUTION).icon(p::Asset::new("ui/shield.webp")));
    p::Card::new("weather_forecast").title(title).subtitle(subtitle).list(list)
}

/// "There's a 40% chance of rain, mostly around 3pm." Omitted entirely when
/// the day is dry enough that saying so is filler.
fn rain_sentence(f: &Forecast, l: &dyn L10n) -> Option<String> {
    let (date, from_hour) = hourly_window(f, When::Now)?;
    let outlook = f.precip_outlook(date, from_hour)?;
    if outlook.max_prob < RAIN_MENTION_PCT { return None; }
    let pct = l.num(outlook.max_prob);
    Some(match outlook.peak_hour {
        Some(h) => l.t("speak.detail_rain_peak", &[("pct", &pct), ("time", &l.hour_label(h))]),
        None => l.t("speak.detail_rain", &[("pct", &pct)]),
    })
}

/// "Wind is northwest at 10 km/h and humidity is 60%." Drops the wind half on
/// a still day (a bearing for 2 km/h is meaningless) and the humidity half
/// when the provider didn't send one.
fn air_sentence(f: &Forecast, sys: System, l: &dyn L10n) -> Option<String> {
    let humidity = f.current.humidity_pct.map(|h| l.num(h));
    let blowing = f.current.wind_speed_ms >= WIND_MENTION_MS;
    let bearing = if blowing { f.current.wind_direction_deg } else { None };
    match (bearing, humidity) {
        (Some(deg), Some(pct)) => Some(l.t("speak.detail_wind_humidity", &[
            ("dir", &l.t(compass_point(deg), &[])),
            ("speed", &wind(sys, f.current.wind_speed_ms, l)),
            ("pct", &pct)])),
        (Some(deg), None) => Some(l.t("speak.detail_wind", &[
            ("dir", &l.t(compass_point(deg), &[])),
            ("speed", &wind(sys, f.current.wind_speed_ms, l))])),
        (None, Some(pct)) => Some(l.t("speak.detail_humidity", &[("pct", &pct)])),
        (None, None) => None,
    }
}

fn wind_speak(f: &Forecast, when: When, sys: System, l: &dyn L10n) -> String {
    // A future day has no "current" wind. Read the windiest hour of that day
    // instead of answering about right now, which is a different question.
    let (speed, bearing, gust) = match hourly_window(f, when) {
        Some((date, from)) if when != When::Now => match f.wind_outlook(date, from) {
            Some((s, d)) => (s, d, None),
            None => (f.current.wind_speed_ms, f.current.wind_direction_deg, f.current.wind_gust_ms),
        },
        _ => (f.current.wind_speed_ms, f.current.wind_direction_deg, f.current.wind_gust_ms),
    };
    let band = l.t(wind_band(speed), &[]);
    let speed_s = wind(sys, speed, l);
    // Gusts only earn a mention when they're meaningfully above the steady
    // speed; otherwise it's the same number twice.
    let gust_s = gust.filter(|g| *g > speed + 2.0).map(|g| wind(sys, g, l));
    match (bearing, gust_s) {
        (Some(deg), Some(g)) => l.t("speak.wind_gust_dir", &[("band", &band),
            ("dir", &l.t(compass_point(deg), &[])), ("speed", &speed_s), ("gust", &g)]),
        (Some(deg), None) => l.t("speak.wind_dir", &[("band", &band),
            ("dir", &l.t(compass_point(deg), &[])), ("speed", &speed_s)]),
        (None, Some(g)) => l.t("speak.wind_gust", &[("band", &band), ("speed", &speed_s), ("gust", &g)]),
        (None, None) => l.t("speak.wind", &[("band", &band), ("speed", &speed_s)]),
    }
}

fn rain_speak(f: &Forecast, when: When, l: &dyn L10n) -> String {
    let outlook = hourly_window(f, when).and_then(|(d, h)| f.precip_outlook(d, h));
    let day = (!f.daily.is_empty()).then(|| day_at(f, when));
    // Hourly is the sharper answer; the daily max is the fallback when the
    // series doesn't reach that far.
    let probability = outlook.as_ref().map(|o| o.max_prob)
        .or_else(|| day.and_then(|d| d.precip_probability));
    let mm = day.map_or(f.current.precip_mm, |d| d.precip_mm);
    match probability {
        Some(prob) => {
            let band = l.t(rain_band(prob), &[]);
            let pct = l.num(prob);
            match outlook.and_then(|o| o.peak_hour) {
                Some(h) => l.t("speak.rain_peak", &[("band", &band), ("pct", &pct),
                    ("time", &l.hour_label(h))]),
                None => l.t("speak.rain", &[("band", &band), ("pct", &pct)]),
            }
        }
        // No probability anywhere → answer from the amount so we never say
        // "[blank] — about 0 millimetres".
        None if mm >= 0.1 => l.t("speak.rain_amount", &[("mm", &l.num(mm))]),
        None => l.t("speak.rain_none", &[]),
    }
}

fn facet_speak(f: &Forecast, when: When, facet: Facet, sys: System, l: &dyn L10n) -> String {
    match facet {
        Facet::Wind => wind_speak(f, when, sys, l),
        Facet::Rain => rain_speak(f, when, l),
        Facet::Uv => {
            let uv = if when != When::Now && !f.daily.is_empty() {
                day_at(f, when).uv_index_max
            } else { f.current.uv_index };
            match uv {
                Some(u) => l.t("speak.uv", &[("band", &l.t(uv_band(u), &[])), ("value", &l.num(u))]),
                None => l.t("speak.uv_unknown", &[]),
            }
        }
        Facet::Humidity => match f.current.humidity_pct {
            Some(h) => l.t("speak.humidity", &[("pct", &l.num(h)),
                ("band", &l.t(humidity_band(h), &[]))]),
            None => l.t("speak.humidity_unknown", &[]),
        },
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

    // Multi-day forecast. "Today" is deliberately not one of these: "how is
    // the weather today" is asking what it's like out there, not for a week's
    // list, so it falls through to current conditions exactly as the bare
    // question does. The variant still earns its keep in the facet answers
    // above, where "today" means the day's peak rather than this minute's
    // reading.
    if matches!(when, When::Tomorrow | When::ThisWeek) && !f.daily.is_empty() {
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

    // Current conditions (default): a lead sentence, then whichever details
    // earn their place.
    let cond_label = l.t(f.current.condition.label_key(), &[]);
    let temp_s = temp(sys, f.current.temp_c, l);
    let feels_s = temp(sys, f.current.feels_like_c, l);
    let mut speak = match &place {
        Some(pl) => l.t("speak.current_place",
            &[("place", pl), ("temp", &temp_s), ("cond", &cond_label), ("feels", &feels_s)]),
        None => l.t("speak.current_no_place",
            &[("temp", &temp_s), ("cond", &cond_label), ("feels", &feels_s)]),
    };
    for sentence in [rain_sentence(f, l), air_sentence(f, sys, l)].into_iter().flatten() {
        speak.push(' ');
        speak.push_str(&sentence);
    }
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

    // Fake i18n: echo "key" plus its arg values (so nested band/dir keys
    // passed as args surface in the rendered string), round numbers, echo
    // the date for day labels, "15h" for hour labels.
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
        fn hour_label(&self, hour24: u8) -> String { alloc::format!("{hour24}h") }
    }

    fn current_only() -> Forecast {
        Forecast { place_label: Some("Valletta".to_string()),
            local_now: "2026-06-17T12:00".to_string(),
            current: Conditions { temp_c: 14.0, feels_like_c: 13.0, condition: Condition::PartlyCloudy,
                is_day: true, wind_speed_ms: 8.0, wind_gust_ms: Some(12.0), wind_direction_deg: Some(315.0),
                precip_mm: 0.0, humidity_pct: Some(60.0), uv_index: Some(5.0) },
            daily: Vec::new(), hourly: Vec::new() }
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

    fn hour(date: &str, h: u8, prob: f64, wind: f64, dir: f64) -> HourlyConditions {
        HourlyConditions { time: alloc::format!("{date}T{h:02}:00"),
            precip_probability: Some(prob), wind_speed_ms: wind, wind_direction_deg: Some(dir) }
    }

    /// Today's afternoon spikes to 80% at 3pm; tomorrow is a steady blow.
    fn with_hourly() -> Forecast {
        let mut f = with_daily();
        f.hourly = alloc::vec![
            hour("2026-06-17", 12, 5.0, 3.0, 90.0),
            hour("2026-06-17", 13, 10.0, 3.0, 90.0),
            hour("2026-06-17", 14, 30.0, 4.0, 90.0),
            hour("2026-06-17", 15, 80.0, 4.0, 90.0),
            hour("2026-06-17", 16, 20.0, 3.0, 90.0),
            hour("2026-06-17", 17, 5.0, 3.0, 90.0),
            hour("2026-06-18", 9, 70.0, 12.0, 225.0),
            hour("2026-06-18", 10, 70.0, 18.0, 225.0),
            hour("2026-06-18", 11, 70.0, 14.0, 225.0),
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
    }

    #[test]
    fn current_speak_adds_rain_peak_and_air_detail() {
        let env = build(&with_hourly(), When::Now, Facet::None, System::Metric, "en", &Fakes);
        // Lead sentence, then the rain peak (80% at 15:00), then wind + humidity.
        assert!(env.contains("speak.current_place"));
        assert!(env.contains("speak.detail_rain_peak 80 15h"));
        // 8 m/s from 315° → northwest at 29 km/h, humidity 60%.
        assert!(env.contains("speak.detail_wind_humidity dir.nw 29 km/h 60"));
    }

    #[test]
    fn current_speak_omits_the_rain_sentence_on_a_dry_day() {
        let mut f = with_hourly();
        for h in f.hourly.iter_mut() { h.precip_probability = Some(5.0); }
        let env = build(&f, When::Now, Facet::None, System::Metric, "en", &Fakes);
        assert!(!env.contains("speak.detail_rain"));
        assert!(env.contains("speak.detail_wind_humidity"));
    }

    #[test]
    fn current_speak_says_a_flat_chance_without_naming_an_hour() {
        let mut f = with_hourly();
        for h in f.hourly.iter_mut() { h.precip_probability = Some(45.0); }
        let env = build(&f, When::Now, Facet::None, System::Metric, "en", &Fakes);
        assert!(env.contains("speak.detail_rain 45"));
        assert!(!env.contains("speak.detail_rain_peak"));
    }

    #[test]
    fn current_speak_drops_the_bearing_when_the_air_is_still() {
        let mut f = with_hourly();
        f.current.wind_speed_ms = 0.4;
        let env = build(&f, When::Now, Facet::None, System::Metric, "en", &Fakes);
        assert!(env.contains("speak.detail_humidity 60"));
        assert!(!env.contains("dir."));
    }

    #[test]
    fn current_speak_has_no_air_sentence_without_wind_or_humidity() {
        let mut f = current_only();
        f.current.wind_speed_ms = 0.0;
        f.current.wind_direction_deg = None;
        f.current.humidity_pct = None;
        let env = build(&f, When::Now, Facet::None, System::Metric, "en", &Fakes);
        assert!(!env.contains("speak.detail_"));
    }

    #[test]
    fn today_asks_for_current_conditions_not_the_week_list() {
        // "How is the weather today" must answer like "how is the weather":
        // the current-conditions stat card, not the multi-day forecast list.
        let env = build(&with_daily(), When::Today, Facet::None, System::Metric, "en", &Fakes);
        assert!(env.contains("weather_current"));
        assert!(!env.contains("weather_forecast"));
        assert!(!env.contains("\"list\""));
        assert_eq!(
            spoken(&env),
            spoken(&build(&with_daily(), When::Now, Facet::None, System::Metric, "en", &Fakes)),
        );
    }

    #[test]
    fn today_still_means_the_day_not_this_minute_for_facets() {
        // The UV facet reads the daily maximum for "today" (7.0) rather than
        // the current reading (5.0) — narrowing the no-facet case must not
        // collapse that distinction.
        let today = build(&with_daily(), When::Today, Facet::Uv, System::Metric, "en", &Fakes);
        let now = build(&with_daily(), When::Now, Facet::Uv, System::Metric, "en", &Fakes);
        assert!(today.contains("speak.uv uv.high 7"));
        assert!(now.contains("speak.uv uv.moderate 5"));
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
        assert!(env.contains("card.forecast_summary"));
    }

    #[test]
    fn forecast_envelope_lists_days() {
        let env = build(&with_daily(), When::ThisWeek, Facet::None, System::Metric, "en", &Fakes);
        assert!(env.contains("2026-06-17"));            // day label (faked = the date) present
        assert!(env.contains("2026-06-18"));
        assert!(env.contains("card.row_temps"));        // per-day temps row template key used
    }

    #[test]
    fn rain_facet_names_the_peak_hour_from_the_hourly_series() {
        let env = build(&with_hourly(), When::Today, Facet::Rain, System::Metric, "en", &Fakes);
        assert!(env.contains("speak.rain_peak rain.very_likely 80 15h"));
    }

    #[test]
    fn rain_facet_ignores_hours_that_have_already_passed() {
        // Same day, but it's now 4pm — the 3pm spike is history, so the
        // answer is the 20% left in the afternoon, not 80%.
        let mut f = with_hourly();
        f.local_now = "2026-06-17T16:00".to_string();
        let env = build(&f, When::Now, Facet::Rain, System::Metric, "en", &Fakes);
        assert!(env.contains("rain.possible 20"));
        assert!(!env.contains("80"));
    }

    #[test]
    fn rain_facet_falls_back_to_the_daily_max_without_hourly() {
        // Tomorrow → daily[1] precip_probability 80 → very_likely band.
        let env = build(&with_daily(), When::Tomorrow, Facet::Rain, System::Metric, "en", &Fakes);
        assert!(env.contains("speak.rain rain.very_likely 80"));
        assert!(!env.contains("speak.rain_peak"));
    }

    #[test]
    fn rain_facet_no_probability_uses_amount() {
        let mut wet = current_only();
        wet.current.precip_mm = 3.0;
        let env = build(&wet, When::Now, Facet::Rain, System::Metric, "en", &Fakes);
        assert!(env.contains("speak.rain_amount"));
        let dry = current_only();
        let env2 = build(&dry, When::Now, Facet::Rain, System::Metric, "en", &Fakes);
        assert!(env2.contains("speak.rain_none"));
    }

    #[test]
    fn uv_facet_speaks_band() {
        let env = build(&current_only(), When::Now, Facet::Uv, System::Metric, "en", &Fakes);
        assert!(env.contains("uv.moderate"));           // uv 5 → moderate band key surfaced
    }

    #[test]
    fn uv_facet_on_forecast_uses_daily_max() {
        // when=Tomorrow → daily[1].uv_index_max = 6.0 → "high" band (6..8)
        let env = build(&with_daily(), When::Tomorrow, Facet::Uv, System::Metric, "en", &Fakes);
        assert!(env.contains("uv.high"));
    }

    #[test]
    fn wind_facet_reports_bearing_and_gusts() {
        // 8 m/s from 315° gusting 12 → breezy, northwest, 29 km/h, gusting 43.
        let env = build(&current_only(), When::Now, Facet::Wind, System::Metric, "en", &Fakes);
        assert!(env.contains("speak.wind_gust_dir wind.breezy dir.nw 29 km/h 43 km/h"));
    }

    #[test]
    fn wind_facet_drops_the_gust_when_it_matches_the_steady_speed() {
        let mut f = current_only();
        f.current.wind_gust_ms = Some(9.0); // only 1 m/s above steady — not worth saying
        let env = build(&f, When::Now, Facet::Wind, System::Metric, "en", &Fakes);
        assert!(env.contains("speak.wind_dir wind.breezy dir.nw 29 km/h"));
        assert!(!env.contains("wind_gust"));
    }

    #[test]
    fn wind_facet_answers_about_the_day_asked_for_not_right_now() {
        // Regression: "is it windy tomorrow" used to report the current wind.
        // Tomorrow peaks at 18 m/s from 225° → gale, southwest, 65 km/h.
        let env = build(&with_hourly(), When::Tomorrow, Facet::Wind, System::Metric, "en", &Fakes);
        assert!(env.contains("speak.wind_dir wind.gale dir.sw 65 km/h"));
        assert!(!env.contains("dir.nw"));               // not today's bearing
    }

    #[test]
    fn wind_facet_falls_back_to_current_when_the_day_has_no_hourly() {
        let env = build(&with_daily(), When::Tomorrow, Facet::Wind, System::Metric, "en", &Fakes);
        assert!(env.contains("wind.breezy"));           // current 8 m/s
    }

    #[test]
    fn humidity_facet_speaks_band_and_value() {
        let env = build(&current_only(), When::Now, Facet::Humidity, System::Metric, "en", &Fakes);
        assert!(env.contains("speak.humidity 60 humidity.humid"));
    }

    #[test]
    fn humidity_facet_without_a_reading_says_so() {
        let mut f = current_only();
        f.current.humidity_pct = None;
        let env = build(&f, When::Now, Facet::Humidity, System::Metric, "en", &Fakes);
        assert!(env.contains("speak.humidity_unknown"));
    }

    /// Renders through the shipped strings files rather than a fake, so a
    /// placeholder the code never fills (or fills under another name) fails
    /// here instead of reaching a user as a literal "{pct}".
    struct RealStrings(&'static str);
    impl RealStrings {
        fn table(&self) -> serde_json::Value {
            let raw = match self.0 {
                "it" => include_str!("../strings/it.json"),
                _ => include_str!("../strings/en.json"),
            };
            serde_json::from_str(raw).expect("strings file is valid JSON")
        }
    }
    impl L10n for RealStrings {
        fn t(&self, key: &str, args: &[(&str, &str)]) -> String {
            let table = self.table();
            let mut s = table.get(key).and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("missing strings key: {key}")).to_string();
            for (k, v) in args {
                s = s.replace(&alloc::format!("{{{k}}}"), v);
            }
            assert!(!s.contains('{'), "unfilled placeholder in {key}: {s}");
            s
        }
        fn num(&self, v: f64) -> String { alloc::format!("{}", v.round() as i64) }
        fn day_label(&self, iso_date: &str) -> String { iso_date.to_string() }
    }

    fn spoken(env: &str) -> String {
        let v: serde_json::Value = serde_json::from_str(env).unwrap();
        v.get("speak").and_then(|s| s.as_str()).unwrap().to_string()
    }

    #[test]
    fn english_current_conditions_reads_as_a_sentence() {
        let mut f = with_hourly();
        f.place_label = None;
        f.current.temp_c = 15.0;
        f.current.feels_like_c = 15.0;
        f.current.condition = Condition::Cloudy;
        f.current.wind_speed_ms = 2.8;   // 10 km/h
        let env = build(&f, When::Now, Facet::None, System::Metric, "en", &RealStrings("en"));
        assert_eq!(spoken(&env),
            "It's 15 degrees and cloudy, feeling like 15. \
80% chance of rain, mostly around 3pm. \
Wind is northwest at 10 km/h and humidity is 60%.");
    }

    #[test]
    fn english_dry_calm_day_stays_short() {
        let mut f = with_hourly();
        f.place_label = None;
        f.current.temp_c = 15.0;
        f.current.feels_like_c = 15.0;
        f.current.condition = Condition::Clear;
        f.current.wind_speed_ms = 0.2;
        f.current.humidity_pct = None;
        for h in f.hourly.iter_mut() { h.precip_probability = Some(0.0); }
        let env = build(&f, When::Now, Facet::None, System::Metric, "en", &RealStrings("en"));
        assert_eq!(spoken(&env), "It's 15 degrees and clear, feeling like 15.");
    }

    #[test]
    fn italian_current_conditions_reads_as_a_sentence() {
        let mut f = with_hourly();
        f.place_label = None;
        f.current.temp_c = 15.0;
        f.current.feels_like_c = 15.0;
        f.current.condition = Condition::Cloudy;
        f.current.wind_speed_ms = 2.8;
        let env = build(&f, When::Now, Facet::None, System::Metric, "it", &RealStrings("it"));
        assert_eq!(spoken(&env),
            "Ci sono 15 gradi, nuvoloso, percepiti 15. \
80% di probabilità di pioggia, soprattutto verso le 15. \
Il vento soffia da nord-ovest a 10 km/h, umidità 60%.");
    }

    #[test]
    fn every_facet_renders_in_both_languages() {
        let f = with_hourly();
        for locale in ["en", "it"] {
            for facet in [Facet::Wind, Facet::Rain, Facet::Uv, Facet::Humidity] {
                for when in [When::Now, When::Today, When::Tomorrow, When::ThisWeek] {
                    // RealStrings panics on a missing key or an unfilled slot.
                    let env = build(&f, when, facet, System::Metric, locale, &RealStrings(locale));
                    assert!(!spoken(&env).is_empty());
                }
            }
        }
    }

    #[test]
    fn imperial_speaks_mph_and_fahrenheit() {
        let env = build(&current_only(), When::Now, Facet::Wind, System::Imperial, "en", &Fakes);
        assert!(env.contains("18 mph"));                // 8 m/s
        assert!(env.contains("57°"));                   // 14°C on the card headline
    }
}
