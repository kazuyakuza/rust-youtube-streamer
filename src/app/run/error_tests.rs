//! Run-error mapping tests (plan §8 TA): every run-origin variant maps to
//! the startup-failure code, the pre-streaming WaitCancelled class maps to
//! success, the Display lines stay stable and secret-free, and the
//! primary-shutdown selector follows the deterministic matrix. Doc
//! contract (`run_command_is_never_named_by_tests`, plan §8 TA-5): the
//! ctrlc handler is process-global, so `run_command`/`run_outcome` are
//! exercised ONLY through the §8 TD seams — never directly by tests.

use std::io;

use crate::app::pipeline::{ChatRetrievalError, FramePipelineError, InterruptHandlerError};
use crate::app::run::error::RunError;
use crate::app::run::gate::GateFinish;
use crate::app::run::streaming::primary_shutdown_outcome;
use crate::app::run::RUN_STARTUP_STOP_MESSAGE;
use crate::streaming::StreamingError;
use crate::youtube::{
    ClientConfigError, CredentialServiceError, LiveApiError, TokenEndpointError, REQUIRED_SCOPE,
};

fn all_run_errors() -> Vec<RunError> {
    vec![
        RunError::Interrupt(InterruptHandlerError::test_failure("already taken")),
        RunError::ClientConfig(ClientConfigError::MissingField {
            field: "installed.client_id".to_string(),
        }),
        RunError::EndpointInit(TokenEndpointError::ResponseParse),
        RunError::Credential(CredentialServiceError::Absent),
        RunError::InsufficientScope,
        RunError::Broadcast(LiveApiError::LiveStreamingNotEnabled),
        RunError::Pipeline(pipeline_failure()),
        RunError::GateThreadStopped,
        RunError::ChatRetrieval(ChatRetrievalError::Transport {
            detail: "fixture".to_string(),
        }),
    ]
}

fn pipeline_failure() -> FramePipelineError {
    FramePipelineError::Sink(StreamingError::WaitFailed {
        source: io::Error::other("wait fixture"),
    })
}

fn gate_fatal() -> LiveApiError {
    LiveApiError::BroadcastState {
        detail: "streamStatus error".to_string(),
    }
}

#[test]
fn run_origin_failures_map_to_the_startup_failure_code() {
    for error in all_run_errors() {
        assert_eq!(error.exit_code(), 1, "exit code for: {error}");
    }
}

#[test]
fn wait_cancelled_maps_to_success_exit_code() {
    let error = RunError::Broadcast(LiveApiError::WaitCancelled);
    assert_eq!(error.exit_code(), 0);
    assert!(error.is_stop_request_cancellation());
    assert_eq!(
        RUN_STARTUP_STOP_MESSAGE,
        "run stopped by the stop request before streaming started"
    );
}

#[test]
fn run_error_display_lines_are_stable_and_secret_free() {
    let insufficient_scope = RunError::InsufficientScope.to_string();
    assert!(insufficient_scope.contains(REQUIRED_SCOPE));
    assert!(insufficient_scope.contains("run the authorization command again"));
    let gate_stopped = RunError::GateThreadStopped.to_string();
    assert!(gate_stopped.contains("confirmation thread stopped"));
    let credential = RunError::Credential(CredentialServiceError::Absent).to_string();
    assert_eq!(
        credential,
        "no stored credentials are available; run the authorization command first"
    );
    for error in all_run_errors() {
        let rendered = error.to_string();
        assert!(!rendered.contains("rtmps"), "no RTMPS text in: {rendered}");
        assert!(!rendered.contains("Bearer"), "no Bearer in: {rendered}");
        assert!(
            !rendered.contains("stream-key"),
            "no key material in: {rendered}"
        );
    }
}

#[test]
fn primary_shutdown_outcome_matrix_is_deterministic() {
    let broadcast_fatal =
        primary_shutdown_outcome(Err(pipeline_failure()), GateFinish::Fatal(gate_fatal()));
    assert!(matches!(
        broadcast_fatal,
        Err(RunError::Broadcast(LiveApiError::BroadcastState { .. }))
    ));
    let pipeline_primary = primary_shutdown_outcome(Err(pipeline_failure()), GateFinish::Cancelled);
    assert!(matches!(pipeline_primary, Err(RunError::Pipeline(_))));
    let pipeline_outranks_panic =
        primary_shutdown_outcome(Err(pipeline_failure()), GateFinish::Panicked);
    assert!(matches!(
        pipeline_outranks_panic,
        Err(RunError::Pipeline(_))
    ));
    let fatal_after_clean_stop = primary_shutdown_outcome(Ok(()), GateFinish::Fatal(gate_fatal()));
    assert!(matches!(
        fatal_after_clean_stop,
        Err(RunError::Broadcast(LiveApiError::BroadcastState { .. }))
    ));
    let panicked_after_clean_stop = primary_shutdown_outcome(Ok(()), GateFinish::Panicked);
    assert!(matches!(
        panicked_after_clean_stop,
        Err(RunError::GateThreadStopped)
    ));
    assert!(primary_shutdown_outcome(Ok(()), GateFinish::Confirmed).is_ok());
    assert!(primary_shutdown_outcome(Ok(()), GateFinish::Cancelled).is_ok());
}

#[test]
fn chat_retrieval_failure_maps_to_the_chat_funnel_without_masking_the_gate() {
    let rendered = RunError::ChatRetrieval(ChatRetrievalError::Transport {
        detail: "fixture".to_string(),
    })
    .to_string();
    assert_eq!(
        rendered,
        "live chat transport failed after repeated attempts: fixture"
    );

    let chat_retrieval = || {
        FramePipelineError::ChatRetrieval(ChatRetrievalError::Transport {
            detail: "fixture".to_string(),
        })
    };
    let gate_outranks =
        primary_shutdown_outcome(Err(chat_retrieval()), GateFinish::Fatal(gate_fatal()));
    assert!(
        matches!(gate_outranks, Err(RunError::Broadcast(_))),
        "gate-Fatal outranks the chat failure (D-9 pin)"
    );

    let chat_is_primary = primary_shutdown_outcome(Err(chat_retrieval()), GateFinish::Confirmed);
    let error = chat_is_primary.expect_err("the chat failure is primary when the gate is healthy");
    assert!(matches!(error, RunError::ChatRetrieval(_)));
    assert_eq!(error.exit_code(), 1);
    assert!(!error.is_stop_request_cancellation());
}
