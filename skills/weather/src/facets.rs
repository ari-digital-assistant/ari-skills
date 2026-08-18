/// Wind speed band (m/s), Beaufort-informed. Returns a strings key.
pub fn wind_band(ms: f64) -> &'static str {
    if ms < 3.0 { "wind.calm" }
    else if ms < 6.0 { "wind.light" }
    else if ms < 11.0 { "wind.breezy" }
    else if ms < 17.0 { "wind.windy" }
    else { "wind.gale" }
}

/// Rain likelihood from precip probability (0..100).
pub fn rain_band(probability: f64) -> &'static str {
    if probability < 20.0 { "rain.unlikely" }
    else if probability < 50.0 { "rain.possible" }
    else if probability < 80.0 { "rain.likely" }
    else { "rain.very_likely" }
}

/// Relative humidity band (0..100). Returns a strings key.
pub fn humidity_band(pct: f64) -> &'static str {
    if pct < 30.0 { "humidity.dry" }
    else if pct < 60.0 { "humidity.comfortable" }
    else if pct < 80.0 { "humidity.humid" }
    else { "humidity.very_humid" }
}

/// Meteorological wind direction (degrees the wind blows *from*) to an
/// 8-point compass key. Spoken answers don't benefit from 16 points —
/// "west-northwest" is a mouthful nobody asked for.
pub fn compass_point(deg: f64) -> &'static str {
    const POINTS: [&str; 8] = ["dir.n", "dir.ne", "dir.e", "dir.se",
                               "dir.s", "dir.sw", "dir.w", "dir.nw"];
    // +382.5 is 360 + half a sector: it shifts the whole range positive (no
    // `rem_euclid`, which is std-only) so the truncating cast rounds to the
    // nearest point rather than saturating at zero on a negative bearing.
    let idx = ((deg + 382.5) / 45.0) as usize % 8;
    POINTS[idx]
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
        assert_eq!(rain_band(10.0), "rain.unlikely");
        assert_eq!(rain_band(40.0), "rain.possible");
        assert_eq!(rain_band(70.0), "rain.likely");
        assert_eq!(rain_band(90.0), "rain.very_likely");
    }
    #[test]
    fn humidity_bands() {
        assert_eq!(humidity_band(20.0), "humidity.dry");
        assert_eq!(humidity_band(45.0), "humidity.comfortable");
        assert_eq!(humidity_band(65.0), "humidity.humid");
        assert_eq!(humidity_band(85.0), "humidity.very_humid");
    }
    #[test]
    fn compass_points_cover_every_sector() {
        assert_eq!(compass_point(0.0), "dir.n");
        assert_eq!(compass_point(359.0), "dir.n");
        assert_eq!(compass_point(45.0), "dir.ne");
        assert_eq!(compass_point(90.0), "dir.e");
        assert_eq!(compass_point(135.0), "dir.se");
        assert_eq!(compass_point(181.0), "dir.s");
        assert_eq!(compass_point(225.0), "dir.sw");
        assert_eq!(compass_point(270.0), "dir.w");
        assert_eq!(compass_point(315.0), "dir.nw");
    }
    #[test]
    fn compass_sector_boundaries_round_to_the_nearer_point() {
        // 22.5 is the N/NE boundary — it belongs to NE, 22.4 still to N.
        assert_eq!(compass_point(22.4), "dir.n");
        assert_eq!(compass_point(22.5), "dir.ne");
        assert_eq!(compass_point(337.5), "dir.n");
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
