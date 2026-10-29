//! RFC-3339 → epoch-seconds pin (moved verbatim from the retired Phase-06
//! mapping roster): the documented UTC forms parse and every out-of-range
//! or malformed shape is rejected. Offline only — pure function fan-out.

use super::parse_rfc3339_seconds;

#[test]
fn rfc3339_parsing_covers_documented_utc_forms() {
    assert_eq!(
        parse_rfc3339_seconds("2026-10-25T12:30:45Z"),
        Some(1_792_931_445)
    );
    assert_eq!(
        parse_rfc3339_seconds("2026-10-25T12:30:45.123Z"),
        Some(1_792_931_445)
    );
    assert_eq!(parse_rfc3339_seconds("1970-01-01T00:00:00Z"), Some(0));
    assert_eq!(parse_rfc3339_seconds("2026-02-29T00:00:00Z"), None);
    assert_eq!(parse_rfc3339_seconds("2026-04-31T00:00:00Z"), None);
    assert_eq!(parse_rfc3339_seconds("2026-13-01T00:00:00Z"), None);
    assert_eq!(parse_rfc3339_seconds("2026-10-25T24:00:00Z"), None);
    assert_eq!(parse_rfc3339_seconds("2026-10-25T12:30:45"), None);
    assert_eq!(parse_rfc3339_seconds("2026-10-25T12:30:45.1234Z"), None);
}
