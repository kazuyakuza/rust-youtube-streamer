//! Stream-health derivation pins (stream-health Task 2): the CBR bufsize
//! doubling rule over the configured bitrate and its typed rejection of a
//! non-numeric bitrate notation.

use super::*;

#[test]
fn cbr_bufsize_doubles_the_configured_bitrate_keeping_the_suffix() {
    assert_eq!(
        cbr_bufsize_bitrate("6000k").expect("6000k must double to 12000k"),
        "12000k"
    );
    assert_eq!(
        cbr_bufsize_bitrate("6M").expect("6M must double to 12M"),
        "12M"
    );
    assert_eq!(
        cbr_bufsize_bitrate("5000000").expect("a suffixless bitrate doubles in bits"),
        "10000000"
    );
}

#[test]
fn cbr_bufsize_non_numeric_bitrate_fails_as_typed_notation_error() {
    let notation =
        cbr_bufsize_bitrate("mega").expect_err("non-numeric bitrate must fail at derivation time");
    assert_eq!(
        notation.to_string(),
        "streaming pipeline CBR bufsize derivation requires a numeric-leading bitrate in configuration (got 'mega')"
    );
}
