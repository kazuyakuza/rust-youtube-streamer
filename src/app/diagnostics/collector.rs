//! The collector: the ONE opt-in handle, cloned to the chat worker and the frame
//! loop; disabled configs never build one. Cross-thread aggregation sits behind
//! `Arc<Mutex<state>>`; per-clone `RefCell` bookkeeping is safe (one thread per
//! clone, no interleaved sends). Worker/queue reporting: `collector_stages` sibling.

use std::cell::RefCell;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Instant;

use serde::Serialize;

use crate::app::diagnostics::events::{SummaryEvent, TimingDetails, TimingEvent};
use crate::app::diagnostics::state::DiagnosticsState;
use crate::app::diagnostics::writer::{build_writer, DiagnosticsFileWriter};
use crate::chat::ChatMessage;

use super::collector_stages::{CollectorLocal, CollectorSettings, DequeuedMessage};

/// The monotonic time seam: one process-global anchor so stamps taken on the
/// worker thread stay diffable against stamps taken on the main thread.
pub(super) trait MonotonicNanos: Send + Sync {
    fn now_nanos(&self) -> u64;
}

pub(super) struct InstantMonotonicNanos;

static GLOBAL_ANCHOR: LazyLock<Instant> = LazyLock::new(Instant::now);

impl MonotonicNanos for InstantMonotonicNanos {
    fn now_nanos(&self) -> u64 {
        u64::try_from(GLOBAL_ANCHOR.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }
}

#[derive(Clone)]
pub(crate) struct DiagnosticsCollector {
    pub(super) settings: Arc<CollectorSettings>,
    pub(super) state: Arc<Mutex<DiagnosticsState>>,
    pub(super) clock: Arc<dyn MonotonicNanos>,
    writer: Option<Arc<Mutex<DiagnosticsFileWriter>>>,
    pub(super) local: RefCell<CollectorLocal>,
}

impl DiagnosticsCollector {
    pub(super) fn from_settings(settings: CollectorSettings) -> Self {
        let writer = build_writer(&settings);
        Self {
            settings: Arc::new(settings),
            state: Arc::new(Mutex::new(DiagnosticsState::new())),
            clock: Arc::new(InstantMonotonicNanos),
            writer,
            local: RefCell::new(CollectorLocal::default()),
        }
    }

    pub(crate) fn note_message_dequeued(&self, message: &ChatMessage) {
        let now = self.now();
        let correlation = message.message_id();
        let depth = self.state_depth(|state| state.take_dequeued(correlation, now));
        if !self.settings.pipeline_timing {
            return;
        }
        self.local.borrow_mut().last_dequeued = Some(DequeuedMessage {
            correlation: correlation.map(str::to_string),
            stamp_nanos: now,
        });
        let details =
            TimingDetails::dequeued(correlation, message.timestamp()).with_queue_depth(depth);
        self.emit(&TimingEvent::new("dequeued", details));
    }

    pub(crate) fn note_render_started(&self) {
        if self.settings.pipeline_timing {
            self.local.borrow_mut().render_start_nanos = Some(self.now());
        }
    }

    pub(crate) fn note_render_completed(&self) {
        let now = self.now();
        let start = self.local.borrow_mut().render_start_nanos.take();
        if let Some(start) = start {
            self.with_state(|state| state.record_render(now.saturating_sub(start)));
            self.record_message_render(now);
        }
    }

    pub(crate) fn note_write_started(&self) {
        if self.settings.pipeline_timing {
            self.local.borrow_mut().write_start_nanos = Some(self.now());
        }
    }

    pub(crate) fn note_write_completed(&self) {
        let now = self.now();
        let start = self.local.borrow_mut().write_start_nanos.take();
        if let Some(start) = start {
            self.with_state(|state| state.record_write(now.saturating_sub(start)));
            let dequeued = self.local.borrow_mut().last_dequeued.take();
            if let Some(entry) = dequeued {
                let lag_ns = now.saturating_sub(entry.stamp_nanos);
                let details = TimingDetails::stage(entry.correlation.as_deref(), lag_ns);
                self.emit(&TimingEvent::new("written", details));
            }
        }
    }

    pub(crate) fn note_frame_tick(&self) {
        if !self.settings.pipeline_timing {
            return;
        }
        let now = self.now();
        let previous = self.local.borrow_mut().last_tick_nanos.replace(now);
        if let Some(previous) = previous {
            let interval = now.saturating_sub(previous);
            let jitter = interval.abs_diff(self.settings.pacing_interval_nanos);
            self.with_state(|state| state.record_frame_tick(interval, jitter));
        }
    }

    pub(crate) fn maybe_emit_summary(&self) {
        if !self.settings.periodic_summary {
            return;
        }
        let now = self.now();
        if self.summary_is_due(now) {
            self.emit_summary(now);
        }
    }

    fn record_message_render(&self, now: u64) {
        let dequeued = self.local.borrow();
        let Some(entry) = dequeued.last_dequeued.as_ref() else {
            return;
        };
        let message_ns = now.saturating_sub(entry.stamp_nanos);
        self.with_state(|state| state.record_message_render(message_ns));
        let details = TimingDetails::stage(entry.correlation.as_deref(), message_ns);
        self.emit(&TimingEvent::new("rendered", details));
    }

    fn summary_is_due(&self, now: u64) -> bool {
        let interval = u64::from(self.settings.summary_interval_seconds) * super::NANOS_PER_SECOND;
        let mut local = self.local.borrow_mut();
        if local.last_summary_nanos.is_none() {
            local.last_summary_nanos = Some(now);
            return false;
        }
        let due = local
            .last_summary_nanos
            .is_some_and(|last| now.saturating_sub(last) >= interval);
        if due {
            local.last_summary_nanos = Some(now);
        }
        due
    }

    fn emit_summary(&self, now: u64) {
        let snapshot = self.with_state(|state| state.snapshot(now));
        self.emit(&SummaryEvent::new(
            self.settings.summary_interval_seconds,
            snapshot,
        ));
    }

    pub(super) fn emit(&self, event: &impl Serialize) {
        let Ok(line) = serde_json::to_string(event) else {
            return;
        };
        match &self.writer {
            Some(writer) => write_locked(writer, &line),
            None => tracing::debug!(target: "diagnostics", "{}", line),
        }
    }

    pub(super) fn now(&self) -> u64 {
        self.clock.now_nanos()
    }

    pub(super) fn with_state<T>(&self, action: impl FnOnce(&mut DiagnosticsState) -> T) -> T {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        action(&mut state)
    }

    pub(super) fn state_depth(&self, action: impl FnOnce(&mut DiagnosticsState)) -> u64 {
        self.with_state(|state| {
            action(state);
            state.queue_depth()
        })
    }
}

fn write_locked(writer: &Arc<Mutex<DiagnosticsFileWriter>>, line: &str) {
    if let Ok(mut writer) = writer.lock() {
        writer.write_event(line);
    }
}
