// File: src/main.rs
// Description: A flexible timestamp generator.
// Prints a compact timestamp: YY-DOY-BASEMIN with a configurable base
// and can convert from a standard timestamp format.

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};
use clap::{Parser, ValueEnum};

// SECTION 1: COMMAND-LINE INTERFACE SETUP
// ========================================

/// A utility to print a compact timestamp: YY-DOY-BASEMIN
///
/// The base for the minutes-since-midnight component can be configured.
/// It defaults to base-12 for optimal 3-character space utilization.
/// The timestamp can be generated from the current time or a specified time string.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The numerical base to use for encoding the minutes since midnight.
    #[arg(short, long, value_enum, default_value_t = Base::B12)]
    base: Base,

    /// Convert a specific timestamp instead of using the current time.
    ///
    /// Tries to parse multiple formats in order of specificity.
    /// Timestamps with offsets are converted to local time.
    /// Naive timestamps are interpreted in the local timezone.
    /// Date-only inputs default to midnight.
    ///
    /// Examples:
    /// - 2025-06-30T22:42:05Z      (RFC 3339)
    /// - 2025-06-28T20:28+02:00    (ISO 8601 with offset, no seconds)
    /// - 20250628T20:28+02:00      (Compact date with offset)
    /// - 20250630T224205Z          (Compact ISO 8601)
    /// - 2025-06-30T22:42:05       (Naive full)
    /// - 2025-06-30T22:42          (Naive no seconds)
    /// - 20250630T2242             (Fully compact)
    /// - 2025-06-30                (Date only)
    /// - 20250630                  (Date only compact)
    #[arg(long)]
    from: Option<String>,
}

/// Defines the available choices for the numerical base.
#[derive(ValueEnum, Clone, Debug)]
enum Base {
    /// Base-12 (Duodecimal): 0-9, A-B. Optimal for 3 chars (max value 9BB).
    B12,
    /// Base-36: 0-9, A-Z. Inefficient for 3 chars (max value 13Z).
    B36,
}

// SECTION 2: CORE LOGIC
// =====================

/// Smartly parses a string into a local DateTime object.
///
/// It tries a series of common timestamp formats in order of specificity.
/// If a date-only format is matched, the time is assumed to be midnight.
/// All parsed times are converted to the system's local timezone.
fn parse_flexible_timestamp(s: &str) -> Result<DateTime<Local>, String> {
    // Helper to convert a NaiveDateTime to a local DateTime, handling ambiguity.
    let to_local = |ndt: NaiveDateTime| {
        Local
            .from_local_datetime(&ndt)
            .single()
            .ok_or_else(|| "Ambiguous local time".to_string())
    };

    // Attempt 1: Full ISO 8601 / RFC 3339 (e.g., 2025-06-30T22:42:05Z)
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Local));
    }

    // Attempt 2: ISO 8601 with offset, no seconds (e.g., 2025-06-28T20:28+02:00)
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M%z") {
        return Ok(dt.with_timezone(&Local));
    }

    // Attempt 3: Compact date, time with colon, offset, no seconds (e.g., 20250628T20:28+02:00)
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y%m%dT%H:%M%z") {
        return Ok(dt.with_timezone(&Local));
    }

    // Attempt 4: ISO 8601 compact with Z (e.g., 20250630T224205Z)
    // Note: The 'Z' is a literal, not a timezone name for `%Z`.
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ") {
        return Ok(Utc.from_utc_datetime(&dt).with_timezone(&Local));
    }

    // Attempt 5: Naive date and time with seconds (e.g., 2025-06-30T22:42:05)
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return to_local(ndt);
    }

    // Attempt 6: Naive date and time, no seconds, with colon (e.g., 2025-06-30T22:42)
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M") {
        return to_local(ndt);
    }

    // Attempt 7: Naive date and time, no seconds, compact time (e.g., 2025-06-30T2242)
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H%M") {
        return to_local(ndt);
    }

    // Attempt 8: Compact date, naive time, no seconds, with colon (e.g., 20250630T22:42)
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y%m%dT%H:%M") {
        return to_local(ndt);
    }

    // Attempt 9: Fully compact date and time, no seconds (e.g., 20250630T2242)
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M") {
        return to_local(ndt);
    }

    // Attempt 10: Date-only with hyphens (e.g., 2025-06-30)
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = date.and_hms_opt(0, 0, 0).unwrap();
        return to_local(dt);
    }

    // Attempt 11: Date-only compact (e.g., 20250630)
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y%m%d") {
        let dt = date.and_hms_opt(0, 0, 0).unwrap();
        return to_local(dt);
    }

    Err(format!("Could not parse '{}' as a valid timestamp.", s))
}

/// Converts a non-negative integer to a string in the specified base.
fn to_base_n(mut n: u32, base: u32) -> String {
    if !(2..=36).contains(&base) {
        panic!("Base must be between 2 and 36.");
    }
    if n == 0 {
        return "0".to_string();
    }

    const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut result = String::new();

    while n > 0 {
        result.push(CHARSET[(n % base) as usize] as char);
        n /= base;
    }

    result.chars().rev().collect()
}

// SECTION 3: MAIN EXECUTION
// =========================

fn main() {
    let cli = Cli::parse();

    // Determine the source datetime: either from the --from flag or the current time.
    let source_dt = match cli.from {
        Some(from_str) => match parse_flexible_timestamp(&from_str) {
            Ok(dt) => dt,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        None => Local::now(),
    };

    // Extract components from the source datetime.
    let yy = source_dt.format("%y").to_string();
    let doy = source_dt.format("%j").to_string();
    let minutes_since_midnight = source_dt.time().num_seconds_from_midnight() / 60;

    // Determine the numerical base from the parsed arguments.
    let base_num = match cli.base {
        Base::B12 => 12,
        Base::B36 => 36,
    };

    // Convert minutes to the chosen base string.
    let base_min_str = to_base_n(minutes_since_midnight, base_num);

    // Print the final formatted timestamp, padding the base part to 3 characters.
    println!("{}-{}-{:0>3}", yy, doy, base_min_str);
}

// SECTION 4: UNIT TESTS
// =====================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    // Helper to create a UTC DateTime for consistent test assertions.
    fn make_utc_dt(year: i32, month: u32, day: u32, h: u32, m: u32, s: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, h, m, s).unwrap()
    }

    #[test]
    fn test_parse_full_iso_with_hyphens() {
        let input = "2025-06-30T22:42:05Z";
        let expected = make_utc_dt(2025, 6, 30, 22, 42, 5);
        let parsed = parse_flexible_timestamp(input).unwrap();
        assert_eq!(parsed.with_timezone(&Utc), expected);
    }

    #[test]
    fn test_parse_full_iso_with_offset() {
        let input = "2025-07-01T04:12:05+05:30"; // India Standard Time
        let expected = make_utc_dt(2025, 6, 30, 22, 42, 5);
        let parsed = parse_flexible_timestamp(input).unwrap();
        assert_eq!(parsed.with_timezone(&Utc), expected);
    }

    #[test]
    fn test_parse_compact_iso_with_z() {
        // Note: RFC3339 parser does NOT handle this compact form.
        let input = "20250630T224205Z";
        let expected = make_utc_dt(2025, 6, 30, 22, 42, 5);
        let parsed = parse_flexible_timestamp(input).unwrap();
        assert_eq!(parsed.with_timezone(&Utc), expected);
    }

    #[test]
    fn test_parse_date_only_with_hyphens() {
        let input = "2025-06-30";
        let parsed = parse_flexible_timestamp(input).unwrap();
        assert_eq!(parsed.year(), 2025);
        assert_eq!(parsed.month(), 6);
        assert_eq!(parsed.day(), 30);
        assert_eq!(parsed.hour(), 0);
        assert_eq!(parsed.minute(), 0);
    }

    #[test]
    fn test_parse_date_only_compact() {
        let input = "20250630";
        let parsed = parse_flexible_timestamp(input).unwrap();
        assert_eq!(parsed.year(), 2025);
        assert_eq!(parsed.month(), 6);
        assert_eq!(parsed.day(), 30);
        assert_eq!(parsed.hour(), 0);
        assert_eq!(parsed.minute(), 0);
    }

    #[test]
    fn test_parse_naive_datetime() {
        let input = "2025-06-30T22:42:05";
        let parsed = parse_flexible_timestamp(input).unwrap();
        assert_eq!(parsed.year(), 2025);
        assert_eq!(parsed.month(), 6);
        assert_eq!(parsed.day(), 30);
        assert_eq!(parsed.hour(), 22);
        assert_eq!(parsed.minute(), 42);
    }

    #[test]
    fn test_parse_invalid_string() {
        let input = "not-a-real-date";
        assert!(parse_flexible_timestamp(input).is_err());
    }

    #[test]
    fn test_base_conversion() {
        // 22*60 + 42 = 1362 minutes
        assert_eq!(to_base_n(1362, 12), "956");
        assert_eq!(to_base_n(1362, 36), "11U");
        // 20*60 + 48 = 1248 minutes
        assert_eq!(to_base_n(1248, 12), "880");
        assert_eq!(to_base_n(1248, 36), "YO");
    }

    #[test]
    fn test_parse_naive_datetime_no_seconds_colon() {
        let input = "2025-06-30T22:42";
        let parsed = parse_flexible_timestamp(input).unwrap();
        assert_eq!(parsed.year(), 2025);
        assert_eq!(parsed.month(), 6);
        assert_eq!(parsed.day(), 30);
        assert_eq!(parsed.hour(), 22);
        assert_eq!(parsed.minute(), 42);
        assert_eq!(parsed.second(), 0);
    }

    #[test]
    fn test_parse_naive_datetime_no_seconds_compact_time() {
        let input = "2025-06-30T2242";
        let parsed = parse_flexible_timestamp(input).unwrap();
        assert_eq!(parsed.year(), 2025);
        assert_eq!(parsed.month(), 6);
        assert_eq!(parsed.day(), 30);
        assert_eq!(parsed.hour(), 22);
        assert_eq!(parsed.minute(), 42);
        assert_eq!(parsed.second(), 0);
    }

    #[test]
    fn test_parse_compact_date_naive_time_no_seconds_colon() {
        let input = "20250630T22:42";
        let parsed = parse_flexible_timestamp(input).unwrap();
        assert_eq!(parsed.year(), 2025);
        assert_eq!(parsed.month(), 6);
        assert_eq!(parsed.day(), 30);
        assert_eq!(parsed.hour(), 22);
        assert_eq!(parsed.minute(), 42);
        assert_eq!(parsed.second(), 0);
    }

    #[test]
    fn test_parse_fully_compact_no_seconds() {
        let input = "20250630T2242";
        let parsed = parse_flexible_timestamp(input).unwrap();
        assert_eq!(parsed.year(), 2025);
        assert_eq!(parsed.month(), 6);
        assert_eq!(parsed.day(), 30);
        assert_eq!(parsed.hour(), 22);
        assert_eq!(parsed.minute(), 42);
        assert_eq!(parsed.second(), 0);
    }

    #[test]
    fn test_parse_iso_no_seconds_with_offset() {
        let input = "2025-06-28T20:28+02:00";
        let expected = make_utc_dt(2025, 6, 28, 18, 28, 0);
        let parsed = parse_flexible_timestamp(input).unwrap();
        assert_eq!(parsed.with_timezone(&Utc), expected);
    }

    #[test]
    fn test_parse_compact_date_no_seconds_with_offset() {
        let input = "20250628T20:28+02:00";
        let expected = make_utc_dt(2025, 6, 28, 18, 28, 0);
        let parsed = parse_flexible_timestamp(input).unwrap();
        assert_eq!(parsed.with_timezone(&Utc), expected);
    }
}
