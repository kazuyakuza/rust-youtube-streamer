//! RFC 3339 UTC timestamp derivation for `snippet.scheduledStartTime`
//! (plan F3, D-T1-5). `liveBroadcasts.insert` REQUIRES a future
//! `scheduledStartTime`, and with `enableAutoStart = true` the broadcast
//! still goes live the moment video flows regardless of the scheduled
//! time, so the runtime pins it to `now + 1 minute`. The standard library
//! has no calendar formatting and the dependency policy forbids a clock
//! crate, so the civil-from-days math is duplicated here from the sealed
//! `streaming::utc_timestamp` module (streaming is boundary-sealed
//! against youtube; a small pure duplicate is cheaper than a
//! cross-module contract).
//!
//! Grounding: `liveBroadcasts.insert` requires a future
//! `snippet.scheduledStartTime` (RFC 3339, e.g. `2013-01-15T01:29:48Z`)
//! per https://developers.google.com/youtube/v3/live/docs/liveBroadcasts;
//! with `contentDetails.enableAutoStart=true` the broadcast goes live the
//! moment video flows regardless of the scheduled time per
//! https://developers.google.com/youtube/v3/live/life-of-a-broadcast, so
//! the one-minute pin never delays the stream.
//!
//! Staging: dead until the runtime builds a `CreateBroadcastCommand` —
//! Task 2's request assembly or Task 3's `run` wiring supplies the clock.
//!
//! Design plan:
//! `.kilo/plans/20261013-phase05-task1-official-api-research-design.md`
//! (§1.3, D-T1-5, F3); TODO `.agent/todos/20261013/20261013-todo-1.md`
//! Task 1.

/// One minute, the fixed offset from "now" to the scheduled start.
pub(crate) const SCHEDULED_TIME_OFFSET_SECONDS: u64 = 60;

const SECONDS_PER_DAY: u64 = 86_400;
const SECONDS_PER_HOUR: u64 = 3_600;
const SECONDS_PER_MINUTE: u64 = 60;
const DAYS_PER_400_YEARS: u64 = 146_097;
const NON_LEAP_YEAR_DAYS: u64 = 365;
const LEAP_YEAR_DAYS: u64 = 366;
const EPOCH_YEAR: u64 = 1_970;
const FEBRUARY_INDEX: usize = 1;
const LEAP_DAY_COUNT: u32 = 1;
const MONTH_LENGTHS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// RFC 3339 UTC timestamp for `liveBroadcasts.insert`'s required
/// `snippet.scheduledStartTime`: the supplied clock moment plus the fixed
/// one-minute offset. The epoch addition saturates at `u64::MAX` — no
/// panic, no clock crate.
pub(crate) fn scheduled_start_time_from_epoch(now_epoch_secs: u64) -> String {
    format_rfc3339(now_epoch_secs.saturating_add(SCHEDULED_TIME_OFFSET_SECONDS))
}

/// Pure `u64`-epoch → `"YYYY-MM-DDThh:mm:ssZ"` formatter (civil-from-days
/// and era math, proleptic Gregorian; epoch 0 becomes
/// `"1970-01-01T00:00:00Z"`).
fn format_rfc3339(epoch_secs: u64) -> String {
    let seconds_of_day = epoch_secs % SECONDS_PER_DAY;
    let (year, month, day) = civil_date_from_epoch_days(epoch_secs / SECONDS_PER_DAY);
    let hour = seconds_of_day / SECONDS_PER_HOUR;
    let minute = (seconds_of_day % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
    let second = seconds_of_day % SECONDS_PER_MINUTE;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Converts whole days since 1970-01-01 into a `(year, month, day)` triple.
/// A fixed-length 400-year block skips the bulk arithmetically; the residual
/// day count is then below one 400-year block, so the remaining years are
/// stepped one at a time until the day-of-year is located.
fn civil_date_from_epoch_days(days: u64) -> (u64, u32, u32) {
    let mut remaining_days = days;
    let mut year = EPOCH_YEAR + 400 * (remaining_days / DAYS_PER_400_YEARS);
    remaining_days %= DAYS_PER_400_YEARS;
    year = skip_whole_years(year, &mut remaining_days);
    let (month, day) = month_and_day(year, remaining_days);
    (year, month, day)
}

fn skip_whole_years(mut year: u64, remaining_days: &mut u64) -> u64 {
    while *remaining_days >= days_in_year(year) {
        *remaining_days -= days_in_year(year);
        year += 1;
    }
    year
}

fn days_in_year(year: u64) -> u64 {
    if is_leap_year(year) {
        LEAP_YEAR_DAYS
    } else {
        NON_LEAP_YEAR_DAYS
    }
}

/// Walks the month table (leap-February adjusted) to convert a zero-based
/// day-of-year into a one-based `(month, day)` pair.
fn month_and_day(year: u64, day_of_year: u64) -> (u32, u32) {
    let month_lengths = month_lengths(year);
    let mut remaining_days = day_of_year;
    let mut month_index = 0;
    while remaining_days >= u64::from(month_lengths[month_index]) {
        remaining_days -= u64::from(month_lengths[month_index]);
        month_index += 1;
    }
    (month_index as u32 + 1, remaining_days as u32 + 1)
}

fn month_lengths(year: u64) -> [u32; 12] {
    if is_leap_year(year) {
        let mut lengths = MONTH_LENGTHS;
        lengths[FEBRUARY_INDEX] += LEAP_DAY_COUNT;
        return lengths;
    }
    MONTH_LENGTHS
}

fn is_leap_year(year: u64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[cfg(test)]
#[path = "scheduled_start_time_tests.rs"]
mod scheduled_start_time_tests;
