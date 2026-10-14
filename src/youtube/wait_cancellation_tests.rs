//! Cancellation-seam tests (plan §4 roster E): the clone-shared flag
//! semantics and the cleared initial state. Pure fakes; no network.

use super::*;

#[test]
fn clone_shares_one_flag() {
    let original = WaitCanceller::new();
    let clone = original.clone();
    clone.request_cancel();
    assert!(original.is_cancelled());
    assert!(clone.is_cancelled());
}

#[test]
fn fresh_canceller_is_not_cancelled() {
    let canceller = WaitCanceller::new();
    assert!(!canceller.is_cancelled());
}
