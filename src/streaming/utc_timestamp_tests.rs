//! UTC timestamp anchor tests: the civil conversion must reproduce the
//! documented epoch instants exactly, including minute rollover and the
//! leap-day civil conversion.

use super::*;

#[test]
fn epoch_zero_formats_as_the_1970_anchor() {
    assert_eq!(format_utc_timestamp(0), "19700101-000000");
}

#[test]
fn billion_second_milestone_is_2001_09_09() {
    assert_eq!(format_utc_timestamp(1_000_000_000), "20010909-014640");
}

#[test]
fn late_2020_epoch_formats_as_2020_09_13() {
    assert_eq!(format_utc_timestamp(1_600_000_000), "20200913-122640");
}

#[test]
fn minute_rollover_pads_the_seconds_field() {
    assert_eq!(format_utc_timestamp(59), "19700101-000059");
    assert_eq!(format_utc_timestamp(60), "19700101-000100");
}

#[test]
fn leap_day_tests_the_civil_conversion() {
    // Verified: 946_684_800 = 2000-01-01T00:00:00Z
    // (https://unixepoch.net/ reference table), plus 59 days to
    // 2000-02-29 (31 days of January + 28 of February): 946_684_800 +
    // 59 * 86_400 = 951_782_400. The plan draft's 951_868_800 is one day
    // later (2000-03-01); the plan mandates this online verification.
    assert_eq!(format_utc_timestamp(951_782_400), "20000229-000000");
}
