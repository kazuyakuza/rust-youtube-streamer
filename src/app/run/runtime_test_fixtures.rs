//! Offline test doubles for the run runtime (plan D-T3-9; the
//! `youtube::live_test_fixtures` / `config::test_fixtures` precedent, but
//! defined IN `app::run` because the youtube-side fixture is
//! `pub(super)` to `youtube`). Everything is `Mutex`-based (never
//! `RefCell`) because the confirmation gate thread requires
//! `Send + Sync`. Fixture material stays `-fixture` shaped
//! (`rtmps://ingest.example.test/...`, `stream-name-fixture`,
//! `access-token-fixture`) — NEVER real-shaped, never reaching Google.
//! Splits (D-T3-14 line cap): the streaming-runner fake lives in
//! `runtime_runner_fixture`, the credential fakes in
//! `runtime_credential_fakes`.

use std::collections::VecDeque;
use std::sync::Mutex;

use serde_json::json;

use crate::youtube::{
    BindCommand, CreateBroadcastCommand, CreateStreamCommand, LiveApiError, LiveApiTransport,
    LiveBroadcast, LiveStream,
};

pub(super) const STREAM_ID_FIXTURE: &str = "stream-id-fixture";
pub(super) const BROADCAST_ID_FIXTURE: &str = "broadcast-id-fixture";
const INGEST_BASE_FIXTURE: &str = "rtmps://ingest.example.test/ingest";
const STREAM_NAME_FIXTURE: &str = "stream-name-fixture";
pub(super) const DESTINATION_URL_FIXTURE: &str =
    "rtmps://ingest.example.test/ingest/stream-name-fixture";

type StreamOutcome = Result<LiveStream, LiveApiError>;
type BroadcastOutcome = Result<LiveBroadcast, LiveApiError>;
type StreamLookup = Result<Option<LiveStream>, LiveApiError>;
type BroadcastLookup = Result<Option<LiveBroadcast>, LiveApiError>;

/// Scripted `LiveApiTransport` fake: per-operation queued results and a
/// recorded operation-label sequence.
#[derive(Default)]
pub(super) struct ScriptedTransport {
    create_stream_results: Mutex<VecDeque<StreamOutcome>>,
    create_broadcast_results: Mutex<VecDeque<BroadcastOutcome>>,
    bind_results: Mutex<VecDeque<BroadcastOutcome>>,
    lookup_stream_results: Mutex<VecDeque<StreamLookup>>,
    lookup_broadcast_results: Mutex<VecDeque<BroadcastLookup>>,
    operations: Mutex<Vec<&'static str>>,
}

impl ScriptedTransport {
    /// The TD-1/TD-4 happy preparation script (O1→O2→O3→readiness, each
    /// exactly once) plus the scripted post-start gate lookup.
    pub(super) fn script_preparation_then_gate(&self, gate_lookup: StreamLookup) {
        self.push(&self.create_stream_results, Ok(inserted_stream()));
        self.push(&self.create_broadcast_results, Ok(inserted_broadcast()));
        self.push(&self.bind_results, Ok(bound_broadcast()));
        self.push(
            &self.lookup_stream_results,
            Ok(Some(confirmed_ready_stream())),
        );
        self.push(
            &self.lookup_broadcast_results,
            Ok(Some(broadcast_with_lifecycle("ready"))),
        );
        self.push(&self.lookup_stream_results, gate_lookup);
    }

    /// The TD-3 script: only the O1 outcome is queued, and it FAILS with
    /// the typed not-found class.
    pub(super) fn script_create_stream_failure(&self) {
        self.push(
            &self.create_stream_results,
            Err(LiveApiError::ApiNotFound {
                resource: "liveStream".to_string(),
            }),
        );
    }

    /// Gate-only script (TB roster): queues ONLY the confirmation-wait
    /// lookup, so no preparation cycle runs and no poll interval sleeps.
    pub(super) fn script_gate_lookup(&self, gate_lookup: StreamLookup) {
        self.push(&self.lookup_stream_results, gate_lookup);
    }

    pub(super) fn operations(&self) -> Vec<&'static str> {
        self.operations.lock().expect("fixture mutex").clone()
    }

    fn push<T>(&self, queue: &Mutex<VecDeque<T>>, outcome: T) {
        queue.lock().expect("fixture mutex").push_back(outcome);
    }

    fn record(&self, label: &'static str) {
        self.operations.lock().expect("fixture mutex").push(label);
    }
}

impl LiveApiTransport for ScriptedTransport {
    fn create_stream(&self, _command: &CreateStreamCommand) -> Result<LiveStream, LiveApiError> {
        self.record("create_stream");
        pop(&self.create_stream_results)
    }

    fn create_broadcast(
        &self,
        _command: &CreateBroadcastCommand,
    ) -> Result<LiveBroadcast, LiveApiError> {
        self.record("create_broadcast");
        pop(&self.create_broadcast_results)
    }

    fn bind(&self, _command: &BindCommand) -> Result<LiveBroadcast, LiveApiError> {
        self.record("bind");
        pop(&self.bind_results)
    }

    fn lookup_stream(&self, _id: &str) -> Result<Option<LiveStream>, LiveApiError> {
        self.record("lookup_stream");
        pop(&self.lookup_stream_results)
    }

    fn lookup_broadcast(&self, _id: &str) -> Result<Option<LiveBroadcast>, LiveApiError> {
        self.record("lookup_broadcast");
        pop(&self.lookup_broadcast_results)
    }
}

fn pop<T>(queue: &Mutex<VecDeque<Result<T, LiveApiError>>>) -> Result<T, LiveApiError> {
    queue
        .lock()
        .expect("fixture mutex")
        .pop_front()
        .expect("the test scripted a queued result")
}

pub(super) fn stream_with_status(label: &str) -> LiveStream {
    stream_from_json(json!({
        "id": STREAM_ID_FIXTURE,
        "status": {"streamStatus": label},
        "cdn": {"ingestionInfo": ingestion_info()}
    }))
}

fn confirmed_ready_stream() -> LiveStream {
    stream_with_status("ready")
}

fn inserted_stream() -> LiveStream {
    stream_from_json(json!({
        "id": STREAM_ID_FIXTURE,
        "cdn": {"ingestionInfo": ingestion_info()}
    }))
}

fn inserted_broadcast() -> LiveBroadcast {
    broadcast_from_json(json!({"id": BROADCAST_ID_FIXTURE}))
}

fn bound_broadcast() -> LiveBroadcast {
    broadcast_from_json(json!({
        "id": BROADCAST_ID_FIXTURE,
        "contentDetails": {"boundStreamId": STREAM_ID_FIXTURE}
    }))
}

fn broadcast_with_lifecycle(label: &str) -> LiveBroadcast {
    broadcast_from_json(json!({
        "id": BROADCAST_ID_FIXTURE,
        "status": {"lifeCycleStatus": label}
    }))
}

fn stream_from_json(body: serde_json::Value) -> LiveStream {
    serde_json::from_value(body).expect("the fixture stream resource parses")
}

fn broadcast_from_json(body: serde_json::Value) -> LiveBroadcast {
    serde_json::from_value(body).expect("the fixture broadcast resource parses")
}

fn ingestion_info() -> serde_json::Value {
    json!({
        "streamName": STREAM_NAME_FIXTURE,
        "rtmpsIngestionAddress": INGEST_BASE_FIXTURE
    })
}
