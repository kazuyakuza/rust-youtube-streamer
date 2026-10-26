//! Typed failures of the production `run` orchestration (plan D-T3-8,
//! D-T3-15 A2): one variant per failure class, hand-written `Display`
//! split into per-group helpers, no panics, and the scheduled exit-code
//! mapping — every run-origin fatal failure is the startup-failure class
//! (1); the pre-streaming stop-request cancellation maps to 0 through the
//! exit funnel. Payloads are typed inner errors that are already
//! secret-free (the RTMPS destination never enters any of them).

use std::fmt;

use crate::app::error::STARTUP_FAILURE_EXIT;
use crate::app::pipeline::{ChatRetrievalError, FramePipelineError, InterruptHandlerError};
use crate::youtube::{
    ClientConfigError, CredentialServiceError, LiveApiError, TokenEndpointError, REQUIRED_SCOPE,
};

use super::RUN_SUCCESS_EXIT;

/// All fatal `run`-origin failures; one class per startup/streaming stage.
#[derive(Debug)]
pub(super) enum RunError {
    Interrupt(InterruptHandlerError),
    ClientConfig(ClientConfigError),
    EndpointInit(TokenEndpointError),
    Credential(CredentialServiceError),
    InsufficientScope,
    Broadcast(LiveApiError),
    Pipeline(FramePipelineError),
    GateThreadStopped,
    /// The typed chat-retrieval funnel class (record D-8/D-9): the
    /// production source's terminal cell, surfaced as ONE redacted line.
    ChatRetrieval(ChatRetrievalError),
}

impl RunError {
    /// The stop-request cancellation class (D-T3-7): a pre-streaming
    /// Ctrl+C ends the run on the normal-stop path.
    pub(super) fn is_stop_request_cancellation(&self) -> bool {
        matches!(self, RunError::Broadcast(LiveApiError::WaitCancelled))
    }

    /// Scheduled exit-code mapping (D-T3-8): every run-origin fatal
    /// failure exits 1; the cancellation class exits 0.
    pub(super) fn exit_code(&self) -> u8 {
        if self.is_stop_request_cancellation() {
            return RUN_SUCCESS_EXIT;
        }
        STARTUP_FAILURE_EXIT
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::Interrupt(error) => write!(formatter, "{error}"),
            RunError::ClientConfig(error) => write!(formatter, "{error}"),
            RunError::EndpointInit(error) => format_endpoint_init(formatter, error),
            RunError::Credential(error) => write!(formatter, "{error}"),
            RunError::InsufficientScope => format_insufficient_scope(formatter),
            RunError::Broadcast(error) => write!(formatter, "{error}"),
            RunError::Pipeline(error) => write!(formatter, "{error}"),
            RunError::GateThreadStopped => format_gate_thread_stopped(formatter),
            RunError::ChatRetrieval(error) => write!(formatter, "{error}"),
        }
    }
}

fn format_endpoint_init(
    formatter: &mut fmt::Formatter<'_>,
    error: &TokenEndpointError,
) -> fmt::Result {
    write!(
        formatter,
        "unable to initialize the Google token endpoint: {error}"
    )
}

/// The ONE scope line allowed to name the public `REQUIRED_SCOPE` URL
/// (D-T3-7: it is public documentation, never a granted-scope value).
fn format_insufficient_scope(formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
        formatter,
        "the verified authorization does not grant the required YouTube scope '{REQUIRED_SCOPE}'; run the authorization command again"
    )
}

fn format_gate_thread_stopped(formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("the startup confirmation thread stopped without recording an outcome")
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod error_tests;
