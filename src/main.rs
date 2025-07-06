// File: src/main.rs
// Description: A flexible timestamp generator and expander.
// Generates a compact timestamp: YY-DOY-BASEMIN.
// Expands a compact timestamp back to a standard format.

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use regex::Regex;

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
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a compact timestamp (default behavior)
    #[command(visible_alias = "gen")]
    Generate(GenerateArgs),
    /// Expand a compact timestamp back to a standard format
    Expand(ExpandArgs),
}

/// Arguments for the `generate` command.
#[derive(Parser, Debug)]
pub struct GenerateArgs {
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
    #[arg(long, verbatim_doc_comment)]
    from: Option<String>,
}

/// Arguments for the `expand` command.
#[derive(Parser, Debug)]
pub struct ExpandArgs {
    /// The string containing the compact timestamp to expand (e.g., "log-25-181-956.txt").
    #[arg(required = true)]
    input_string: String,

    /// The numerical base to assume for the minutes component of the timestamp.
    #[arg(short, long, value_enum, default_value_t = Base::B12)]
    base: Base,

    /// The output format for the expanded timestamp, using chrono specifiers.
    ///
    /// The format cannot include seconds or sub-second precision (e.g., %S, %s, %f).
    #[arg(short, long, default_value = "%Y-%m-%dT%H:%M", verbatim_doc_comment)]
    format: String,
}

/// Defines the available choices for the numerical base.
#[derive(ValueEnum, Clone, Debug, Copy)]
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

/// Converts a string in a given base to a non-negative integer.
fn from_base_n(s: &str, base: u32) -> Result<u32, String> {
    if !(2..=36).contains(&base) {
        panic!("Base must be between 2 and 36.");
    }
    u32::from_str_radix(s, base)
        .map_err(|_| format!("Invalid digit found in '{}' for base {}", s, base))
}

/// Validates that the format string does not request unsupported precision.
fn validate_format_string(format: &str) -> Result<(), String> {
    if format.contains("%S") || format.contains("%s") || format.contains("%f") {
        Err(
            "Output format string cannot contain second or sub-second specifiers (%S, %s, %f)."
                .to_string(),
        )
    } else {
        Ok(())
    }
}

// SECTION 3: COMMAND HANDLERS & MAIN EXECUTION
// ============================================

fn main() {
    // If no subcommand is provided, default to 'generate'.
    // This makes `compact-ts --base b36` work as it did before.
    let args: Vec<String> = std::env::args().collect();
    let first_arg = args.get(1).map(|s| s.as_str());

    let cli = if let Some(arg) = first_arg {
        if !matches!(arg, "generate" | "gen" | "expand" | "-h" | "--help" | "-V" | "--version") {
            let mut new_args = args;
            new_args.insert(1, "generate".to_string());
            Cli::parse_from(new_args)
        } else {
            Cli::parse()
        }
    } else {
        // No args, default to `generate` which will print current time.
        let mut new_args = args;
        new_args.insert(1, "generate".to_string());
        Cli::parse_from(new_args)
    };

    let result = match cli.command {
        Commands::Generate(args) => handle_generate_command(args),
        Commands::Expand(args) => handle_expand_command(args),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Handles the logic for the `generate` subcommand.
fn handle_generate_command(args: GenerateArgs) -> Result<(), String> {
    // Determine the source datetime: either from the --from flag or the current time.
    let source_dt = match args.from {
        Some(from_str) => parse_flexible_timestamp(&from_str)?,
        None => Local::now(),
    };

    // Extract components from the source datetime.
    let yy = source_dt.format("%y").to_string();
    let doy = source_dt.format("%j").to_string();
    let minutes_since_midnight = source_dt.time().num_seconds_from_midnight() / 60;

    // Determine the numerical base from the parsed arguments.
    let base_num = match args.base {
        Base::B12 => 12,
        Base::B36 => 36,
    };

    // Convert minutes to the chosen base string.
    let base_min_str = to_base_n(minutes_since_midnight, base_num);

    // Print the final formatted timestamp, padding the base part to 3 characters.
    println!("{}-{}-{:0>3}", yy, doy, base_min_str);
    Ok(())
}

/// Handles the logic for the `expand` subcommand.
fn handle_expand_command(args: ExpandArgs) -> Result<(), String> {
    validate_format_string(&args.format)?;

    let (base_num, re_pattern) = match args.base {
        Base::B12 => (12, r"(\d{2})-(\d{3})-([0-9A-B]{3})"),
        Base::B36 => (36, r"(\d{2})-(\d{3})-([0-9A-Za-z]{3})"),
    };

    let re = Regex::new(re_pattern).unwrap(); // Pattern is static and valid.
    let caps = re.captures(&args.input_string).ok_or_else(|| {
        format!(
            "No compact timestamp found in \"{}\".",
            args.input_string
        )
    })?;

    let yy_str = &caps[1];
    let doy_str = &caps[2];
    let basemin_str = &caps[3];

    // Decode components
    let year = 2000 + yy_str.parse::<i32>().unwrap(); // Regex ensures it's \d{2}
    let doy = doy_str.parse::<u32>().unwrap(); // Regex ensures it's \d{3}

    let minutes_since_midnight = from_base_n(basemin_str, base_num)?;

    // Validate components
    if minutes_since_midnight >= 1440 {
        return Err(format!(
            "Invalid minutes value '{}' ({} decimal), must be less than 1440.",
            basemin_str, minutes_since_midnight
        ));
    }

    // Construct NaiveDateTime
    let date = NaiveDate::from_yo_opt(year, doy)
        .ok_or_else(|| format!("Invalid day of year: {}.", doy))?;
    let naive_dt = date
        .and_hms_opt(
            minutes_since_midnight / 60,
            minutes_since_midnight % 60,
            0,
        )
        .unwrap(); // Will not panic as minutes are < 1440

    // Convert to local time
    let local_dt = Local
        .from_local_datetime(&naive_dt)
        .single()
        .ok_or_else(|| "Ambiguous local time".to_string())?;

    // Format and print
    println!("{}", local_dt.format(&args.format));

    Ok(())
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
    fn test_base_conversion_to_base() {
        // 22*60 + 42 = 1362 minutes
        assert_eq!(to_base_n(1362, 12), "956");
        assert_eq!(to_base_n(1362, 36), "11U");
        // 20*60 + 48 = 1248 minutes
        assert_eq!(to_base_n(1248, 12), "880");
        assert_eq!(to_base_n(1248, 36), "YO");
    }

    // --- Tests for `expand` logic ---

    #[test]
    fn test_base_conversion_from_base() {
        assert_eq!(from_base_n("956", 12).unwrap(), 1362);
        assert_eq!(from_base_n("11U", 36).unwrap(), 1362);
        assert_eq!(from_base_n("880", 12).unwrap(), 1248);
        assert_eq!(from_base_n("YO", 36).unwrap(), 1248);
        assert_eq!(from_base_n("yo", 36).unwrap(), 1248); // case-insensitive
    }

    #[test]
    fn test_base_conversion_from_base_invalid() {
        assert!(from_base_n("95C", 12).is_err()); // C is not in base 12
        assert!(from_base_n("11$", 36).is_err()); // $ is not in base 36
    }

    #[test]
    fn test_validate_format_string() {
        assert!(validate_format_string("%Y-%m-%d %H:%M").is_ok());
        assert!(validate_format_string("%A, %B %d").is_ok());
        assert!(validate_format_string("hello world").is_ok()); // no specifiers is ok
    }

    #[test]
    fn test_validate_format_string_invalid() {
        assert!(validate_format_string("%Y-%m-%d %H:%M:%S").is_err()); // has %S
        assert!(validate_format_string("%Y-%m-%d %H:%M:%S.%f").is_err()); // has %f
        assert!(validate_format_string("%s").is_err()); // has %s
    }

    // A testable core function for the expansion logic.
    fn expand_core(input: &str, base: Base) -> Result<DateTime<Local>, String> {
        let (base_num, re_pattern) = match base {
            Base::B12 => (12, r"(\d{2})-(\d{3})-([0-9A-B]{3})"),
            Base::B36 => (36, r"(\d{2})-(\d{3})-([0-9A-Za-z]{3})"),
        };
        let re = Regex::new(re_pattern).unwrap();
        let caps = re.captures(input).ok_or_else(|| "no match".to_string())?;

        let year = 2000 + caps[1].parse::<i32>().unwrap();
        let doy = caps[2].parse::<u32>().unwrap();
        let basemin_str = &caps[3];
        let minutes_since_midnight = from_base_n(basemin_str, base_num)?;

        if minutes_since_midnight >= 1440 {
            return Err("invalid minutes".to_string());
        }

        let date = NaiveDate::from_yo_opt(year, doy).ok_or_else(|| "invalid doy".to_string())?;
        let naive_dt = date
            .and_hms_opt(
                minutes_since_midnight / 60,
                minutes_since_midnight % 60,
                0,
            )
            .unwrap();
        Local
            .from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| "ambiguous".to_string())
    }

    #[test]
    fn test_expand_core_logic_b12() {
        // 2025-06-30 is day 181. 22:42 is 1362 minutes, which is 956 in base 12.
        let result = expand_core("25-181-956", Base::B12).unwrap();
        assert_eq!(result.year(), 2025);
        assert_eq!(result.month(), 6);
        assert_eq!(result.day(), 30);
        assert_eq!(result.hour(), 22);
        assert_eq!(result.minute(), 42);
    }

    #[test]
    fn test_expand_core_logic_b36() {
        // 2025-06-30 is day 181. 22:42 is 1362 minutes, which is 11U in base 36.
        let result = expand_core("prefix-25-181-11U-suffix", Base::B36).unwrap();
        assert_eq!(result.year(), 2025);
        assert_eq!(result.month(), 6);
        assert_eq!(result.day(), 30);
        assert_eq!(result.hour(), 22);
        assert_eq!(result.minute(), 42);
    }

    #[test]
    fn test_expand_core_invalid_doy() {
        // 2025 is not a leap year, so 366 is invalid.
        assert!(expand_core("25-366-000", Base::B12).is_err());
        // 2024 is a leap year.
        assert!(expand_core("24-366-000", Base::B12).is_ok());
    }

    #[test]
    fn test_expand_core_invalid_minutes() {
        // AAA in base 12 is 10*144 + 10*12 + 10 = 1570, which is > 1439.
        let result = expand_core("25-181-AAA", Base::B12);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "invalid minutes");
    }
}
