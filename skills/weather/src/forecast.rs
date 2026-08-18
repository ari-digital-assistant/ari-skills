use alloc::string::String;
use alloc::vec::Vec;
use crate::conditions::Condition;

#[derive(Debug, Clone, PartialEq)]
pub struct Conditions {
    pub temp_c: f64,
    pub feels_like_c: f64,
    pub condition: Condition,
    pub is_day: bool,
    pub wind_speed_ms: f64,
    pub wind_gust_ms: Option<f64>,
    pub wind_direction_deg: Option<f64>,
    pub precip_mm: f64,
    pub humidity_pct: Option<f64>,
    pub uv_index: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DailyConditions {
    pub date: String, // ISO "YYYY-MM-DD"
    pub temp_min_c: f64,
    pub temp_max_c: f64,
    pub condition: Condition,
    pub precip_mm: f64,
    pub precip_probability: Option<f64>,
    pub uv_index_max: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HourlyConditions {
    pub time: String, // local ISO "YYYY-MM-DDTHH:MM"
    pub precip_probability: Option<f64>,
    pub wind_speed_ms: f64,
    pub wind_direction_deg: Option<f64>,
}

/// How rain is distributed across a day's remaining hours. Distinguishes
/// "70% chance, all of it around 4pm" from "70% chance, drizzling all day" —
/// only the first is worth naming a time for.
#[derive(Debug, Clone, PartialEq)]
pub struct PrecipOutlook {
    pub max_prob: f64,
    /// `Some` only when the rain clusters around one hour.
    pub peak_hour: Option<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Forecast {
    pub place_label: Option<String>, // None on the GPS path
    pub local_now: String,           // local ISO "YYYY-MM-DDTHH:MM" at the forecast location
    pub current: Conditions,
    pub daily: Vec<DailyConditions>,  // today first
    pub hourly: Vec<HourlyConditions>,
}

/// Hour-of-day from a local ISO "YYYY-MM-DDTHH:MM" timestamp.
pub fn hour_of(time: &str) -> Option<u8> {
    time.get(11..13)?.parse().ok()
}

/// A peak counts as a peak only when it stands this far above the day's
/// average — otherwise the rain is spread out and naming an hour misleads.
const PEAK_RATIO: f64 = 1.5;
/// Below this many hours left there's no distribution to speak of.
const MIN_PEAK_WINDOW: usize = 3;

impl Forecast {
    /// The most frequent condition across the daily entries (first wins on a
    /// tie). Used for the forecast summary chip. `Unknown` if `daily` is empty.
    pub fn dominant_daily_condition(&self) -> Condition {
        let mut best = Condition::Unknown;
        let mut best_count = 0usize;
        for c in self.daily.iter().map(|d| d.condition) {
            let count = self.daily.iter().filter(|d| d.condition == c).count();
            if count > best_count { best = c; best_count = count; }
        }
        best
    }

    /// `(max daily high, min daily low)` in °C across the forecast days — the
    /// week's extremes. Used by BOTH the forecast card's summary chip and the
    /// spoken "this week" line so they can't diverge. Returns `(MIN, MAX)`
    /// sentinels for an empty `daily` (callers guard against that).
    pub fn week_extremes(&self) -> (f64, f64) {
        let max_hi = self.daily.iter().map(|d| d.temp_max_c).fold(f64::MIN, f64::max);
        let min_lo = self.daily.iter().map(|d| d.temp_min_c).fold(f64::MAX, f64::min);
        (max_hi, min_lo)
    }

    /// Local hour at the forecast location, 0 when the provider timestamp is
    /// unusable.
    pub fn local_hour(&self) -> u8 {
        hour_of(&self.local_now).unwrap_or(0)
    }

    fn hours_on<'a>(&'a self, date: &'a str, from_hour: u8) -> impl Iterator<Item = &'a HourlyConditions> + 'a {
        self.hourly.iter().filter(move |h| {
            h.time.starts_with(date) && hour_of(&h.time).is_some_and(|hr| hr >= from_hour)
        })
    }

    /// Rain distribution across `date` from `from_hour` onwards. `None` when
    /// the hourly series doesn't cover that window.
    pub fn precip_outlook(&self, date: &str, from_hour: u8) -> Option<PrecipOutlook> {
        let mut count = 0usize;
        let mut total = 0.0;
        let mut max_prob = f64::MIN;
        let mut peak_hour = 0u8;
        for h in self.hours_on(date, from_hour) {
            let Some(p) = h.precip_probability else { continue };
            if p > max_prob {
                max_prob = p;
                peak_hour = hour_of(&h.time).unwrap_or(0);
            }
            total += p;
            count += 1;
        }
        if count == 0 { return None; }
        let mean = total / count as f64;
        let clustered = count >= MIN_PEAK_WINDOW && max_prob >= mean * PEAK_RATIO;
        Some(PrecipOutlook { max_prob, peak_hour: clustered.then_some(peak_hour) })
    }

    /// Windiest hour of `date` from `from_hour` onwards, as
    /// `(speed m/s, direction at that hour)`. `None` when the hourly series
    /// doesn't cover that window.
    pub fn wind_outlook(&self, date: &str, from_hour: u8) -> Option<(f64, Option<f64>)> {
        self.hours_on(date, from_hour)
            .fold(None, |best: Option<&HourlyConditions>, h| match best {
                Some(b) if b.wind_speed_ms >= h.wind_speed_ms => Some(b),
                _ => Some(h),
            })
            .map(|h| (h.wind_speed_ms, h.wind_direction_deg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    fn day(hi: f64, lo: f64, c: Condition) -> DailyConditions {
        DailyConditions { date: String::new(), temp_min_c: lo, temp_max_c: hi,
            condition: c, precip_mm: 0.0, precip_probability: None, uv_index_max: None }
    }
    fn bare_current() -> Conditions {
        Conditions { temp_c: 0.0, feels_like_c: 0.0, condition: Condition::Clear,
            is_day: true, wind_speed_ms: 0.0, wind_gust_ms: None, wind_direction_deg: None,
            precip_mm: 0.0, humidity_pct: None, uv_index: None }
    }
    fn forecast(daily: Vec<DailyConditions>, hourly: Vec<HourlyConditions>) -> Forecast {
        Forecast { place_label: None, local_now: "2026-08-18T09:00".to_string(),
            current: bare_current(), daily, hourly }
    }
    fn hour(h: u8, prob: f64, wind: f64, dir: f64) -> HourlyConditions {
        HourlyConditions { time: alloc::format!("2026-08-18T{h:02}:00"),
            precip_probability: Some(prob), wind_speed_ms: wind, wind_direction_deg: Some(dir) }
    }

    #[test]
    fn forecast_shape() {
        let f = forecast(Vec::new(), Vec::new());
        assert_eq!(f.current.temp_c, 0.0);
        assert!(f.daily.is_empty());
        assert_eq!(f.current.condition, Condition::Clear);
    }

    #[test]
    fn dominant_condition_picks_most_frequent() {
        let mut f = forecast(alloc::vec![
            day(0.0, 0.0, Condition::Cloudy), day(0.0, 0.0, Condition::Cloudy),
            day(0.0, 0.0, Condition::Clear)], Vec::new());
        assert_eq!(f.dominant_daily_condition(), Condition::Cloudy);
        f.daily.clear();
        assert_eq!(f.dominant_daily_condition(), Condition::Unknown);
    }

    #[test]
    fn week_extremes_are_max_high_and_min_low() {
        let f = forecast(alloc::vec![
            day(33.0, 20.0, Condition::Cloudy), day(28.0, 15.0, Condition::Cloudy),
            day(31.0, 18.0, Condition::Cloudy)], Vec::new());
        // Max high across days = 33, min low across days = 15 (NOT the first day's low).
        assert_eq!(f.week_extremes(), (33.0, 15.0));
    }

    #[test]
    fn hour_of_reads_the_hour_field() {
        assert_eq!(hour_of("2026-08-18T15:00"), Some(15));
        assert_eq!(hour_of("2026-08-18T00:00"), Some(0));
        assert_eq!(hour_of("2026-08-18"), None);
        assert_eq!(hour_of("nonsense----xx:00"), None);
    }

    #[test]
    fn local_hour_falls_back_to_zero_on_a_bad_timestamp() {
        let mut f = forecast(Vec::new(), Vec::new());
        assert_eq!(f.local_hour(), 9);
        f.local_now = "rubbish".to_string();
        assert_eq!(f.local_hour(), 0);
    }

    #[test]
    fn precip_outlook_names_the_hour_when_rain_clusters() {
        let f = forecast(Vec::new(), alloc::vec![
            hour(12, 0.0, 1.0, 0.0), hour(13, 10.0, 1.0, 0.0), hour(14, 20.0, 1.0, 0.0),
            hour(15, 80.0, 1.0, 0.0), hour(16, 10.0, 1.0, 0.0), hour(17, 0.0, 1.0, 0.0)]);
        let o = f.precip_outlook("2026-08-18", 12).unwrap();
        assert_eq!(o.max_prob, 80.0);
        assert_eq!(o.peak_hour, Some(15));
    }

    #[test]
    fn precip_outlook_withholds_the_hour_when_rain_is_spread() {
        // Flat 60% all afternoon: max == mean, so no hour is worth naming.
        let f = forecast(Vec::new(), alloc::vec![
            hour(12, 60.0, 1.0, 0.0), hour(13, 60.0, 1.0, 0.0),
            hour(14, 60.0, 1.0, 0.0), hour(15, 60.0, 1.0, 0.0)]);
        let o = f.precip_outlook("2026-08-18", 12).unwrap();
        assert_eq!(o.max_prob, 60.0);
        assert_eq!(o.peak_hour, None);
    }

    #[test]
    fn precip_outlook_withholds_the_hour_for_a_window_too_short_to_have_a_shape() {
        let f = forecast(Vec::new(), alloc::vec![hour(22, 0.0, 1.0, 0.0), hour(23, 90.0, 1.0, 0.0)]);
        let o = f.precip_outlook("2026-08-18", 22).unwrap();
        assert_eq!(o.max_prob, 90.0);
        assert_eq!(o.peak_hour, None);
    }

    #[test]
    fn precip_outlook_skips_hours_already_past() {
        let f = forecast(Vec::new(), alloc::vec![
            hour(6, 90.0, 1.0, 0.0), hour(14, 5.0, 1.0, 0.0),
            hour(15, 10.0, 1.0, 0.0), hour(16, 5.0, 1.0, 0.0)]);
        // The 90% downpour was at 6am; asking at 2pm must not resurrect it.
        let o = f.precip_outlook("2026-08-18", 14).unwrap();
        assert_eq!(o.max_prob, 10.0);
    }

    #[test]
    fn precip_outlook_is_none_off_the_end_of_the_series() {
        let f = forecast(Vec::new(), alloc::vec![hour(15, 10.0, 1.0, 0.0)]);
        assert_eq!(f.precip_outlook("2026-08-19", 0), None);
        assert_eq!(f.precip_outlook("2026-08-18", 20), None);
    }

    #[test]
    fn wind_outlook_returns_the_windiest_hour_and_its_direction() {
        let f = forecast(Vec::new(), alloc::vec![
            hour(9, 0.0, 3.0, 90.0), hour(10, 0.0, 11.5, 315.0), hour(11, 0.0, 6.0, 200.0)]);
        assert_eq!(f.wind_outlook("2026-08-18", 9), Some((11.5, Some(315.0))));
        assert_eq!(f.wind_outlook("2026-08-18", 11), Some((6.0, Some(200.0))));
        assert_eq!(f.wind_outlook("2026-08-19", 0), None);
    }
}
