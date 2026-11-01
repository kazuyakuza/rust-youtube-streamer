//! The bounded sleep and terminal helper block of the chat delivery machinery
//! (split from `chat_stream_delivery` per the line-cap precedent): stop-sliced
//! waits, the capped doubling backoff, the one-shot terminal writer, and the
//! panic-detail constant. Everything here is stop-aware and allocation-free on
//! the steady path.

use std::sync::Mutex;

use crate::app::pipeline::{ChatRetrievalError, StopFlag};

use crate::app::run::chat_worker::{CHAT_BACKOFF_MAX_MS, CHAT_SLEEP_SLICE_MS};

/// Bounded stop-sliced wait (D-6) — byte-verbatim pair (the sync slice fn
/// with the existing cfg-split: production `thread::sleep`, test no-op).
pub(in crate::app::run) fn sleep_in_stop_slices(total_ms: u64, stop_flag: &StopFlag) {
    let mut remaining_ms = total_ms;
    while remaining_ms > 0 {
        if stop_flag.is_requested() {
            return;
        }
        let slice_ms = remaining_ms.min(CHAT_SLEEP_SLICE_MS);
        sleep_slice(slice_ms);
        remaining_ms -= slice_ms;
    }
}

pub(in crate::app::run) fn backoff_delay_ms(attempt: u32, base_ms: u64) -> u64 {
    // Phase-06 Task 3: the D-6 pinned max/min body (record §4.3.7) — the
    // clippy-named allow keeps the plan's shape (plan §6(f)).
    #[allow(clippy::manual_clamp)]
    let base = base_ms.max(CHAT_SLEEP_SLICE_MS).min(CHAT_BACKOFF_MAX_MS);
    let doubled = base.saturating_mul(2u64.saturating_pow(attempt - 1));
    doubled.min(CHAT_BACKOFF_MAX_MS)
}

/// Stores the terminal failure exactly once (poisoned locks ignored — the
/// existing `if let Ok` convention).
pub(in crate::app::run) fn record_terminal(
    cell: &Mutex<Option<ChatRetrievalError>>,
    error: ChatRetrievalError,
) {
    if let Ok(mut recorded) = cell.lock() {
        *recorded = Some(error);
    }
}

pub(in crate::app::run) const CHAT_WORKER_PANIC_DETAIL: &str =
    "the chat streaming worker panicked; its backtrace is on the standard error stream and the run fails with exit code 1";

pub(in crate::app::run) fn transport_terminal(detail: String) -> ChatRetrievalError {
    ChatRetrievalError::Transport { detail }
}

#[cfg(not(test))]
pub(in crate::app::run) fn sleep_slice(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

#[cfg(test)]
pub(in crate::app::run) fn sleep_slice(_ms: u64) {}
