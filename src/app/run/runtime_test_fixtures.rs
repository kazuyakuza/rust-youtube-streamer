//! Offline test doubles for the run runtime (plan D-T3-9; the
//! `youtube::live_test_fixtures` / `config::test_fixtures` precedent, but
//! defined IN `app::run` because the youtube-side fixture is
//! `pub(super)` to `youtube`). Everything is `Mutex`-based (never
//! `RefCell`) because the confirmation gate thread requires
//! `Send + Sync`. Fixture material stays `-fixture` shaped
//! (`rtmps://ingest.example.test/...`, `stream-name-fixture`,
//! `access-token-fixture`) — NEVER real-shaped, never reaching Google.
//! Split (D-T3-14 line cap): the streaming-runner fake lives in the
//! `runtime_runner_fixture` sibling.

use std::collections::VecDeque;
use std::sync::Mutex;

use serde_json::json;

use crate::youtube::{
    parse_client_config, AccessToken, AuthorizationVerifier, BindCommand, ClientConfig,
    CodeExchangeRequest, CreateBroadcastCommand, CreateStreamCommand, CredentialStore,
    CredentialStoreError, GrantedScopes, LiveApiError, LiveApiTransport, LiveBroadcast, LiveStream,
    RefreshRequest, StoredCredentials, TokenEndpointError, TokenEndpointTransport,
    TokenExchangeResult, VerificationError, VerifiedAuthorization, REQUIRED_SCOPE,
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

/// Credential-store fake: grants `REQUIRED_SCOPE`; `absent()` fails the
/// load with the typed no-credentials class.
pub(super) struct FakeStore {
    fail_load: bool,
}

impl FakeStore {
    pub(super) fn granting() -> Self {
        Self { fail_load: false }
    }

    pub(super) fn absent() -> Self {
        Self { fail_load: true }
    }
}

impl CredentialStore for FakeStore {
    fn load(&self) -> Result<StoredCredentials, CredentialStoreError> {
        if self.fail_load {
            return Err(CredentialStoreError::ContentNotAvailable);
        }
        Ok(granted_stored_credentials())
    }

    fn save(&self, _credentials: &StoredCredentials) -> Result<(), CredentialStoreError> {
        Ok(())
    }
}

fn granted_stored_credentials() -> StoredCredentials {
    StoredCredentials {
        access_token: AccessToken::new("access-token-fixture"),
        expires_in_seconds: 3600,
        refresh_token: None,
        granted_scopes: GrantedScopes::from_space_delimited(REQUIRED_SCOPE),
    }
}

/// Token-endpoint fake: the happy paths never refresh, so both exchanges
/// answer with the unreadable-response class.
pub(super) struct FakeEndpoint;

impl TokenEndpointTransport for FakeEndpoint {
    fn authorization_code_exchange(&self, _request: &CodeExchangeRequest) -> TokenExchangeResult {
        Err(TokenEndpointError::ResponseParse)
    }

    fn refresh_token_exchange(&self, _request: &RefreshRequest) -> TokenExchangeResult {
        Err(TokenEndpointError::ResponseParse)
    }
}

/// Verifier fake returning scripted scope evidence.
pub(super) struct FakeVerifier {
    evidence: &'static str,
}

impl FakeVerifier {
    pub(super) fn granting_required_scope() -> Self {
        Self {
            evidence: REQUIRED_SCOPE,
        }
    }

    pub(super) fn lacking_required_scope() -> Self {
        Self {
            evidence: "other-scope-fixture",
        }
    }
}

impl AuthorizationVerifier for FakeVerifier {
    fn verify(
        &self,
        _access_token: &AccessToken,
    ) -> Result<VerifiedAuthorization, VerificationError> {
        Ok(VerifiedAuthorization::new(
            GrantedScopes::from_space_delimited(self.evidence),
        ))
    }
}

/// A minimal parsed client config (endpoints accepted by the allowlist).
pub(super) fn fixture_client_config() -> ClientConfig {
    parse_client_config(
        r#"{"installed": {
            "client_id": "client-id-fixture",
            "client_secret": "client-secret-fixture",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "token_uri": "https://oauth2.googleapis.com/token"
        }}"#,
    )
    .expect("the fixture client config parses")
}
