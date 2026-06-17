//! Open-Meteo backend parsers. This module covers the geocoding half:
//! turning a place name into coordinates via Open-Meteo's keyless geocoding
//! API. Pure parsing — no I/O; the host performs the actual HTTP fetch and
//! hands us the response body.
//!
//! [`ParseError`] is shared with the MET Norway parser (`metno.rs`, later).
//! Public items are wired up by `lib.rs` in a later task.
#![allow(dead_code)] // consumed by lib.rs wire-up (later task)

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
}
