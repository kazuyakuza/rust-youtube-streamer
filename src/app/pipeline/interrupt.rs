//! Ctrl+C as a shutdown request (TODO Task 2; plan decision S7, D7): the
//! `ctrlc` crate's dedicated handler thread flips an `Arc<AtomicBool>` shared
//! with the pacing loop, which polls it at the top of every frame iteration.
//! Verified cross-platform scope and documented limitations:
//! 1. Installing the handler REPLACES default Ctrl+C termination on both
//!    platforms: the handler only sets the flag, repeated Ctrl+C is inert,
//!    and if the pipeline is stuck in a long blocking op (stdin write to a
//!    hung FFmpeg, unbounded `finish()` wait) the operator must terminate the
//!    process with OS tools. Documented limitation — never claimed otherwise.
//! 2. Ctrl+C during a pacing sleep: the flag is set asynchronously and
//!    observed after the current sleep returns — shutdown latency is at most
//!    one frame interval.
//! 3. Ctrl+C while rendering/writing a frame: observed at the next iteration
//!    top — the current frame completes, then shut down (at most one extra
//!    frame).
//! 4. In Docker/Linux the foreground process receives SIGINT normally
//!    (interactive runs); detached background processes have no console
//!    signal — documented, no claims about daemonized execution.
//! 5. The FFmpeg exit wait in `finish()` is intentionally unbounded in this
//!    phase (Task-1 contract: no timeout; a configured-timeout field needs D10
//!    — deferred, not silently claimed).

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Shared shutdown request. `Clone` shares ONE `Arc<AtomicBool>` between the
/// Ctrl+C handler and the pipeline owner. Starts cleared.
#[derive(Clone, Debug)]
pub(crate) struct StopFlag {
    stopped: Arc<AtomicBool>,
}

impl StopFlag {
    pub(crate) fn new() -> Self {
        Self {
            stopped: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Handler-side mutation; `AtomicBool` allows mutation through `&self`.
    pub(crate) fn request_stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }

    pub(crate) fn is_requested(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }
}

/// Fatal failure of the ctrlc installation, typed (no panics); carries only
/// a display reason, never the crate error type.
#[derive(Debug)]
pub(crate) struct InterruptHandlerError {
    reason: String,
}

impl fmt::Display for InterruptHandlerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unable to install the Ctrl+C interrupt handler: {}",
            self.reason
        )
    }
}

impl std::error::Error for InterruptHandlerError {}

/// Installs the application's single Ctrl+C handler (app-level, exactly
/// once) and returns the flag the handler flips. Called by the mode command
/// before the pipeline runs.
pub(crate) fn install_interrupt_handler() -> Result<StopFlag, InterruptHandlerError> {
    let stop_flag = StopFlag::new();
    let handler_flag = stop_flag.clone();
    ctrlc::set_handler(move || handler_flag.request_stop()).map_err(|error| {
        InterruptHandlerError {
            reason: error.to_string(),
        }
    })?;
    Ok(stop_flag)
}
