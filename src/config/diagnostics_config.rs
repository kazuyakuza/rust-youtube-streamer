//! Optional diagnostics section: every observability capability is a
//! separately switchable flag disabled by default. The section is absent from
//! pre-0.14.0 operator configuration, so its fields are all optional and the
//! accessors resolve the documented defaults.

use serde::Deserialize;

pub const DEFAULT_SUMMARY_INTERVAL_SECONDS: u32 = 30;
pub const DEFAULT_MAX_FILE_BYTES: u32 = 1_048_576;
pub const DEFAULT_MAX_GENERATIONS: u32 = 3;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticsConfig {
    /// Per-message stage timing rows plus the in-memory frame histograms.
    pub pipeline_timing_events: Option<bool>,
    /// Queue, backpressure, and reconnect counters.
    pub queue_metrics: Option<bool>,
    /// One aggregate summary row per configured period.
    pub periodic_summary: Option<bool>,
    /// Summary period in seconds when `periodic_summary` is enabled.
    pub summary_interval_seconds: Option<u32>,
    /// Structured JSONL diagnostics file, separate from the chat log.
    pub output_file: Option<String>,
    /// Per-generation size cap in bytes.
    pub max_file_bytes: Option<u32>,
    /// Kept rotated generation count.
    pub max_generations: Option<u32>,
}

impl DiagnosticsConfig {
    pub fn pipeline_timing_enabled(&self) -> bool {
        self.pipeline_timing_events.unwrap_or(false)
    }

    pub fn queue_metrics_enabled(&self) -> bool {
        self.queue_metrics.unwrap_or(false)
    }

    pub fn periodic_summary_enabled(&self) -> bool {
        self.periodic_summary.unwrap_or(false)
    }

    pub fn summary_interval(&self) -> u32 {
        self.summary_interval_seconds
            .unwrap_or(DEFAULT_SUMMARY_INTERVAL_SECONDS)
    }

    pub fn output_file(&self) -> Option<&str> {
        self.output_file.as_deref()
    }

    pub fn max_file_bytes(&self) -> u32 {
        self.max_file_bytes.unwrap_or(DEFAULT_MAX_FILE_BYTES)
    }

    pub fn max_generations(&self) -> u32 {
        self.max_generations.unwrap_or(DEFAULT_MAX_GENERATIONS)
    }
}
