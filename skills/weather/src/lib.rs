#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

mod conditions;
mod dates;
mod facets;
mod forecast;
mod metno;
mod openmeteo;
mod present;
mod router;
mod units;

#[cfg(target_arch = "wasm32")]
use ari_skill_sdk as ari;

/// Ceremonial — the manifest's `matching.patterns` score this skill
/// (`custom_score: false`), so the host never calls this export.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn score(_ptr: i32, _len: i32) -> f32 {
    0.95
}

#[cfg(target_arch = "wasm32")]
use present::L10n;

#[cfg(target_arch = "wasm32")]
struct SdkL10n;
#[cfg(target_arch = "wasm32")]
impl L10n for SdkL10n {
    fn t(&self, key: &str, args: &[(&str, &str)]) -> alloc::string::String {
        use alloc::string::ToString;
        ari::t(key, args).unwrap_or(key).to_string()
    }
    fn num(&self, v: f64) -> alloc::string::String {
        // Whole-number rendering — weather values read best rounded.
        // `f64::round` lives in std (libm intrinsic), unavailable under
        // no_std/wasm, so round-half-away-from-zero by hand.
        let r = if v >= 0.0 { (v + 0.5) as i64 } else { (v - 0.5) as i64 };
        alloc::format!("{}", r)
    }
    fn day_label(&self, iso_date: &str) -> alloc::string::String {
        use alloc::string::ToString;
        match dates::iso_weekday(iso_date) {
            Some(wd) => {
                let key = match wd {
                    0 => "weekday.0", 1 => "weekday.1", 2 => "weekday.2", 3 => "weekday.3",
                    4 => "weekday.4", 5 => "weekday.5", _ => "weekday.6",
                };
                ari::t(key, &[]).map(|s| s.to_string()).unwrap_or_else(|| iso_date.to_string())
            }
            None => iso_date.to_string(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let raw = unsafe { ari::input(ptr, len) };
    let args = ari::args();
    let req = router::parse_request(args, raw);
    let locale = ari::get_locale();
    let unit_setting = ari::setting_get("units").unwrap_or("auto");
    let sys = units::system_for(unit_setting, locale);

    let forecast = match resolve_and_fetch(&req, locale) {
        Ok(f) => f,
        Err(msg_key) => {
            let msg = ari::t(msg_key, &[]).unwrap_or("Sorry, I couldn't get the weather.");
            let env = ari::presentation::Envelope::new().speak(msg).to_json();
            return ari::respond_action(&env);
        }
    };
    let env = present::build(&forecast, req.when, req.facet, sys, locale, &SdkL10n);
    ari::respond_action(&env)
}

/// Resolve coordinates + fetch the right backend. Returns a `Forecast` or a
/// strings key for the error to speak.
#[cfg(target_arch = "wasm32")]
fn resolve_and_fetch(req: &router::Request, locale: &str) -> Result<forecast::Forecast, &'static str> {
    if let Some(place) = &req.location {
        let gbody = http_get(&openmeteo::geocode_url(place, locale)).ok_or("err.network")?;
        let hit = openmeteo::parse_geocode(&gbody).map_err(|_| "err.network")?.ok_or("err.not_found")?;
        let fbody = http_get(&openmeteo::forecast_url(hit.lat, hit.lon)).ok_or("err.network")?;
        return openmeteo::parse_forecast(&fbody, Some(hit.name)).map_err(|_| "err.network");
    }
    let loc = ari::location();
    match loc.status {
        ari::LocationStatus::Ok => {
            if req.use_metno() {
                let body = http_get(&metno::forecast_url(loc.lat, loc.lon)).ok_or("err.network")?;
                metno::parse_current(&body).map_err(|_| "err.network")
            } else {
                let body = http_get(&openmeteo::forecast_url(loc.lat, loc.lon)).ok_or("err.network")?;
                openmeteo::parse_forecast(&body, None).map_err(|_| "err.network")
            }
        }
        _ => Err("err.location_off"),
    }
}

/// GET with a tiny storage_kv cache (~10 min TTL). Cache value is
/// `"<epoch_ms>\n<body>"`. Returns the body on a 2xx (or fresh cache hit).
#[cfg(target_arch = "wasm32")]
fn http_get(url: &str) -> Option<alloc::string::String> {
    use alloc::string::ToString;
    use alloc::format;
    const TTL_MS: i64 = 600_000;
    let key = format!("wx:{url}");
    if let Some(cached) = ari::storage_get(&key) {
        if let Some((ts_str, body)) = cached.split_once('\n') {
            if let Ok(ts) = ts_str.parse::<i64>() {
                if ari::now_ms() - ts < TTL_MS {
                    return Some(body.to_string());
                }
            }
        }
    }
    let resp = ari::http_request("GET", url, &[], None);
    if resp.status >= 200 && resp.status < 300 {
        if let Some(body) = resp.body {
            let _ = ari::storage_set(&key, &format!("{}\n{}", ari::now_ms(), body));
            return Some(body);
        }
    }
    None
}
