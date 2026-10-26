//! The RFC-3339 → epoch-seconds helper of the Phase-06 chat mapping
//! (record D-4/D-11, plan §3.7; caller-authorized sibling split): the
//! hand-rolled reverse of the existing civil-from-days family — documented
//! ISO 8601 UTC forms `YYYY-MM-DDThh:mm:ssZ` and `YYYY-MM-DDThh:mm:ss.fffZ`
//! only; `None` for a missing `Z` terminator, wrong component widths,
//! out-of-range fields, a non-leap 02-29, or more than 3 fractional
//! digits; never panics, no `.expect()`, NO time crate.
//!
//! Grounding: S1 `snippet.publishedAt` (citation + access date in
//! `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` §1).
//! Registered from `live_chat_mapping` via `#[path]`; offline tests only.

const YEAR_DIGITS: usize = 4;
const PAIR_DIGITS: usize = 2;
const MAX_FRACTION_DIGITS: usize = 3;
const MAX_HOUR: u32 = 23;
const MAX_MINUTE_SECOND: u32 = 59;
const EPOCH_CIVIL_YEAR: u32 = 1_970;
const DAYS_PER_YEAR: i64 = 365;
const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3_600;
const SECONDS_PER_DAY: u64 = 86_400;
const MONTH_START_DAYS: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

pub(crate) fn parse_rfc3339_seconds(value: &str) -> Option<u64> {
    let (date, time) = split_utc_date_time(value)?;
    let clock = split_validated_fraction(time)?;
    let (year, month, day) = parse_year_month_day(date)?;
    let (hour, minute, second) = parse_hour_minute_second(clock)?;
    let days = validated_epoch_days(year, month, day)?;
    epoch_seconds(days, hour, minute, second)
}

fn split_utc_date_time(value: &str) -> Option<(&str, &str)> {
    let utc = value.strip_suffix('Z')?;
    utc.split_once('T')
}

/// Keeps the clock part only when the optional fraction is 1–3 digits.
fn split_validated_fraction(time: &str) -> Option<&str> {
    let Some((clock, fraction)) = time.split_once('.') else {
        return Some(time);
    };
    let fraction_is_valid = !fraction.is_empty()
        && fraction.len() <= MAX_FRACTION_DIGITS
        && fraction.bytes().all(|byte| byte.is_ascii_digit());
    fraction_is_valid.then_some(clock)
}

fn parse_year_month_day(date: &str) -> Option<(u32, u32, u32)> {
    let mut segments = date.split('-');
    let year = parse_fixed_width(segments.next()?, YEAR_DIGITS)?;
    let month = parse_fixed_width(segments.next()?, PAIR_DIGITS)?;
    let day = parse_fixed_width(segments.next()?, PAIR_DIGITS)?;
    segments.next().is_none().then_some((year, month, day))
}

fn parse_hour_minute_second(clock: &str) -> Option<(u32, u32, u32)> {
    let mut segments = clock.split(':');
    let hour = parse_fixed_width(segments.next()?, PAIR_DIGITS)?;
    let minute = parse_fixed_width(segments.next()?, PAIR_DIGITS)?;
    let second = parse_fixed_width(segments.next()?, PAIR_DIGITS)?;
    if segments.next().is_some() {
        return None;
    }
    Some((
        in_range(hour, MAX_HOUR)?,
        in_range(minute, MAX_MINUTE_SECOND)?,
        in_range(second, MAX_MINUTE_SECOND)?,
    ))
}

fn parse_fixed_width(segment: &str, digits: usize) -> Option<u32> {
    if segment.len() != digits {
        return None;
    }
    if !segment.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    segment.parse::<u32>().ok()
}

fn in_range(value: u32, maximum: u32) -> Option<u32> {
    (value <= maximum).then_some(value)
}

/// Calendar validation (month/day ranges, leap February) → epoch days.
fn validated_epoch_days(year: u32, month: u32, day: u32) -> Option<i64> {
    if !is_valid_month(month) {
        return None;
    }
    if !(1..=days_in_month(year, month)).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day))
}

fn is_valid_month(month: u32) -> bool {
    (1..=12).contains(&month)
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: u32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Days since 1970-01-01 for a validated civil date (proleptic Gregorian;
/// negative before the epoch — the u64 narrowing below yields `None`).
fn days_from_civil(year: u32, month: u32, day: u32) -> i64 {
    let years_since_epoch = i64::from(year) - i64::from(EPOCH_CIVIL_YEAR);
    let leap_days = leap_days_before_year(year) - leap_days_before_year(EPOCH_CIVIL_YEAR);
    let day_of_year = month_start_day(year, month) + i64::from(day) - 1;
    years_since_epoch * DAYS_PER_YEAR + leap_days + day_of_year
}

/// Leap days strictly before Jan 1 of `year` (all inputs ≥ 1 → floor-safe).
fn leap_days_before_year(year: u32) -> i64 {
    let prior_year = i64::from(year) - 1;
    prior_year / 4 - prior_year / 100 + prior_year / 400
}

fn month_start_day(year: u32, month: u32) -> i64 {
    let mut start = MONTH_START_DAYS[(month - 1) as usize];
    if february_offset_applies(month, year) {
        start += 1;
    }
    start
}

fn february_offset_applies(month: u32, year: u32) -> bool {
    month > 2 && is_leap_year(year)
}

fn epoch_seconds(days: i64, hour: u32, minute: u32, second: u32) -> Option<u64> {
    let day_seconds = u64::try_from(days).ok()?.checked_mul(SECONDS_PER_DAY)?;
    let clock_seconds = u64::from(hour) * SECONDS_PER_HOUR
        + u64::from(minute) * SECONDS_PER_MINUTE
        + u64::from(second);
    day_seconds.checked_add(clock_seconds)
}
