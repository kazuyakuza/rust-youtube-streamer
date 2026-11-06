//! Opt-in, low-overhead pipeline observability (TODO Task 1): a read-only
//! collector shared between the chat worker thread and the main frame loop.
//! Every capability is disabled by default, the aggregate state is bounded
//! (fixed counters, fixed histograms, a capped in-flight map), emitted rows
//! never carry message text, authors, tokens, stream keys, or RTMPS URLs, and
//! disabled diagnostics allocate nothing at all.

mod collector;
mod collector_stages;
mod events;
mod events_summary;
mod histogram;
mod state;
mod writer;

use crate::config::diagnostics_config::DiagnosticsConfig;
use crate::config::AppConfig;

pub(crate) use collector::DiagnosticsCollector;
use collector_stages::CollectorSettings;

#[cfg(test)]
pub(crate) mod diagnostics_test_fixtures;

const NANOS_PER_SECOND: u64 = 1_000_000_000;

impl DiagnosticsCollector {
    /// The ONE construction point: `None` unless at least one capability is
    /// enabled, so a disabled configuration never allocates state, a clock,
    /// or a file writer.
    pub(crate) fn from_config(config: &AppConfig) -> Option<Self> {
        let diagnostics = config.diagnostics.as_ref()?;
        if is_any_capability_enabled(diagnostics) {
            Some(Self::from_settings(CollectorSettings::from_diagnostics(
                diagnostics,
                frame_interval_nanos(config.video.fps),
            )))
        } else {
            None
        }
    }
}

fn is_any_capability_enabled(diagnostics: &DiagnosticsConfig) -> bool {
    diagnostics.pipeline_timing_enabled()
        || diagnostics.queue_metrics_enabled()
        || diagnostics.periodic_summary_enabled()
}

fn frame_interval_nanos(fps: u32) -> u64 {
    if fps == 0 {
        return NANOS_PER_SECOND;
    }
    NANOS_PER_SECOND / u64::from(fps)
}

#[cfg(test)]
#[path = "histogram_tests.rs"]
mod histogram_tests;

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;

#[cfg(test)]
#[path = "collector_tests.rs"]
mod collector_tests;

#[cfg(test)]
#[path = "collector_provider_tests.rs"]
mod collector_provider_tests;

#[cfg(test)]
#[path = "writer_tests.rs"]
mod writer_tests;
