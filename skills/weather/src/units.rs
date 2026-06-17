use alloc::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum System { Metric, Imperial }

pub fn c_to_f(c: f64) -> f64 { c * 9.0 / 5.0 + 32.0 }
pub fn ms_to_kmh(ms: f64) -> f64 { ms * 3.6 }
pub fn ms_to_mph(ms: f64) -> f64 { ms * 2.2369362920544025 }

/// `setting` is `auto|metric|imperial`; `auto` derives from locale —
/// only `en-US` (and `en_US`) defaults to imperial, everything else metric.
pub fn system_for(setting: &str, locale: &str) -> System {
    match setting {
        "metric" => System::Metric,
        "imperial" => System::Imperial,
        _ => {
            let l: String = locale.replace('_', "-").to_lowercase();
            if l == "en-us" { System::Imperial } else { System::Metric }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversions_exact() {
        assert!((c_to_f(14.0) - 57.2).abs() < 1e-9);
        assert!((ms_to_kmh(10.0) - 36.0).abs() < 1e-9);
        assert!((ms_to_mph(10.0) - 22.369362920544025).abs() < 1e-9);
    }
    #[test]
    fn system_selection() {
        assert_eq!(system_for("imperial", "en-GB"), System::Imperial);
        assert_eq!(system_for("metric", "en-US"), System::Metric);
        assert_eq!(system_for("auto", "en-US"), System::Imperial);
        assert_eq!(system_for("auto", "it-IT"), System::Metric);
        assert_eq!(system_for("auto", "en"), System::Metric);
    }
}
