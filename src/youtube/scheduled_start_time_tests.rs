//! RFC 3339 derivation tests for the scheduled broadcast start time:
//! exact formatting, the leap-day boundary, and the fixed one-minute
//! offset (plan §5 T4). Pure clock-math tests — no wall clock, no I/O.

use super::*;

#[test]
fn epoch_zero_plus_offset_formats_exact() {
    assert_eq!(format_rfc3339(0), "1970-01-01T00:00:00Z");
}

#[test]
fn known_epoch_matches_rfc3339() {
    // 2024-01-01T00:00:00Z == 1_704_067_200 epoch seconds (hand-computed).
    assert_eq!(format_rfc3339(1_704_067_200), "2024-01-01T00:00:00Z");
    // 2026-10-13T12:34:56Z == 1_791_894_896 epoch seconds (hand-computed:
    // 20_454 days to 2026-01-01, + 285 days to Oct 13, + 45_296 s).
    assert_eq!(format_rfc3339(1_791_894_896), "2026-10-13T12:34:56Z");
}

#[test]
fn leap_day_included_correctly() {
    // 2024-02-29T00:00:00Z == 1_709_164_800 epoch seconds (hand-computed).
    assert_eq!(format_rfc3339(1_709_164_800), "2024-02-29T00:00:00Z");
    // 2024-03-01T00:00:00Z == 1_709_251_200 epoch seconds (the leap day
    // advanced February to 29 days rather than 28).
    assert_eq!(format_rfc3339(1_709_251_200), "2024-03-01T00:00:00Z");
}

#[test]
fn offset_is_exactly_one_minute() {
    let base = 1_704_067_200_u64;
    assert_eq!(scheduled_start_time_from_epoch(base), format_rfc3339(base + 60));
    assert_eq!(scheduled_start_time_from_epoch(0), format_rfc3339(60));
    assert_eq!(scheduled_start_time_from_epoch(0), "1970-01-01T00:01:00Z");
}
