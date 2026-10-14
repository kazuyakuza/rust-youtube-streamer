//! The youtube-local wait-cancellation seam (plan F-T2-4, D-T2-10): a
//! clone-shared stop flag that the readiness poll loops check between
//! attempts. The app layer's Ctrl+C handler flips BOTH the app `StopFlag`
//! and a `WaitCanceller` clone (the Task-3 contract, now wired in
//! `app::run`); `youtube` itself never reads app types — the sealed
//! boundary rule forbids importing `crate::app`, so this mirrors the
//! `StopFlag` shape locally (`Arc<AtomicBool>`, `SeqCst` ordering, the
//! same duplication-over-cross-module-contract ruling as
//! `utc_timestamp`).
//!
//! Design plan:
//! `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md`
//! (F-T2-4, D-T2-10); TODO `.agent/todos/20261013/20261013-todo-1.md`
//! Task 2.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Shared wait-cancellation request. `Clone` shares ONE flag between the
/// canceller owner and every cloned handle; starts cleared.
#[derive(Clone)]
pub(crate) struct WaitCanceller {
    cancelled: Arc<AtomicBool>,
}

impl WaitCanceller {
    /// Constructed by the run-mode Ctrl+C link (the canceller the handler
    /// shares with the waits).
    pub(crate) fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Handler-side mutation; `AtomicBool` allows mutation through `&self`.
    pub(crate) fn request_cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
#[path = "wait_cancellation_tests.rs"]
mod wait_cancellation_tests;
