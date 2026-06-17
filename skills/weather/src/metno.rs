//! MET Norway Locationforecast 2.0 (`complete`) parser. Covers the GPS +
//! current-conditions case only — MET serves "here, now"; multi-day forecasts
//! are handled by the Open-Meteo backend. Pure parsing; the host performs the
//! HTTP fetch (MET requires an identifying `User-Agent`) and hands us the body.
//!
//! [`ParseError`] is shared with the Open-Meteo parser (`openmeteo.rs`).
//! Public items are wired up by `lib.rs` in a later task.
#![allow(dead_code)] // consumed by lib.rs wire-up (later task)

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::conditions::{condition_from_met, met_is_day};
use crate::forecast::{Conditions, Forecast, Source};
use crate::openmeteo::ParseError;

/// Build the `complete` endpoint URL. MET asks clients to truncate
/// coordinates to ~4 decimal places to improve cache hit rates.
pub fn forecast_url(lat: f64, lon: f64) -> String {
    format!(
        "https://api.met.no/weatherapi/locationforecast/2.0/complete?lat={:.4}&lon={:.4}",
        lat, lon
    )
}

/// Parse the first timeseries entry into current [`Conditions`]. `daily` is
/// always empty — this backend is current-conditions only.
pub fn parse_current(body: &str) -> Result<Forecast, ParseError> {
    let v: serde_json::Value = serde_json::from_str(body).map_err(|_| ParseError("bad json"))?;
    let ts0 = v
        .pointer("/properties/timeseries/0/data")
        .ok_or(ParseError("no ts0"))?;
    let inst = ts0
        .pointer("/instant/details")
        .ok_or(ParseError("no instant"))?;
    let temp = inst
        .get("air_temperature")
        .and_then(|x| x.as_f64())
        .ok_or(ParseError("no temp"))?;
    let symbol = ts0
        .pointer("/next_1_hours/summary/symbol_code")
        .and_then(|x| x.as_str())
        .unwrap_or("");
    let precip = ts0
        .pointer("/next_1_hours/details/precipitation_amount")
        .and_then(|x| x.as_f64())
        .unwrap_or(0.0);

    let current = Conditions {
        temp_c: temp,
        feels_like_c: temp, // MET has no apparent temperature
        condition: condition_from_met(symbol),
        is_day: met_is_day(symbol),
        wind_speed_ms: inst.get("wind_speed").and_then(|x| x.as_f64()).unwrap_or(0.0),
        wind_gust_ms: inst.get("wind_speed_of_gust").and_then(|x| x.as_f64()),
        precip_mm: precip,
        precip_probability: None, // MET `complete` omits PoP
        humidity_pct: inst.get("relative_humidity").and_then(|x| x.as_f64()),
        uv_index: inst
            .get("ultraviolet_index_clear_sky")
            .and_then(|x| x.as_f64()),
    };

    Ok(Forecast {
        place_label: None,
        source: Source::MetNorway,
        current,
        daily: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn url_truncates_to_4dp() {
        assert_eq!(
            forecast_url(35.898933, 14.514688),
            "https://api.met.no/weatherapi/locationforecast/2.0/complete?lat=35.8989&lon=14.5147"
        );
    }
    #[test]
    fn parse_current_from_complete() {
        let body = include_str!("fixtures/metno_complete.json");
        let f = parse_current(body).unwrap();
        assert_eq!(f.source, crate::forecast::Source::MetNorway);
        assert_eq!(f.place_label, None);
        assert!(f.current.temp_c.is_finite());
        assert!(f.current.humidity_pct.is_some());
        assert!(f.current.uv_index.is_some()); // ultraviolet_index_clear_sky
        assert!(f.current.precip_probability.is_none()); // MET complete has none
        assert!(f.daily.is_empty()); // current-only
    }
}
