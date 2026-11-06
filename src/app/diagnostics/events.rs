//! Structured JSONL event model: one serialize-only row kind per emitted
//! line. Every row carries `event` plus `epoch_seconds` (wall clock, for
//! operator correlation only) and kind-specific fields. No row can represent
//! message text, an author, a token, a stream key, or an RTMPS URL.

use serde::Serialize;

/// One per-message stage row (`stage` is `dequeued`, `rendered`, or `written`).
#[derive(Debug, Serialize)]
pub(super) struct TimingEvent<'a> {
    event: &'static str,
    epoch_seconds: u64,
    correlation: Option<&'a str>,
    provider_epoch_seconds: Option<u64>,
    stage: &'static str,
    lag_ns: Option<u64>,
    queue_depth: Option<u64>,
}

/// The stage-specific fields of one timing row (≤2-args rule parameter object).
pub(super) struct TimingDetails<'a> {
    correlation: Option<&'a str>,
    provider_epoch_seconds: Option<u64>,
    lag_ns: Option<u64>,
    queue_depth: Option<u64>,
}

impl<'a> TimingDetails<'a> {
    pub(super) fn dequeued(
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

    pub(super) fn with_queue_depth(mut self, queue_depth: u64) -> Self {
        self.queue_depth = Some(queue_depth);
        self
    }

    pub(super) fn stage(correlation: Option<&'a str>, lag_ns: u64) -> Self {
        Self {
            correlation,
            provider_epoch_seconds: None,
            lag_ns: Some(lag_ns),
            queue_depth: None,
        }
    }

    pub(super) fn with_provider(mut self, provider_epoch_seconds: Option<u64>) -> Self {
        self.provider_epoch_seconds = provider_epoch_seconds;
        self
    }
}

impl<'a> TimingEvent<'a> {
    pub(super) fn new(stage: &'static str, details: TimingDetails<'a>) -> Self {
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
pub(super) struct QueueEvent<'a> {
    event: &'static str,
    epoch_seconds: u64,
    correlation: Option<&'a str>,
    batch_size: Option<u64>,
    queue_depth: u64,
    backpressure: Option<bool>,
    wait_ns: Option<u64>,
    backoff_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'a str>,
}

/// The enqueue-outcome fields of one queue row (≤2-args rule parameter object).
pub(super) struct QueueEnqueueFields<'a> {
    pub(super) correlation: Option<&'a str>,
    pub(super) backpressured: bool,
    pub(super) wait_ns: u64,
}

impl<'a> QueueEvent<'a> {
    /// One queue row skeleton; the constructors fill their kind-specific fields.
    fn with_depth(queue_depth: u64) -> Self {
        Self {
            event: "queue",
            epoch_seconds: epoch_seconds_now(),
            correlation: None,
            batch_size: None,
            queue_depth,
            backpressure: None,
            wait_ns: None,
            backoff_ms: None,
            reason: None,
        }
    }

    pub(super) fn enqueue(fields: QueueEnqueueFields<'a>, queue_depth: u64) -> Self {
        let mut event = Self::with_depth(queue_depth);
        event.correlation = fields.correlation;
        event.backpressure = Some(fields.backpressured);
        event.wait_ns = Some(fields.wait_ns);
        event
    }

    pub(super) fn batch(size: u64, queue_depth: u64) -> Self {
        let mut event = Self::with_depth(queue_depth);
        event.batch_size = Some(size);
        event
    }

    pub(super) fn full(correlation: Option<&'a str>, queue_depth: u64) -> Self {
        let mut event = Self::with_depth(queue_depth);
        event.correlation = correlation;
        event.backpressure = Some(true);
        event
    }

    pub(super) fn reconnect(backoff_ms: u64, queue_depth: u64) -> Self {
        let mut event = Self::with_depth(queue_depth);
        event.backoff_ms = Some(backoff_ms);
        event
    }

    pub(super) fn with_reason(mut self, reason: &'static str) -> Self {
        self.reason = Some(reason);
        self
    }
}

/// Wall-clock seconds since the UNIX epoch for row correlation (never diffed
/// against monotonic values).
pub(super) fn epoch_seconds_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}
