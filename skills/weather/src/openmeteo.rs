//! Open-Meteo backend parsers — geocoding (place name → coordinates) and
//! forecasts. Pure parsing — no I/O; the host performs the actual HTTP fetch
//! and hands us the response body.

use alloc::format;
use alloc::string::{String, ToString};

#[derive(Debug, Clone, PartialEq)]
pub struct GeoHit {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub country: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError(pub &'static str);

/// Percent-encode spaces only (place names rarely need more for this API).
fn enc(s: &str) -> String {
    s.replace(' ', "%20")
}

pub fn geocode_url(place: &str, locale: &str) -> String {
    format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language={}&format=json",
        enc(place),
        locale
    )
}

/// `Ok(None)` = no match; `Ok(Some)` = first hit; `Err` = malformed JSON.
pub fn parse_geocode(body: &str) -> Result<Option<GeoHit>, ParseError> {
    let v: serde_json::Value = serde_json::from_str(body).map_err(|_| ParseError("bad json"))?;
    let arr = match v.get("results").and_then(|r| r.as_array()) {
        Some(a) if !a.is_empty() => a,
        _ => return Ok(None),
    };
    let r = &arr[0];
    let lat = r
        .get("latitude")
        .and_then(|x| x.as_f64())
        .ok_or(ParseError("no lat"))?;
    let lon = r
        .get("longitude")
        .and_then(|x| x.as_f64())
        .ok_or(ParseError("no lon"))?;
    let name = r
        .get("name")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let country = r
        .get("country")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    Ok(Some(GeoHit {
        name,
        lat,
        lon,
        country,
    }))
}

use alloc::vec::Vec;
use crate::conditions::condition_from_wmo;
use crate::forecast::{Conditions, DailyConditions, Forecast, HourlyConditions};

pub fn forecast_url(lat: f64, lon: f64) -> String {
    format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}\
&current=temperature_2m,relative_humidity_2m,apparent_temperature,is_day,precipitation,weather_code,wind_speed_10m,wind_gusts_10m,wind_direction_10m,uv_index\
&hourly=precipitation_probability,wind_speed_10m,wind_direction_10m\
&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_sum,precipitation_probability_max,uv_index_max\
&timezone=auto&forecast_days=7&wind_speed_unit=ms"
    )
}

fn f64_at(v: &serde_json::Value, key: &str) -> Option<f64> { v.get(key).and_then(|x| x.as_f64()) }

pub fn parse_forecast(body: &str, place_label: Option<String>) -> Result<Forecast, ParseError> {
    let v: serde_json::Value = serde_json::from_str(body).map_err(|_| ParseError("bad json"))?;
    let cur = v.get("current").ok_or(ParseError("no current"))?;
    let code = cur.get("weather_code").and_then(|x| x.as_u64()).unwrap_or(9999) as u16;
    let current = Conditions {
        temp_c: f64_at(cur, "temperature_2m").ok_or(ParseError("no temp"))?,
        feels_like_c: f64_at(cur, "apparent_temperature")
            .or_else(|| f64_at(cur, "temperature_2m")).unwrap_or(0.0),
        condition: condition_from_wmo(code),
        is_day: cur.get("is_day").and_then(|x| x.as_i64()).unwrap_or(1) == 1,
        wind_speed_ms: f64_at(cur, "wind_speed_10m").unwrap_or(0.0),
        wind_gust_ms: f64_at(cur, "wind_gusts_10m"),
        wind_direction_deg: f64_at(cur, "wind_direction_10m"),
        precip_mm: f64_at(cur, "precipitation").unwrap_or(0.0),
        humidity_pct: f64_at(cur, "relative_humidity_2m"),
        uv_index: f64_at(cur, "uv_index"),
    };

    let daily = v.get("daily").ok_or(ParseError("no daily"))?;
    let times = daily.get("time").and_then(|x| x.as_array()).ok_or(ParseError("no daily.time"))?;
    let codes = daily.get("weather_code").and_then(|x| x.as_array());
    let tmax = daily.get("temperature_2m_max").and_then(|x| x.as_array());
    let tmin = daily.get("temperature_2m_min").and_then(|x| x.as_array());
    let psum = daily.get("precipitation_sum").and_then(|x| x.as_array());
    let pprob = daily.get("precipitation_probability_max").and_then(|x| x.as_array());
    let uvmax = daily.get("uv_index_max").and_then(|x| x.as_array());
    let at = |a: &Option<&Vec<serde_json::Value>>, i: usize| -> Option<f64> {
        a.and_then(|arr| arr.get(i)).and_then(|x| x.as_f64())
    };
    let mut days = Vec::with_capacity(times.len());
    for i in 0..times.len() {
        let date = times[i].as_str().unwrap_or("").to_string();
        let wc = codes.and_then(|c| c.get(i)).and_then(|x| x.as_u64()).unwrap_or(9999) as u16;
        days.push(DailyConditions {
            date,
            temp_min_c: at(&tmin, i).unwrap_or(0.0),
            temp_max_c: at(&tmax, i).unwrap_or(0.0),
            condition: condition_from_wmo(wc),
            precip_mm: at(&psum, i).unwrap_or(0.0),
            precip_probability: at(&pprob, i),
            uv_index_max: at(&uvmax, i),
        });
    }
    // `timezone=auto` means every timestamp in the response is already local
    // to the forecast location, so hour-of-day needs no conversion.
    let local_now = cur.get("time").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let hourly = parse_hourly(&v);
    Ok(Forecast { place_label, local_now, current, daily: days, hourly })
}

fn parse_hourly(v: &serde_json::Value) -> Vec<HourlyConditions> {
    let Some(h) = v.get("hourly") else { return Vec::new() };
    let Some(times) = h.get("time").and_then(|x| x.as_array()) else { return Vec::new() };
    let prob = h.get("precipitation_probability").and_then(|x| x.as_array());
    let speed = h.get("wind_speed_10m").and_then(|x| x.as_array());
    let dir = h.get("wind_direction_10m").and_then(|x| x.as_array());
    let at = |a: &Option<&Vec<serde_json::Value>>, i: usize| -> Option<f64> {
        a.and_then(|arr| arr.get(i)).and_then(|x| x.as_f64())
    };
    (0..times.len())
        .filter_map(|i| {
            let time = times[i].as_str()?.to_string();
            Some(HourlyConditions {
                time,
                precip_probability: at(&prob, i),
                wind_speed_ms: at(&speed, i).unwrap_or(0.0),
                wind_direction_deg: at(&dir, i),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn geocode_url_truncates_and_encodes() {
        assert_eq!(
            geocode_url("New York", "en"),
            "https://geocoding-api.open-meteo.com/v1/search?name=New%20York&count=1&language=en&format=json"
        );
    }
    #[test]
    fn parse_geocode_found() {
        let body = include_str!("fixtures/geocode_tokyo.json");
        let g = parse_geocode(body).unwrap().unwrap();
        assert_eq!(g.name, "Tokyo");
        assert!((g.lat - 35.6895).abs() < 0.001);
        assert!((g.lon - 139.69171).abs() < 0.001);
        assert_eq!(g.country.as_deref(), Some("Japan"));
    }
    #[test]
    fn parse_geocode_not_found() {
        let body = include_str!("fixtures/geocode_none.json");
        assert!(parse_geocode(body).unwrap().is_none());
    }
    #[test]
    fn forecast_url_has_expected_params() {
        let u = forecast_url(35.9, 14.51);
        assert!(u.starts_with("https://api.open-meteo.com/v1/forecast?latitude=35.9&longitude=14.51"));
        assert!(u.contains("wind_speed_unit=ms"));
        assert!(u.contains("wind_direction_10m"));
        assert!(u.contains("hourly=precipitation_probability"));
        assert!(u.contains("timezone=auto"));
        assert!(u.contains("forecast_days=7"));
    }
    #[test]
    fn parse_forecast_current_and_daily() {
        let body = include_str!("fixtures/openmeteo_forecast.json");
        let f = parse_forecast(body, Some("Valletta".into())).unwrap();
        assert_eq!(f.place_label.as_deref(), Some("Valletta"));
        assert_eq!(f.local_now, "2026-08-18T15:15");
        assert!(f.current.temp_c.is_finite());
        assert!(f.current.uv_index.is_some());
        assert_eq!(f.daily.len(), 7);
        assert!(f.daily[0].temp_min_c <= f.daily[0].temp_max_c);
        assert_eq!(f.daily[0].date.len(), 10); // "YYYY-MM-DD"
        assert_eq!(f.current.wind_direction_deg, Some(181.0));
    }

    #[test]
    fn parse_forecast_hourly_series() {
        let body = include_str!("fixtures/openmeteo_forecast.json");
        let f = parse_forecast(body, None).unwrap();
        assert_eq!(f.hourly.len(), 48);
        assert_eq!(f.hourly[0].time, "2026-08-18T00:00");
        assert_eq!(f.hourly[15].precip_probability, Some(84.0));
        assert!(f.hourly[15].wind_direction_deg.is_some());
    }

    #[test]
    fn parse_forecast_outlook_finds_the_afternoon_peak() {
        // Singapore sample: a dry morning building to 84% at 2-3pm.
        let body = include_str!("fixtures/openmeteo_forecast.json");
        let f = parse_forecast(body, None).unwrap();
        let o = f.precip_outlook("2026-08-18", 9).unwrap();
        assert_eq!(o.max_prob, 84.0);
        assert_eq!(o.peak_hour, Some(14));
    }

    #[test]
    fn parse_forecast_without_hourly_block_is_still_valid() {
        let body = r#"{"current":{"time":"2026-08-18T15:15","temperature_2m":20.0},
            "daily":{"time":["2026-08-18"]}}"#;
        let f = parse_forecast(body, None).unwrap();
        assert!(f.hourly.is_empty());
        assert_eq!(f.daily.len(), 1);
    }
}
