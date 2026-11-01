//! The offline latency/load harness: rational rate phases, the scenario
//! runner that drives the REAL bounded channel and frame loop on fake clocks,
//! and the round-trip accounting helper. Every value is derived from the
//! configured FPS and the channel capacity; nothing sleeps.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::app::diagnostics::diagnostics_test_fixtures::{
    enabled_collector_with_pacing, SummarySnapshot,
};
use crate::app::pipeline::{
    FakeClock, FramePipeline, PipelinePacing, PipelineParts, RecordingSink, StopFlag, StubRenderer,
    TEST_FRAME_BYTES,
};
use crate::app::run::chat_worker::runtime_chat_fakes::ChatFakes;
use crate::app::run::chat_worker::{chat_channel, CHAT_CHANNEL_CAPACITY};
use crate::chat::{ChatMessage, ChatStore, ChatStoreSettings};
use crate::config::{valid_config, AppConfig};

use self::latency_load_source::{PacedQueueProducerSource, ProducerParts, SharedProducer};

/// Rational messages-per-frame: integer credit accounting only.
pub(super) struct RateProfile {
    pub(super) numerator: u64,
    pub(super) denominator: u64,
}

impl RateProfile {
    pub(super) fn advance(&self, credit: &mut u64) -> u64 {
        *credit = credit.saturating_add(self.numerator);
        let released = *credit / self.denominator;
        *credit -= released * self.denominator;
        released
    }
}

pub(super) struct RatePhase {
    pub(super) frames: u64,
    pub(super) rate: RateProfile,
}

pub(super) struct LatencyScenario {
    pub(super) label: &'static str,
    pub(super) phases: Vec<RatePhase>,
}

impl LatencyScenario {
    pub(super) fn total_frames(&self) -> u64 {
        self.phases.iter().map(|phase| phase.frames).sum()
    }

    pub(super) fn due_total(&self) -> u64 {
        self.phases
            .iter()
            .map(|phase| phase.frames * phase.rate.numerator / phase.rate.denominator)
            .sum()
    }
}

pub(super) struct ScenarioOutcome {
    pub(super) total_frames: u64,
    pub(super) pacing_interval_nanos: u64,
    pub(super) snapshot: SummarySnapshot,
    pub(super) leftover: Vec<ChatMessage>,
    pub(super) _kept_temp: tempfile::TempDir,
}

pub(super) fn run_scenario(scenario: LatencyScenario) -> ScenarioOutcome {
    let config = valid_config();
    let pacing = PipelinePacing::from_config(&config.video).expect("validated FPS derives pacing");
    let pacing_interval_nanos = pacing.interval_nanos();
    let temp = tempfile::Builder::new()
        .prefix(scenario.label)
        .tempdir()
        .expect("the scenario temp dir exists");
    let chat_store = open_log_store(&config, temp.path());
    let harness = enabled_collector_with_pacing(pacing_interval_nanos);
    let snapshotter = harness.clone();
    let stop = StopFlag::new();
    let total_frames = scenario.total_frames();
    let channel = chat_channel();
    let producer = Rc::new(RefCell::new(PacedQueueProducerSource::new(ProducerParts {
        channel,
        fakes: ChatFakes::new(),
        collector: harness,
        scenario,
        pacing_interval_nanos,
        stop_flag: stop.clone(),
    })));
    let sink = RecordingSink::fresh()
        .0
        .stop_after_writes(total_frames as usize, stop.clone());
    let pipeline = FramePipeline::new(PipelineParts {
        chat_store,
        renderer: Box::new(StubRenderer::fresh().0),
        sink,
        source: Box::new(SharedProducer::new(Rc::clone(&producer))),
        clock: Box::new(FakeClock::fresh().0),
        pacing: PipelinePacing::raw(pacing_interval_nanos, TEST_FRAME_BYTES),
        stop_flag: stop,
        diagnostics: Some(snapshotter.collector.clone()),
    });
    pipeline.run().expect("the offline scenario run completes");
    let leftover = producer.borrow_mut().drain_leftover();
    ScenarioOutcome {
        total_frames,
        pacing_interval_nanos,
        snapshot: snapshotter.snapshot(),
        leftover,
        _kept_temp: temp,
    }
}

pub(super) fn assert_full_round_trip_accounting(due_total: u64, run: &ScenarioOutcome) {
    let snapshot = &run.snapshot;
    assert_eq!(
        snapshot.enqueued_total, snapshot.received_total,
        "nothing is retained at the sender in-harness"
    );
    assert_eq!(
        due_total, snapshot.received_total,
        "no silently lost message"
    );
    assert_eq!(
        snapshot.received_total,
        snapshot.dequeued_total + run.leftover.len() as u64,
        "received = dequeued + still queued"
    );
    assert!(
        run.leftover.is_empty(),
        "the drain tail empties the channel"
    );
    assert!(
        snapshot.in_flight_high_water <= CHAT_CHANNEL_CAPACITY as u64,
        "memory stays bounded by the channel capacity"
    );
}

fn open_log_store(config: &AppConfig, directory: &Path) -> ChatStore {
    let mut log_config = config.clone();
    log_config.chat.log_file = directory.join("chat.log").to_string_lossy().into_owned();
    let settings = ChatStoreSettings::from_config(&log_config)
        .expect("validated geometry derives the visible capacity");
    ChatStore::new(settings).expect("the scenario chat log opens")
}

#[path = "latency_load_source.rs"]
mod latency_load_source;

#[cfg(test)]
#[path = "latency_load_tests.rs"]
mod latency_load_tests;

#[cfg(test)]
#[path = "latency_load_loss_tests.rs"]
mod latency_load_loss_tests;
