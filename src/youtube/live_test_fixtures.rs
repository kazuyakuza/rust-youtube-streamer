//! Shared offline test fixtures for the Phase-05 Task-2 runtime tests
//! (Amendment 1 ruling 1): the scripted [`ScriptedLiveApiTransport`] fake
//! plus the canned live-resource builders, `#[cfg(test)]`-gated exactly
//! like the `src/config/test_fixtures.rs` precedent. Every network
//! boundary is faked offline; no test ever reaches Google. Fixture ids /
//! names / urls are `*-fixture` shaped strings only — never real-shaped
//! credentials. The roster's original `use super::live_ingestion_tests::*`
//! import for the readiness-wait tests is superseded by this fixtures
//! path (`.kilo/plans/20261013-phase05-task2-lifecycle-amendment-1.md`).

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use serde_json::json;

use super::api_transport::LiveApiTransport;
use super::live_error::LiveApiError;
use super::live_models::{
    BindCommand, CreateBroadcastCommand, CreateStreamCommand, LiveBroadcast, LiveStream,
};
use super::wait_cancellation::WaitCanceller;

pub(super) const STREAM_ID_FIXTURE: &str = "stream-id-fixture";
pub(super) const BROADCAST_ID_FIXTURE: &str = "broadcast-id-fixture";
pub(super) const STREAM_NAME_FIXTURE: &str = "stream-name-fixture";
pub(super) const INGEST_BASE_FIXTURE: &str = "rtmps://ingest.example.test/ingest";

/// Scripted `LiveApiTransport` fake (roster B spec): per-operation queued
/// `Result`s, a recorded operation-label sequence, captured commands, and
/// the deterministic cancel-during-wait hook.
#[derive(Default)]
pub(super) struct ScriptedLiveApiTransport {
    pub(super) create_stream_results: RefCell<VecDeque<Result<LiveStream, LiveApiError>>>,
    pub(super) create_broadcast_results: RefCell<VecDeque<Result<LiveBroadcast, LiveApiError>>>,
    pub(super) bind_results: RefCell<VecDeque<Result<LiveBroadcast, LiveApiError>>>,
    pub(super) lookup_stream_results: RefCell<VecDeque<Result<Option<LiveStream>, LiveApiError>>>,
    pub(super) lookup_broadcast_results:
        RefCell<VecDeque<Result<Option<LiveBroadcast>, LiveApiError>>>,
    pub(super) operations: RefCell<Vec<&'static str>>,
    pub(super) captured_create_stream: RefCell<Option<CreateStreamCommand>>,
    pub(super) captured_create_broadcast: RefCell<Option<CreateBroadcastCommand>>,
    pub(super) captured_bind: RefCell<Option<BindCommand>>,
    pub(super) canceller_flip: RefCell<Option<WaitCanceller>>,
    pub(super) cancel_after_broadcast_lookups: Cell<u8>,
    broadcast_lookups: Cell<u8>,
}

impl ScriptedLiveApiTransport {
    fn flip_canceller_on_scripted_lookup(&self) {
        let observed_lookups = self.broadcast_lookups.get() + 1;
        self.broadcast_lookups.set(observed_lookups);
        if observed_lookups != self.cancel_after_broadcast_lookups.get() {
            return;
        }
        let flip = self.canceller_flip.borrow();
        if let Some(canceller) = flip.as_ref() {
            canceller.request_cancel();
        }
    }
}

impl LiveApiTransport for ScriptedLiveApiTransport {
    fn create_stream(&self, command: &CreateStreamCommand) -> Result<LiveStream, LiveApiError> {
        self.operations.borrow_mut().push("create_stream");
        *self.captured_create_stream.borrow_mut() = Some(copied_stream_command(command));
        next_result(&self.create_stream_results)
    }

    fn create_broadcast(
        &self,
        command: &CreateBroadcastCommand,
    ) -> Result<LiveBroadcast, LiveApiError> {
        self.operations.borrow_mut().push("create_broadcast");
        *self.captured_create_broadcast.borrow_mut() = Some(copied_broadcast_command(command));
        next_result(&self.create_broadcast_results)
    }

    fn bind(&self, command: &BindCommand) -> Result<LiveBroadcast, LiveApiError> {
        self.operations.borrow_mut().push("bind");
        *self.captured_bind.borrow_mut() = Some(BindCommand {
            broadcast_id: command.broadcast_id.clone(),
            stream_id: command.stream_id.clone(),
        });
        next_result(&self.bind_results)
    }

    fn lookup_stream(&self, _id: &str) -> Result<Option<LiveStream>, LiveApiError> {
        self.operations.borrow_mut().push("lookup_stream");
        next_result(&self.lookup_stream_results)
    }

    fn lookup_broadcast(&self, _id: &str) -> Result<Option<LiveBroadcast>, LiveApiError> {
        self.operations.borrow_mut().push("lookup_broadcast");
        self.flip_canceller_on_scripted_lookup();
        next_result(&self.lookup_broadcast_results)
    }
}

/// Push one scripted outcome onto a fake operation queue.
pub(super) fn queued<T>(queue: &RefCell<VecDeque<T>>, outcome: T) {
    queue.borrow_mut().push_back(outcome);
}

fn next_result<T>(queue: &RefCell<VecDeque<Result<T, LiveApiError>>>) -> Result<T, LiveApiError> {
    queue
        .borrow_mut()
        .pop_front()
        .expect("the test scripted a queued result")
}

fn copied_stream_command(command: &CreateStreamCommand) -> CreateStreamCommand {
    CreateStreamCommand {
        title: command.title.clone(),
        resolution: command.resolution.clone(),
        frame_rate: command.frame_rate.clone(),
    }
}

fn copied_broadcast_command(command: &CreateBroadcastCommand) -> CreateBroadcastCommand {
    CreateBroadcastCommand {
        title: command.title.clone(),
        description: command.description.clone(),
        privacy_status: command.privacy_status.clone(),
        scheduled_start_time: command.scheduled_start_time.clone(),
    }
}

/// O1 insert reply: id + the full `cdn.ingestionInfo` destination source.
pub(super) fn inserted_stream() -> LiveStream {
    stream_from_json(json!({
        "id": STREAM_ID_FIXTURE,
        "cdn": {"ingestionInfo": ingestion_info_json()}
    }))
}

/// O4 confirmation reply: id + `ready` status + the full cdn block.
pub(super) fn confirmed_ready_stream() -> LiveStream {
    stream_from_json(json!({
        "id": STREAM_ID_FIXTURE,
        "status": {"streamStatus": "ready"},
        "cdn": {"ingestionInfo": ingestion_info_json()}
    }))
}

/// The not-yet-ready (`created`) poll observation of both resources.
pub(super) fn not_ready_stream() -> LiveStream {
    stream_with_status("created")
}

pub(super) fn not_ready_broadcast() -> LiveBroadcast {
    broadcast_with_lifecycle("created")
}

pub(super) fn stream_with_status(label: &str) -> LiveStream {
    stream_from_json(json!({
        "id": STREAM_ID_FIXTURE,
        "status": {"streamStatus": label},
        "cdn": {"ingestionInfo": ingestion_info_json()}
    }))
}

/// O2 insert reply: the broadcast id only.
pub(super) fn inserted_broadcast() -> LiveBroadcast {
    broadcast_from_json(json!({"id": BROADCAST_ID_FIXTURE}))
}

/// O3 bind reply: the broadcast id + the bound stream id.
pub(super) fn bound_broadcast() -> LiveBroadcast {
    broadcast_from_json(json!({
        "id": BROADCAST_ID_FIXTURE,
        "contentDetails": {"boundStreamId": STREAM_ID_FIXTURE}
    }))
}

pub(super) fn broadcast_with_lifecycle(label: &str) -> LiveBroadcast {
    broadcast_from_json(json!({
        "id": BROADCAST_ID_FIXTURE,
        "status": {"lifeCycleStatus": label}
    }))
}

pub(super) fn stream_from_json(body: serde_json::Value) -> LiveStream {
    serde_json::from_value(body).expect("the fixture stream resource parses")
}

pub(super) fn broadcast_from_json(body: serde_json::Value) -> LiveBroadcast {
    serde_json::from_value(body).expect("the fixture broadcast resource parses")
}

fn ingestion_info_json() -> serde_json::Value {
    json!({
        "streamName": STREAM_NAME_FIXTURE,
        "rtmpsIngestionAddress": INGEST_BASE_FIXTURE
    })
}
