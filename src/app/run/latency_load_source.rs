//! The paced queue producer of the offline latency harness: each frame step
//! advances the diagnostics fake clock, pulls through the REAL `ChatSource`,
//! releases the rational rate into a sender-side backlog, and injects
//! capacity-guarded carriers through the REAL delivery path. Single-threaded,
//! zero sleeps, zero network.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::app::diagnostics::diagnostics_test_fixtures::TestCollector;
use crate::app::pipeline::{ChatSourceError, MessageSource, StopFlag};
use crate::app::run::chat_stream_worker::StreamWorker;
use crate::app::run::chat_worker::runtime_chat_fakes::{
    worker_plan, ChatFakes, LIVE_CHAT_ID_FIXTURE,
};
use crate::app::run::chat_worker::{ChatChannel, CHAT_CHANNEL_CAPACITY};
use crate::app::run::source::ChatSource;
use crate::chat::ChatMessage;
use crate::youtube::{
    response_plain, response_with_token, text_item, yielded, RESUME_TOKEN_FIXTURE,
};

use super::{LatencyScenario, RatePhase};

/// The constructor bundle for [`PacedQueueProducerSource`] (≤2-args rule).
pub(super) struct ProducerParts {
    pub(super) channel: ChatChannel,
    pub(super) fakes: ChatFakes,
    pub(super) collector: TestCollector,
    pub(super) scenario: LatencyScenario,
    pub(super) pacing_interval_nanos: u64,
    pub(super) stop_flag: StopFlag,
}

/// One real-lane producer: consumes from the real bounded channel and injects
/// through `StreamWorker::process_batch`, retaining surplus in a FIFO backlog
/// while the channel is full (the frame-granularity retry model).
pub(super) struct PacedQueueProducerSource {
    delegate: ChatSource,
    worker: StreamWorker,
    phases: Vec<RatePhase>,
    phase_index: usize,
    frames_in_phase: u64,
    credit: u64,
    pending: VecDeque<String>,
    next_id: u64,
    window_opened: bool,
    collector: TestCollector,
    pacing_interval_nanos: u64,
    injected_total: u64,
    pulled_total: u64,
}

impl PacedQueueProducerSource {
    pub(super) fn new(parts: ProducerParts) -> Self {
        let mut plan = worker_plan(&parts.fakes, &parts.channel);
        plan.diagnostics = Some(parts.collector.collector.clone());
        plan.stop_flag = parts.stop_flag;
        let worker = StreamWorker::new(plan, LIVE_CHAT_ID_FIXTURE.to_string());
        let channel = parts.channel;
        let delegate = ChatSource::new(channel.receiver, channel.terminal);
        Self {
            delegate,
            worker,
            phases: parts.scenario.phases,
            phase_index: 0,
            frames_in_phase: 0,
            credit: 0,
            pending: VecDeque::new(),
            next_id: 0,
            window_opened: false,
            collector: parts.collector,
            pacing_interval_nanos: parts.pacing_interval_nanos,
            injected_total: 0,
            pulled_total: 0,
        }
    }

    /// Drains what the bounded channel still holds after the run stopped.
    pub(super) fn drain_leftover(&mut self) -> Vec<ChatMessage> {
        let mut leftover = Vec::new();
        while let Ok(Some(message)) = self.delegate.next_message() {
            leftover.push(message);
        }
        leftover
    }

    fn step(&mut self) -> Result<Option<ChatMessage>, ChatSourceError> {
        self.collector.advance(self.pacing_interval_nanos);
        let pulled = self.delegate.next_message()?;
        if pulled.is_some() {
            self.pulled_total += 1;
        }
        self.open_window_once();
        let released = self.release_count();
        self.enqueue_released(released);
        self.inject_pending();
        Ok(pulled)
    }

    fn open_window_once(&mut self) {
        if self.window_opened {
            return;
        }
        self.window_opened = true;
        self.worker.process_batch(yielded(&response_with_token(
            Vec::new(),
            RESUME_TOKEN_FIXTURE,
        )));
    }

    fn release_count(&mut self) -> u64 {
        loop {
            let Some(phase) = self.phases.get(self.phase_index) else {
                return 0;
            };
            if self.frames_in_phase < phase.frames {
                self.frames_in_phase += 1;
                return phase.rate.advance(&mut self.credit);
            }
            self.phase_index += 1;
            self.frames_in_phase = 0;
        }
    }

    fn enqueue_released(&mut self, released: u64) {
        for _ in 0..released {
            let id = format!("latency-m-{}", self.next_id);
            self.next_id += 1;
            self.pending.push_back(id);
        }
    }

    fn inject_pending(&mut self) {
        let in_channel = self.injected_total - self.pulled_total;
        let free = CHAT_CHANNEL_CAPACITY as u64 - in_channel;
        let injectable = free.min(self.pending.len() as u64);
        if injectable == 0 {
            return;
        }
        let carriers: Vec<_> = self
            .pending
            .drain(..injectable as usize)
            .map(|id| text_item(&id))
            .collect();
        self.worker
            .process_batch(yielded(&response_plain(carriers)));
        self.injected_total += injectable;
    }
}

/// Shares the producer with the pipeline while the scenario keeps its own
/// handle for the post-stop leftover drain.
pub(super) struct SharedProducer {
    inner: Rc<RefCell<PacedQueueProducerSource>>,
}

impl SharedProducer {
    pub(super) fn new(inner: Rc<RefCell<PacedQueueProducerSource>>) -> Self {
        Self { inner }
    }
}

impl MessageSource for SharedProducer {
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatSourceError> {
        self.inner.borrow_mut().step()
    }
}
