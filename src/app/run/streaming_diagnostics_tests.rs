//! Run-shutdown streaming diagnostics tests (run-diagnostics Task 1):
//! the message constants are the single source of truth for the two
//! shutdown evidence lines, pinned verbatim, plus the pure decision
//! helpers behind them. Offline only — no process, no Google.

use super::*;

#[test]
fn ffmpeg_exit_messages_are_pinned() {
    assert_eq!(
        FFMPEG_CLEAN_EXIT_MESSAGE,
        "FFmpeg streaming process exited cleanly (exit status 0)"
    );
    assert_eq!(
        FFMPEG_EXIT_FAILURE_MESSAGE,
        "FFmpeg streaming process exited with a failure status"
    );
    assert_eq!(FFMPEG_NO_DIAGNOSTICS_NOTE, "none captured");
}

#[test]
fn stream_exit_failed_classifies_zero_status_and_signal_death() {
    assert!(!stream_exit_failed(Some(0)));
    assert!(stream_exit_failed(Some(1)));
    assert!(stream_exit_failed(None));
}

#[test]
fn an_empty_tail_reads_the_none_captured_note() {
    assert_eq!(diagnostics_text(""), FFMPEG_NO_DIAGNOSTICS_NOTE);
    assert_eq!(diagnostics_text("ERROR"), "ERROR");
}
