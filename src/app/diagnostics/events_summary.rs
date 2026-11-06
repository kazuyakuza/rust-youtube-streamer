//! The periodic summary half of the JSONL event model: one aggregate row
//! carrying the full bounded snapshot, split from `events` per the file-cap
//! precedent. Serialization bytes are unchanged by the split.

use serde::Serialize;

use crate::app::diagnostics::histogram::HistogramSummary;

use super::events::epoch_seconds_now;

const CLOCK_DOMAIN_NOTE: &str = "provider published timestamps are provider-supplied wall clock (publishedAt); all *_ns values are local monotonic deltas; subtracting stamps across clock domains is not a precise duration (uncorrectable clock skew)";

/// One periodic aggregate row, carrying the full bounded snapshot.
#[derive(Debug, Serialize)]
pub(crate) struct SummaryEvent {
    pub(crate) event: &'static str,
    pub(crate) epoch_seconds: u64,
    pub(crate) period_seconds: u32,
    pub(crate) clock_note: &'static str,
    #[serde(flatten)]
    pub(crate) snapshot: SummarySnapshot,
}

impl SummaryEvent {
    pub(crate) fn new(period_seconds: u32, snapshot: SummarySnapshot) -> Self {
        Self {
            event: "summary",
            epoch_seconds: epoch_seconds_now(),
            period_seconds,
            clock_note: CLOCK_DOMAIN_NOTE,
            snapshot,
        }
    }
}

/// The counters and histogram summaries reported by one summary row.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct SummarySnapshot {
    pub(crate) received_total: u64,
    pub(crate) enqueued_total: u64,
    pub(crate) dequeued_total: u64,
    pub(crate) uncorrelated_total: u64,
    pub(crate) full_retry_total: u64,
    pub(crate) worker_retained_total: u64,
    pub(crate) reconnect_count: u64,
    pub(crate) reconnect_backoff_total_ms: u64,
    pub(crate) in_flight_high_water: u64,
    pub(crate) queue_depth_estimate: u64,
    pub(crate) oldest_in_flight_age_ns: Option<u64>,
    pub(crate) message_channel_ns: HistogramSummary,
    pub(crate) message_render_ns: HistogramSummary,
    pub(crate) frame_interval_ns: HistogramSummary,
    pub(crate) frame_jitter_ns: HistogramSummary,
    pub(crate) render_ns: HistogramSummary,
    pub(crate) write_ns: HistogramSummary,
}
