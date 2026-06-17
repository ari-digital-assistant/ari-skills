#![allow(dead_code)] // consumed by lib.rs SdkL10n (wasm only)

/// Weekday for an ISO "YYYY-MM-DD" date, `0=Monday .. 6=Sunday`.
/// `None` if the string isn't three integer fields. Uses Howard Hinnant's
/// days-from-civil algorithm (pure integer math, no date library).
pub fn iso_weekday(date: &str) -> Option<u8> {
    let mut it = date.split('-');
    let y: i64 = it.next()?.parse().ok()?;
    let m: i64 = it.next()?.parse().ok()?;
    let d: i64 = it.next()?.parse().ok()?;
    if it.next().is_some() { return None; } // reject extra fields
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) { return None; }
    let yy = if m <= 2 { y - 1 } else { y };
    let era = (if yy >= 0 { yy } else { yy - 399 }) / 400;
    let yoe = yy - era * 400;                       // [0, 399]
    let mp = (m + 9) % 12;                            // Mar=0 .. Feb=11
    let doy = (153 * mp + 2) / 5 + d - 1;            // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    let days = era * 146097 + doe - 719468;          // days since 1970-01-01
    // 1970-01-01 is Thursday. (days+4).rem_euclid(7) gives 0=Sunday..6=Saturday;
    // shift by +6 mod 7 to make Monday=0.
    let sun0 = (days + 4).rem_euclid(7);             // 0=Sun..6=Sat
    Some(((sun0 + 6).rem_euclid(7)) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_weekdays_mon0() {
        // Mon=0 .. Sun=6
        assert_eq!(iso_weekday("1970-01-01"), Some(3)); // Thursday
        assert_eq!(iso_weekday("2024-01-01"), Some(0)); // Monday
        assert_eq!(iso_weekday("2000-01-01"), Some(5)); // Saturday
        assert_eq!(iso_weekday("2026-06-17"), Some(2)); // Wednesday
    }
    #[test]
    fn bad_dates_are_none() {
        assert_eq!(iso_weekday("not-a-date"), None);
        assert_eq!(iso_weekday("2026-06"), None);
    }
}
