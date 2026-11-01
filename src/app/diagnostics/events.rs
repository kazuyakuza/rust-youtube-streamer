//! Structured JSONL event model: one serialize-only row kind per emitted
//! line. Every row carries `event` plus `epoch_seconds` (wall clock, for
//! operator correlation only) and kind-specific fields. No row can represent
//! message text, an author, a token, a stream key, or an RTMPS URL.

use serde::Serialize;

use crate::app::diagnostics::histogram::HistogramSummary;

pub(crate) const CLOCK_DOMAIN_NOTE: &str = "provider published timestamps are provider-supplied wall clock (publishedAt); all *_ns values are local monotonic deltas; subtracting stamps across clock domains is not a precise duration (uncorrectable clock skew)";

/// One per-message stage row (`stage` is `dequeued`, `rendered`, or `written`).
#[derive(Debug, Serialize)]
pub(crate) struct TimingEvent<'a> {
    pub(crate) event: &'static str,
    pub(crate) epoch_seconds: u64,
    pub(crate) correlation: Option<&'a str>,
    pub(crate) provider_epoch_seconds: Option<u64>,
    pub(crate) stage: &'static str,
    pub(crate) lag_ns: Option<u64>,
    pub(crate) queue_depth: Option<u64>,
}

/// The stage-specific fields of one timing row (≤2-args rule parameter object).
pub(crate) struct TimingDetails<'a> {
    pub(crate) correlation: Option<&'a str>,
    pub(crate) provider_epoch_seconds: Option<u64>,
    pub(crate) lag_ns: Option<u64>,
    pub(crate) queue_depth: Option<u64>,
}

impl<'a> TimingDetails<'a> {
    pub(crate) fn dequeued(
        correlation: Option<&'a str>,
        provider_epoch_seconds: Option<u64>,
    ) -> Self {
        Self {
            correlation,
            provider_epoch_seconds,
            lag_ns: None,
            queue_depth: None,
        }
    }

    pub(crate) fn with_queue_depth(mut self, queue_depth: u64) -> Self {
        self.queue_depth = Some(queue_depth);
        self
    }

    pub(crate) fn stage(correlation: Option<&'a str>, lag_ns: u64) -> Self {
        Self {
            correlation,
            provider_epoch_seconds: None,
            lag_ns: Some(lag_ns),
            queue_depth: None,
        }
    }
}

impl<'a> TimingEvent<'a> {
    pub(crate) fn new(stage: &'static str, details: TimingDetails<'a>) -> Self {
        Self {
            event: "message",
            epoch_seconds: epoch_seconds_now(),
            correlation: details.correlation,
            provider_epoch_seconds: details.provider_epoch_seconds,
            stage,
            lag_ns: details.lag_ns,
            queue_depth: details.queue_depth,
        }
    }
}

/// One queue/backpressure row: batch receipt, enqueue outcome, or reconnect.
#[derive(Debug, Serialize)]
pub(crate) struct QueueEvent<'a> {
    pub(crate) event: &'static str,
    pub(crate) epoch_seconds: u64,
    pub(crate) correlation: Option<&'a str>,
    pub(crate) batch_size: Option<u64>,
    pub(crate) queue_depth: u64,
    pub(crate) backpressure: Option<bool>,
    pub(crate) wait_ns: Option<u64>,
    pub(crate) backoff_ms: Option<u64>,
}

/// The enqueue-outcome fields of one queue row (≤2-args rule parameter object).
pub(crate) struct QueueEnqueueFields<'a> {
    pub(crate) correlation: Option<&'a str>,
    pub(crate) backpressured: bool,
    pub(crate) wait_ns: u64,
}

impl<'a> QueueEvent<'a> {
    pub(crate) fn enqueue(fields: QueueEnqueueFields<'a>, queue_depth: u64) -> Self {
        Self {
            event: "queue",
            epoch_seconds: epoch_seconds_now(),
            correlation: fields.correlation,
            batch_size: None,
            queue_depth,
            backpressure: Some(fields.backpressured),
            wait_ns: Some(fields.wait_ns),
            backoff_ms: None,
        }
    }

    pub(crate) fn batch(size: u64, queue_depth: u64) -> Self {
        Self {
            event: "queue",
            epoch_seconds: epoch_seconds_now(),
            correlation: None,
            batch_size: Some(size),
            queue_depth,
            backpressure: None,
            wait_ns: None,
            backoff_ms: None,
        }
    }

    pub(crate) fn full(correlation: Option<&'a str>, queue_depth: u64) -> Self {
        Self {
            event: "queue",
            epoch_seconds: epoch_seconds_now(),
            correlation,
            batch_size: None,
            queue_depth,
            backpressure: Some(true),
            wait_ns: None,
            backoff_ms: None,
        }
    }

    pub(crate) fn reconnect(backoff_ms: u64, queue_depth: u64) -> Self {
        Self {
            event: "queue",
            epoch_seconds: epoch_seconds_now(),
            correlation: None,
            batch_size: None,
            queue_depth,
            backpressure: None,
            wait_ns: None,
            backoff_ms: Some(backoff_ms),
        }
    }
}

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

/// Wall-clock seconds since the UNIX epoch for row correlation (never diffed
/// against monotonic values).
pub(crate) fn epoch_seconds_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}
