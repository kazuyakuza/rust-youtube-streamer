//! Dedup-window pins (plan §5.2): unseen ids answer `true` exactly once,
//! the window evicts the OLDEST id beyond `CHAT_DEDUP_WINDOW` (512), and
//! re-seen ids answer `false` (the D-5 re-use tolerance contract — the
//! window is a pure id-store; skip-policy ordering is Task 3's flow).
//! Offline only.

use super::*;

#[test]
fn dedup_window_reports_unseen_ids_then_repeated_ids_as_seen() {
    let mut window = ChatDedupWindow::new();
    assert!(window.record("msg-1"), "the first sighting is unseen");
    assert!(!window.record("msg-1"), "the repeat sighting is seen");
    assert!(window.record("msg-2"), "a fresh id is unseen");
}

#[test]
fn dedup_window_evicts_the_oldest_id_beyond_512() {
    let mut window = ChatDedupWindow::new();
    for index in 0..=CHAT_DEDUP_WINDOW {
        assert!(window.record(&format!("id-{index}")));
    }
    assert!(
        window.record("id-0"),
        "the oldest id was evicted past the 512 window"
    );
    assert!(
        !window.record("id-512"),
        "the newest id is still remembered"
    );
}

#[test]
fn dedup_window_tolerates_re_seen_ids_of_skipped_types() {
    let mut window = ChatDedupWindow::new();
    assert!(window.record("x"));
    assert!(!window.record("x"));
}
