//! Shared diagnostics test support: a fixed monotonic clock, a config
//! carrying every capability, and a harness exposing the aggregate state so
//! deterministic tests can assert stage propagation without sleeps.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::config::diagnostics_config::DiagnosticsConfig;

use super::collector::{DiagnosticsCollector, MonotonicNanos};
use super::collector_stages::CollectorSettings;
pub(crate) use super::events::SummarySnapshot;
use super::state::DiagnosticsState;

const TEST_PACING_INTERVAL_NANOS: u64 = 40;

/// Deterministic monotonic clock: the test owns and advances the shared cell.
struct FixedMonotonicNanos {
    shared_cell: Arc<AtomicU64>,
}

impl FixedMonotonicNanos {
    fn new() -> (Self, Arc<AtomicU64>) {
        let shared_cell = Arc::new(AtomicU64::new(0));
        (
            Self {
                shared_cell: Arc::clone(&shared_cell),
            },
            shared_cell,
        )
    }
}

impl MonotonicNanos for FixedMonotonicNanos {
    fn now_nanos(&self) -> u64 {
        self.shared_cell.load(Ordering::SeqCst)
    }
}

#[derive(Clone)]
pub(crate) struct TestCollector {
    pub(crate) collector: DiagnosticsCollector,
    state: Arc<Mutex<DiagnosticsState>>,
    clock: Arc<AtomicU64>,
}

impl TestCollector {
    pub(crate) fn advance(&self, nanos: u64) {
        self.clock.fetch_add(nanos, Ordering::SeqCst);
    }

    pub(crate) fn snapshot(&self) -> SummarySnapshot {
        self.state
            .lock()
            .expect("diagnostics state")
            .snapshot(self.now())
    }

    fn now(&self) -> u64 {
        self.clock.load(Ordering::SeqCst)
    }
}

pub(crate) fn enabled_collector_for_tests() -> TestCollector {
    test_collector(&enabled_diagnostics(None), TEST_PACING_INTERVAL_NANOS)
}

pub(crate) fn enabled_collector_with_pacing(pacing_interval_nanos: u64) -> TestCollector {
    test_collector(&enabled_diagnostics(None), pacing_interval_nanos)
}

pub(crate) fn collector_with_output_file(path: &Path) -> TestCollector {
    let output_file = Some(path.to_string_lossy().into_owned());
    test_collector(
        &enabled_diagnostics(output_file),
        TEST_PACING_INTERVAL_NANOS,
    )
}

pub(super) fn diagnostics_with_capabilities(
    pipeline_timing: bool,
    queue_metrics: bool,
    periodic_summary: bool,
) -> DiagnosticsConfig {
    DiagnosticsConfig {
        pipeline_timing_events: Some(pipeline_timing),
        queue_metrics: Some(queue_metrics),
        periodic_summary: Some(periodic_summary),
        summary_interval_seconds: Some(1),
        output_file: None,
        max_file_bytes: Some(1_048_576),
        max_generations: Some(3),
    }
}

pub(super) fn collector_for_config(diagnostics: &DiagnosticsConfig) -> TestCollector {
    test_collector(diagnostics, TEST_PACING_INTERVAL_NANOS)
}

fn enabled_diagnostics(output_file: Option<String>) -> DiagnosticsConfig {
    DiagnosticsConfig {
        pipeline_timing_events: Some(true),
        queue_metrics: Some(true),
        periodic_summary: Some(true),
        summary_interval_seconds: Some(1),
        output_file,
        max_file_bytes: Some(1_048_576),
        max_generations: Some(3),
    }
}

fn test_collector(diagnostics: &DiagnosticsConfig, pacing_interval_nanos: u64) -> TestCollector {
    let settings = CollectorSettings::from_diagnostics(diagnostics, pacing_interval_nanos);
    let (clock, clock_cell) = FixedMonotonicNanos::new();
    let mut collector = DiagnosticsCollector::from_settings(settings);
    collector.clock = Arc::new(clock);
    let state = Arc::clone(&collector.state);
    TestCollector {
        collector,
        state,
        clock: clock_cell,
    }
}
