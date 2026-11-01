//! The bounded aggregate shared between the chat worker and the frame loop.
//! Fixed counters and histograms plus a capped in-flight map (oldest evicted
//! on overflow) keep memory bounded by construction; every mutation happens
//! under the collector's one mutex.

use std::collections::VecDeque;

use crate::app::diagnostics::events::SummarySnapshot;
use crate::app::diagnostics::histogram::DurationHistogram;

pub(crate) const IN_FLIGHT_CAP: usize = 128;

/// One message inside the bounded application channel, tracked for latency.
pub(crate) struct InFlightEntry {
    pub(crate) correlation: String,
    pub(crate) received_nanos: u64,
    pub(crate) enqueued_nanos: u64,
    pub(crate) backpressured: bool,
}

pub(crate) struct DiagnosticsState {
    received_total: u64,
    enqueued_total: u64,
    dequeued_total: u64,
    uncorrelated_total: u64,
    full_retry_total: u64,
    worker_retained_total: u64,
    reconnect_count: u64,
    reconnect_backoff_total_ms: u64,
    in_flight: VecDeque<InFlightEntry>,
    in_flight_high_water: u64,
    message_channel_ns: DurationHistogram,
    message_render_ns: DurationHistogram,
    frame_interval_ns: DurationHistogram,
    frame_jitter_ns: DurationHistogram,
    render_ns: DurationHistogram,
    write_ns: DurationHistogram,
}

impl DiagnosticsState {
    pub(crate) fn new() -> Self {
        Self {
            received_total: 0,
            enqueued_total: 0,
            dequeued_total: 0,
            uncorrelated_total: 0,
            full_retry_total: 0,
            worker_retained_total: 0,
            reconnect_count: 0,
            reconnect_backoff_total_ms: 0,
            in_flight: VecDeque::new(),
            in_flight_high_water: 0,
            message_channel_ns: DurationHistogram::new(),
            message_render_ns: DurationHistogram::new(),
            frame_interval_ns: DurationHistogram::new(),
            frame_jitter_ns: DurationHistogram::new(),
            render_ns: DurationHistogram::new(),
            write_ns: DurationHistogram::new(),
        }
    }

    pub(crate) fn add_received(&mut self, batch_size: u64) {
        self.received_total = self.received_total.saturating_add(batch_size);
    }

    pub(crate) fn add_full_retry(&mut self) {
        self.full_retry_total = self.full_retry_total.saturating_add(1);
    }

    pub(crate) fn add_enqueued(&mut self, entry: Option<InFlightEntry>, backpressured: bool) {
        self.enqueued_total = self.enqueued_total.saturating_add(1);
        let retained = entry
            .as_ref()
            .map_or(backpressured, |in_flight| in_flight.backpressured);
        if retained {
            self.worker_retained_total = self.worker_retained_total.saturating_add(1);
        }
        if let Some(entry) = entry {
            self.in_flight.push_back(entry);
            self.evict_oldest_beyond_cap();
            self.in_flight_high_water = self.in_flight_high_water.max(self.in_flight.len() as u64);
        }
    }

    pub(crate) fn add_reconnect(&mut self, backoff_ms: u64) {
        self.reconnect_count = self.reconnect_count.saturating_add(1);
        self.reconnect_backoff_total_ms =
            self.reconnect_backoff_total_ms.saturating_add(backoff_ms);
    }

    /// Pops the matching in-flight entry and records channel latency; an
    /// absent or unmatched correlation is counted, never dropped silently.
    pub(crate) fn take_dequeued(&mut self, correlation: Option<&str>, now_nanos: u64) {
        self.dequeued_total = self.dequeued_total.saturating_add(1);
        let Some(correlation) = correlation else {
            self.uncorrelated_total = self.uncorrelated_total.saturating_add(1);
            return;
        };
        let Some(index) = self
            .in_flight
            .iter()
            .position(|entry| entry.correlation == correlation)
        else {
            self.uncorrelated_total = self.uncorrelated_total.saturating_add(1);
            return;
        };
        if let Some(entry) = self.in_flight.remove(index) {
            self.message_channel_ns
                .record(now_nanos.saturating_sub(entry.enqueued_nanos));
        }
    }

    pub(crate) fn record_message_render(&mut self, nanos: u64) {
        self.message_render_ns.record(nanos);
    }

    pub(crate) fn record_frame_tick(&mut self, interval_ns: u64, jitter_ns: u64) {
        self.frame_interval_ns.record(interval_ns);
        self.frame_jitter_ns.record(jitter_ns);
    }

    pub(crate) fn record_render(&mut self, nanos: u64) {
        self.render_ns.record(nanos);
    }

    pub(crate) fn record_write(&mut self, nanos: u64) {
        self.write_ns.record(nanos);
    }

    pub(crate) fn queue_depth(&self) -> u64 {
        self.enqueued_total.saturating_sub(self.dequeued_total)
    }

    pub(crate) fn snapshot(&self, now_nanos: u64) -> SummarySnapshot {
        SummarySnapshot {
            received_total: self.received_total,
            enqueued_total: self.enqueued_total,
            dequeued_total: self.dequeued_total,
            uncorrelated_total: self.uncorrelated_total,
            full_retry_total: self.full_retry_total,
            worker_retained_total: self.worker_retained_total,
            reconnect_count: self.reconnect_count,
            reconnect_backoff_total_ms: self.reconnect_backoff_total_ms,
            in_flight_high_water: self.in_flight_high_water,
            queue_depth_estimate: self.queue_depth(),
            oldest_in_flight_age_ns: self.oldest_in_flight_age(now_nanos),
            message_channel_ns: self.message_channel_ns.snapshot(),
            message_render_ns: self.message_render_ns.snapshot(),
            frame_interval_ns: self.frame_interval_ns.snapshot(),
            frame_jitter_ns: self.frame_jitter_ns.snapshot(),
            render_ns: self.render_ns.snapshot(),
            write_ns: self.write_ns.snapshot(),
        }
    }

    fn oldest_in_flight_age(&self, now_nanos: u64) -> Option<u64> {
        self.in_flight
            .front()
            .map(|entry| now_nanos.saturating_sub(entry.received_nanos))
    }

    fn evict_oldest_beyond_cap(&mut self) {
        while self.in_flight.len() > IN_FLIGHT_CAP {
            self.in_flight.pop_front();
        }
    }
}
