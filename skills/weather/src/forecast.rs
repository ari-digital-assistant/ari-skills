use alloc::string::String;
use alloc::vec::Vec;
use crate::conditions::Condition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source { MetNorway, OpenMeteo }

#[derive(Debug, Clone, PartialEq)]
pub struct Conditions {
    pub temp_c: f64,
    pub feels_like_c: f64,
    pub condition: Condition,
    pub is_day: bool,
    pub wind_speed_ms: f64,
    pub wind_gust_ms: Option<f64>,
    pub precip_mm: f64,
    pub precip_probability: Option<f64>, // 0..100
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
pub struct Forecast {
    pub place_label: Option<String>, // None on the GPS path
    pub source: Source,
    pub current: Conditions,
    pub daily: Vec<DailyConditions>, // today first; empty for current-only
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conditions::Condition;
    #[test]
    fn forecast_shape() {
        let f = Forecast {
            place_label: None,
            source: Source::MetNorway,
            current: Conditions { temp_c: 26.4, feels_like_c: 26.4, condition: Condition::Clear,
                is_day: true, wind_speed_ms: 3.2, wind_gust_ms: None, precip_mm: 0.0,
                precip_probability: None, humidity_pct: Some(64.5), uv_index: Some(4.7) },
            daily: Vec::new(),
        };
        assert_eq!(f.current.temp_c, 26.4);
        assert_eq!(f.source, Source::MetNorway);
        assert!(f.daily.is_empty());
        assert_eq!(f.current.condition, Condition::Clear);
    }
}
