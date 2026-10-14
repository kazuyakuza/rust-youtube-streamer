//! Run-mode constant contracts (plan §7 / D-T3-7): the two plain-stdout
//! stop messages and the clean-stop exit code are pinned verbatim.

use super::{RUN_STARTUP_STOP_MESSAGE, RUN_STOP_MESSAGE, RUN_SUCCESS_EXIT};

#[test]
fn stop_messages_and_success_exit_are_pinned() {
    assert_eq!(RUN_SUCCESS_EXIT, 0);
    assert_eq!(
        RUN_STOP_MESSAGE,
        "streaming stopped; the broadcast auto-completes on YouTube after ingestion ends"
    );
    assert_eq!(
        RUN_STARTUP_STOP_MESSAGE,
        "run stopped by the stop request before streaming started"
    );
}
