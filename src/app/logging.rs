//! Logging initialization seam used once by the startup pipeline, before any
//! mode handler runs. Call sites depend only on this function's signature,
//! never on logger internals, which keeps logging machinery out of the
//! pipeline and the mode handlers.

use super::error::LogInitError;

/// Initializes structured application logging once.
///
/// Current behavior is a deliberate no-op that always succeeds: this phase
/// ships no logger, so initialization cannot fail. A later phase replaces
/// this body with structured logging (tracing + tracing-subscriber) honoring
/// `RUST_LOG` and falling back to a safe documented default when the
/// environment variable is unset or invalid, while call sites and this
/// signature stay unchanged.
pub(super) fn init_logging() -> Result<(), LogInitError> {
    Ok(())
}
