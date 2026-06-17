#![allow(dead_code)] // consumed by present/lib (later tasks)

/// Wind speed band (m/s), Beaufort-informed. Returns a strings key.
pub fn wind_band(ms: f64) -> &'static str {
    if ms < 3.0 { "wind.calm" }
    else if ms < 6.0 { "wind.light" }
    else if ms < 11.0 { "wind.breezy" }
    else if ms < 17.0 { "wind.windy" }
    else { "wind.gale" }
}

/// Rain likelihood from precip probability (0..100). `None` → caller
/// should fall back to amount-based phrasing.
pub fn rain_band(probability: Option<f64>) -> &'static str {
    match probability {
        None => "rain.amount_only",
        Some(p) if p < 20.0 => "rain.unlikely",
        Some(p) if p < 50.0 => "rain.possible",
        Some(p) if p < 80.0 => "rain.likely",
        Some(_) => "rain.very_likely",
    }
}

/// WHO UV index band.
pub fn uv_band(uv: f64) -> &'static str {
    if uv < 3.0 { "uv.low" }
    else if uv < 6.0 { "uv.moderate" }
    else if uv < 8.0 { "uv.high" }
    else if uv < 11.0 { "uv.very_high" }
    else { "uv.extreme" }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wind_bands() {
        assert_eq!(wind_band(2.0), "wind.calm");
        assert_eq!(wind_band(5.0), "wind.light");
        assert_eq!(wind_band(9.0), "wind.breezy");
        assert_eq!(wind_band(14.0), "wind.windy");
        assert_eq!(wind_band(20.0), "wind.gale");
    }
    #[test]
    fn rain_bands_by_probability() {
        assert_eq!(rain_band(Some(10.0)), "rain.unlikely");
        assert_eq!(rain_band(Some(40.0)), "rain.possible");
        assert_eq!(rain_band(Some(70.0)), "rain.likely");
        assert_eq!(rain_band(Some(90.0)), "rain.very_likely");
    }
    #[test]
    fn rain_band_without_probability_uses_none_marker() {
        assert_eq!(rain_band(None), "rain.amount_only");
    }
    #[test]
    fn uv_bands() {
        assert_eq!(uv_band(1.0), "uv.low");
        assert_eq!(uv_band(4.0), "uv.moderate");
        assert_eq!(uv_band(7.0), "uv.high");
        assert_eq!(uv_band(9.0), "uv.very_high");
        assert_eq!(uv_band(11.0), "uv.extreme");
    }
}
