//! Logging initialization seam used once by the startup pipeline, before any
//! mode handler runs.

use super::error::LogInitError;

/// Initializes structured application logging once.
///
/// Task 3 replaces this body with tracing + tracing-subscriber init honoring
/// RUST_LOG; call sites and signature stay unchanged.
pub(super) fn init_logging() -> Result<(), LogInitError> {
    Ok(())
}
