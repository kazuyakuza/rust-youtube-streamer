//! The worker/queue stage-reporting half of the diagnostics collector: the
//! chat-stream delivery side records batch receipts, enqueue attempts,
//! full-channel retries, retained-under-backpressure enqueues, and reconnect
//! backoff. Settings and the per-handle worker bookkeeping live here; the
//! frame-loop half and the construction stay in `collector`.

use std::path::PathBuf;

use crate::app::diagnostics::events::{QueueEnqueueFields, QueueEvent};
use crate::app::diagnostics::state::InFlightEntry;
use crate::config::diagnostics_config::DiagnosticsConfig;

use super::collector::DiagnosticsCollector;

#[derive(Clone)]
pub(super) struct CollectorSettings {
    pub(super) pipeline_timing: bool,
    pub(super) queue_metrics: bool,
    pub(super) periodic_summary: bool,
    pub(super) summary_interval_seconds: u32,
    pub(super) output_file: Option<PathBuf>,
    pub(super) max_file_bytes: u32,
    pub(super) max_generations: u32,
    pub(super) pacing_interval_nanos: u64,
}

impl CollectorSettings {
    pub(super) fn from_diagnostics(
        diagnostics: &DiagnosticsConfig,
        pacing_interval_nanos: u64,
    ) -> Self {
        Self {
            pipeline_timing: diagnostics.pipeline_timing_enabled(),
            queue_metrics: diagnostics.queue_metrics_enabled(),
            periodic_summary: diagnostics.periodic_summary_enabled(),
            summary_interval_seconds: diagnostics.summary_interval(),
            output_file: diagnostics.output_file().map(PathBuf::from),
            max_file_bytes: diagnostics.max_file_bytes(),
            max_generations: diagnostics.max_generations(),
            pacing_interval_nanos,
        }
    }
}

#[derive(Clone, Default)]
pub(super) struct CollectorLocal {
    pub(super) last_receipt_nanos: Option<u64>,
    pub(super) backpressured: bool,
    pub(super) last_tick_nanos: Option<u64>,
    pub(super) render_start_nanos: Option<u64>,
    pub(super) write_start_nanos: Option<u64>,
    pub(super) last_summary_nanos: Option<u64>,
    pub(super) last_dequeued: Option<DequeuedMessage>,
}

#[derive(Clone)]
pub(super) struct DequeuedMessage {
    pub(super) correlation: Option<String>,
    pub(super) stamp_nanos: u64,
}

impl DiagnosticsCollector {
    pub(crate) fn report_batch_received(&self, size: u64) {
        let depth = self.state_depth(|state| state.add_received(size));
        if self.settings.queue_metrics {
            self.emit(&QueueEvent::batch(size, depth));
        }
    }

    pub(crate) fn note_worker_receipt(&self) {
        self.local.borrow_mut().last_receipt_nanos = Some(self.now());
    }

    pub(crate) fn report_enqueue_full(&self, correlation: Option<&str>) {
        self.local.borrow_mut().backpressured = true;
        let depth = self.state_depth(|state| state.add_full_retry());
        if self.settings.queue_metrics {
            self.emit(&QueueEvent::full(correlation, depth));
        }
    }

    pub(crate) fn report_enqueue_success(&self, correlation: Option<&str>) {
        let now = self.now();
        let (received_nanos, backpressured) = self.take_receipt();
        let entry = correlation.map(|id| InFlightEntry {
            correlation: id.to_string(),
            received_nanos,
            enqueued_nanos: now,
            backpressured,
        });
        let depth = self.state_depth(|state| state.add_enqueued(entry, backpressured));
        if self.settings.queue_metrics {
            let fields = QueueEnqueueFields {
                correlation,
                backpressured,
                wait_ns: now.saturating_sub(received_nanos),
            };
            self.emit(&QueueEvent::enqueue(fields, depth));
        }
    }

    pub(crate) fn report_reconnect(&self, backoff_ms: u64) {
        let depth = self.state_depth(|state| state.add_reconnect(backoff_ms));
        if self.settings.queue_metrics {
            self.emit(&QueueEvent::reconnect(backoff_ms, depth));
        }
    }

    fn take_receipt(&self) -> (u64, bool) {
        let now = self.now();
        let mut local = self.local.borrow_mut();
        let received = local.last_receipt_nanos.take().unwrap_or(now);
        let backpressured = std::mem::take(&mut local.backpressured);
        (received, backpressured)
    }
}
