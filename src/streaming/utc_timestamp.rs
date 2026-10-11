//! Pure UTC civil date/time formatting for recording filenames
//! (`YYYYMMDD-HHMMSS`). The standard library has no calendar formatting and
//! the dependency policy forbids adding a crate for this, so the conversion
//! is implemented here from the proleptic Gregorian calendar. UTC is a
//! deliberate, tested deviation from wall-clock local time: filenames are
//! identical on every machine and time zone, with no DST surprises and no
//! timezone database dependency.

const EPOCH_YEAR: u64 = 1_970;
const SECONDS_PER_DAY: u64 = 86_400;
const SECONDS_PER_HOUR: u64 = 3_600;
const SECONDS_PER_MINUTE: u64 = 60;
const DAYS_PER_4_YEARS: u64 = 1_461;
const DAYS_PER_400_YEARS: u64 = 146_097;
const NON_LEAP_YEAR_DAYS: u64 = 365;
const FEBRUARY_INDEX: usize = 1;
const LEAP_DAY_COUNT: u32 = 1;
const MONTH_LENGTHS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Formats `epoch_seconds` as the zero-padded UTC civil timestamp
/// `YYYYMMDD-HHMMSS` (proleptic Gregorian); epoch 0 becomes
/// `19700101-000000`.
pub(crate) fn format_utc_timestamp(epoch_seconds: u64) -> String {
    let seconds_of_day = epoch_seconds % SECONDS_PER_DAY;
    let days = epoch_seconds / SECONDS_PER_DAY;
    let (year, month, day) = civil_date_from_epoch_days(days);
    let hour = seconds_of_day / SECONDS_PER_HOUR;
    let minute = (seconds_of_day % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
    let second = seconds_of_day % SECONDS_PER_MINUTE;
    format!("{year:04}{month:02}{day:02}-{hour:02}{minute:02}{second:02}")
}

/// Converts whole days since 1970-01-01 into a `(year, month, day)` triple.
/// Fixed-length 400-year and 4-year blocks skip the bulk arithmetically; the
/// remaining day count is below one 4-year block, so the year walk stays
/// within three iterations.
fn civil_date_from_epoch_days(days: u64) -> (u64, u32, u32) {
    let mut remaining_days = days;
    let mut year = EPOCH_YEAR + 400 * (remaining_days / DAYS_PER_400_YEARS);
    remaining_days %= DAYS_PER_400_YEARS;
    year += 4 * (remaining_days / DAYS_PER_4_YEARS);
    remaining_days %= DAYS_PER_4_YEARS;
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
        NON_LEAP_YEAR_DAYS + 1
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
#[path = "utc_timestamp_tests.rs"]
mod utc_timestamp_tests;
