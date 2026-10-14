//! The youtube-local wait-cancellation seam (plan F-T2-4, D-T2-10): a
//! clone-shared stop flag that the readiness poll loops check between
//! attempts. Task-3 contract: the app layer's Ctrl+C handler flips BOTH
//! the app `StopFlag` and a `WaitCanceller` clone (or an equivalent
//! adaptation at the app layer); `youtube` itself never reads app types —
//! the sealed boundary rule forbids importing `crate::app`, so this
//! mirrors the `StopFlag` shape locally (`Arc<AtomicBool>`, `SeqCst`
//! ordering, the same duplication-over-cross-module-contract ruling as
//! `utc_timestamp`).
//!
//! Staging: the module declaration carries the task-2 staging allow
//! (D-T2-14 step 1 for the new files) — the poll loops that consume
//! `is_cancelled()` are themselves dead until Task 3's run wiring roots
//! the chain (compiler-verified reach-from-roots lint). Inside the
//! module, `new()` and `request_cancel()` additionally carry item-level
//! allows (F-T2-4) naming their Task-3 Ctrl+C-link retirement.
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
    /// Retire when Task 3's Ctrl+C link flips this seam (it constructs
    /// the canceller the handler shares with the waits).
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Handler-side mutation; `AtomicBool` allows mutation through `&self`.
    /// Retire when Task 3's Ctrl+C link flips this seam.
    #[allow(dead_code)]
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
