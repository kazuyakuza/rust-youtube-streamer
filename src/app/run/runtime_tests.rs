//! Orchestration tests (plan §8 TD): `run_runtime` produces the §5
//! artifact-of-startup sequence exactly, the permission gate blocks ALL
//! resource creation, broadcast failures never reach streaming, and the
//! shutdown order matches production. NOTE (documented drift risk, plan
//! §8 TD-5): the recorded runner MIRRORS `PreparedStreaming::run`'s
//! shutdown sequence — update it here if the production ordering ever
//! changes.

use std::sync::Arc;

use crate::app::pipeline::StopFlag;
use crate::config::valid_config;
use crate::youtube::{LiveApiError, WaitCanceller};

use super::broadcast::public_watch_url;
use super::error::RunError;
use super::run_runtime;
use super::runtime_credential_fakes::{
    fixture_client_config, FakeEndpoint, FakeStore, FakeVerifier,
};
use super::runtime_runner_fixture::{RecordingStreamingRunner, RunnerScript};
use super::runtime_test_fixtures::{
    stream_with_status, ScriptedTransport, BROADCAST_ID_FIXTURE, STREAM_ID_FIXTURE,
};
use super::seams::{RunStops, RuntimeStart, ServiceStops};

fn run_with_scripted(
    transport: &Arc<ScriptedTransport>,
    verifier: &FakeVerifier,
    runner: &RecordingStreamingRunner,
) -> Result<(), RunError> {
    let client_config = fixture_client_config();
    let store = FakeStore::granting();
    let endpoint = FakeEndpoint;
    let shared = Arc::clone(transport);
    let start = RuntimeStart {
        config: &valid_config(),
        stops: RunStops {
            stop_flag: StopFlag::new(),
            canceller: WaitCanceller::new(),
        },
        service: ServiceStops {
            client_config: &client_config,
            store: &store,
            endpoint: &endpoint,
            verifier,
        },
        build_transport: Box::new(move |_access_token| shared),
    };
    run_runtime(start, runner)
}

#[test]
fn run_runtime_orchestrates_credentials_then_resources_then_streaming() {
    let transport = Arc::new(ScriptedTransport::default());
    transport.script_preparation_then_gate(Ok(Some(stream_with_status("active"))));
    let runner = RecordingStreamingRunner::scripted(RunnerScript::ReportOk);

    run_with_scripted(
        &transport,
        &FakeVerifier::granting_required_scope(),
        &runner,
    )
    .expect("the happy runtime script completes");

    assert_eq!(
        transport.operations(),
        [
            "create_stream",
            "create_broadcast",
            "bind",
            "lookup_stream",
            "lookup_broadcast",
            "lookup_stream"
        ],
        "one prepare cycle then the O4 gate lookup; the credential probes issue no transport op"
    );
    let recorded = runner.take_recorded();
    assert_eq!(recorded.destination_debug, "RtmpsDestination(<redacted>)");
    assert_eq!(recorded.stream_id, STREAM_ID_FIXTURE);
    assert_eq!(recorded.watch_url, public_watch_url(BROADCAST_ID_FIXTURE));
    assert_eq!(recorded.gate_finish, "confirmed");
}

#[test]
fn scope_gate_failure_prevents_resource_creation() {
    let transport = Arc::new(ScriptedTransport::default());
    let runner = RecordingStreamingRunner::scripted(RunnerScript::ReportOk);

    let error = run_with_scripted(&transport, &FakeVerifier::lacking_required_scope(), &runner)
        .expect_err("fresh evidence lacking the scope must stop the run");

    assert!(matches!(error, RunError::InsufficientScope));
    assert!(transport.operations().is_empty());
    assert!(!runner.was_invoked());
}

#[test]
fn broadcast_failure_maps_to_run_error_without_streaming() {
    let transport = Arc::new(ScriptedTransport::default());
    transport.script_create_stream_failure();
    let runner = RecordingStreamingRunner::scripted(RunnerScript::ReportOk);

    let error = run_with_scripted(
        &transport,
        &FakeVerifier::granting_required_scope(),
        &runner,
    )
    .expect_err("an O1 failure must stop the run");

    assert!(matches!(
        error,
        RunError::Broadcast(LiveApiError::ApiNotFound { .. })
    ));
    assert_eq!(transport.operations(), ["create_stream"]);
    assert!(!runner.was_invoked());
}

#[test]
fn gate_fatal_stops_streaming_and_primarily_reports_the_wait() {
    let transport = Arc::new(ScriptedTransport::default());
    transport.script_preparation_then_gate(Ok(Some(stream_with_status("error"))));
    let runner = RecordingStreamingRunner::scripted(RunnerScript::MirrorProductionShutdown);

    let error = run_with_scripted(
        &transport,
        &FakeVerifier::granting_required_scope(),
        &runner,
    )
    .expect_err("a gate fatal must be the primary reported error");

    assert!(matches!(
        error,
        RunError::Broadcast(LiveApiError::BroadcastState { .. })
    ));
    let recorded = runner.take_recorded();
    assert_eq!(recorded.gate_finish, "fatal");
    assert_eq!(recorded.destination_debug, "RtmpsDestination(<redacted>)");
}

#[test]
fn shutdown_order_cancels_the_wait_before_joining_the_gate() {
    let transport = Arc::new(ScriptedTransport::default());
    transport.script_preparation_then_gate(Err(LiveApiError::WaitCancelled));
    let runner =
        RecordingStreamingRunner::scripted(RunnerScript::PipelineFailureThenProductionOrder);

    let error = run_with_scripted(
        &transport,
        &FakeVerifier::granting_required_scope(),
        &runner,
    )
    .expect_err("the runner-recorded pipeline failure is the primary");

    assert!(matches!(error, RunError::Pipeline(_)));
    let recorded = runner.take_recorded();
    assert_eq!(recorded.gate_finish, "cancelled");
}
