//! O6 live-chat-id resolution (record D-2, plan §3.6): poll the
//! documented `snippet.liveChatId` of the created broadcast through the
//! chat seam within the bounded budget `CHAT_ID_RESOLUTION_ATTEMPTS` ×
//! `CHAT_ID_RESOLUTION_INTERVAL_MS` (6 × 5 000 ms = 30 s worst case).
//! `Ok(None)` and transient classes (Network/BackendUnavailable/
//! MalformedResponse) consume an attempt; every other classified error
//! aborts immediately; absence after the budget is a typed fatal
//! `ApiNotFound { resource: "live chat" }` whose Display line carries the
//! actionable diagnostic. The live chat id is a NON-secret operator
//! diagnostic (record D-13) but no logging exists in this task at all.
//!
//! Grounding: S5 (`snippet.liveChatId`) — citations + access dates in
//! `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` §1.
//! Offline tests only — scripted seam, zero network.

use super::api_chat_transport::LiveChatTransport;
use super::live_error::{LiveApiError, LIVE_CHAT_RESOURCE};

const CHAT_ID_RESOLUTION_ATTEMPTS: u64 = 6;
const CHAT_ID_RESOLUTION_INTERVAL_MS: u64 = 5_000;

/// Production entry (D-2): bounded resolution at the fixed 5 s interval.
pub(crate) fn resolve_live_chat_id(
    transport: &dyn LiveChatTransport,
    broadcast_id: &str,
) -> Result<String, LiveApiError> {
    resolve_live_chat_id_with_interval(transport, broadcast_id, CHAT_ID_RESOLUTION_INTERVAL_MS)
}

/// The cfg-split core (record D-I zero-sleep precedent): test builds pass
/// a caller-chosen interval; production keeps the 5 000 ms default.
/// Signature = 3 args — the sanctioned readiness-constructor exception
/// family (record D-T3-15); no canceller parameter (Task 3 owns stop
/// wiring; the run-level flag handling is Task 3's stage 11).
fn resolve_live_chat_id_with_interval(
    transport: &dyn LiveChatTransport,
    broadcast_id: &str,
    interval_ms: u64,
) -> Result<String, LiveApiError> {
    for _attempt in 1..=CHAT_ID_RESOLUTION_ATTEMPTS {
        match transport.lookup_broadcast_chat_id(broadcast_id) {
            Ok(Some(live_chat_id)) => return Ok(live_chat_id),
            Ok(None) => sleep_before_next_lookup(interval_ms),
            Err(error) if is_transient_error(&error) => sleep_before_next_lookup(interval_ms),
            Err(error) => return Err(error),
        }
    }
    Err(LiveApiError::ApiNotFound {
        resource: LIVE_CHAT_RESOURCE.to_string(),
    })
}

/// The D-2/O6 transient class: consumes an attempt, never aborts.
fn is_transient_error(error: &LiveApiError) -> bool {
    matches!(
        error,
        LiveApiError::Network { .. }
            | LiveApiError::BackendUnavailable { .. }
            | LiveApiError::MalformedResponse { .. }
    )
}

fn sleep_before_next_lookup(interval_ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(interval_ms));
}

#[cfg(test)]
#[path = "live_chat_test_fixtures.rs"]
mod live_chat_test_fixtures;

#[cfg(test)]
#[path = "live_chat_resolution_tests.rs"]
mod live_chat_resolution_tests;
