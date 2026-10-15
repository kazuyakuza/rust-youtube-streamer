//! Run-mode constant and watch-link contracts (plan §7 / D-T3-7 /
//! D-T2W-5): the two plain-stdout stop messages, the clean-stop exit
//! code, and the public watch-link line/URL are pinned verbatim.

use super::broadcast::{public_watch_url, WATCH_URL_PREFIX};
use super::gate::WATCH_LINK_MESSAGE;
use super::runtime_test_fixtures::BROADCAST_ID_FIXTURE;
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

#[test]
fn watch_link_message_and_url_are_pinned() {
    assert_eq!(WATCH_LINK_MESSAGE, "public watch link for the live stream");
    assert_eq!(
        public_watch_url(BROADCAST_ID_FIXTURE),
        "https://www.youtube.com/watch?v=broadcast-id-fixture"
    );
}

#[test]
fn derived_watch_url_carries_no_secret_material() {
    let watch_url = public_watch_url(BROADCAST_ID_FIXTURE);
    assert!(watch_url.contains(BROADCAST_ID_FIXTURE));
    assert!(watch_url.contains(WATCH_URL_PREFIX));
    assert!(!watch_url.contains("rtmps"));
    assert!(!watch_url.contains("stream-name-fixture"));
    assert!(!watch_url.contains("bearer"));
    assert!(!watch_url.contains("access-token"));
}
