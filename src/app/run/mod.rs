//! The production `run` mode (plan D-T3-1..D-T3-15; TODO Task 3): the
//! 10-stage startup (Ctrl+C pair → credentials/authorization → the
//! run-side permission gate → transport → broadcast/stream resources →
//! the ONE destination reveal → FFmpeg start → the thread-parallel
//! post-start confirmation gate → the continuous chat-free frame loop →
//! ordered shutdown), its exit funnel, and (in `seams`) the testability
//! parameter objects. Resource fate (D-T3-6 rule 8): NEVER delete, NEVER transition — with
//! `enableAutoStop=true` the broadcast auto-completes ~1 minute after
//! ingestion ends; startup failures AFTER O1/O2 leave the created `ready`
//! resources in place (repeat-run accumulation is the documented
//! limitation; operators delete stale resources in YouTube Studio).
//! Live Chat is OUT of Phase 05 (user ruling): the frame pipeline runs
//! chat-free and the `MessageSource` seam stays byte-unchanged for
//! Phase 06.

mod broadcast;
mod credentials;
mod error;
mod gate;
mod seams;
mod source;
mod streaming;

use std::sync::Arc;

use tracing::info;

use crate::app::pipeline::{install_stop_request_handler, StopFlag};
use crate::config::AppConfig;
use crate::streaming::RtmpsDestination;
use crate::youtube::{CredentialService, GoogleLiveApiTransport, WaitCanceller};

use broadcast::{prepare_broadcast, StageRuntime};
use credentials::{authorize_from_service, authorize_scope_evidence, establish_credentials};
use error::RunError;
use gate::{spawn_startup_gate, GatePlan};
use seams::{RunStops, RuntimeStart, ServiceStops};
use streaming::{PreparedStreaming, StreamingExecution, StreamingRunner};

/// Exit code of a clean run stop (D-T3-8).
const RUN_SUCCESS_EXIT: u8 = 0;

/// Clean-stop line (D-T3-7), printed on the streaming stop path.
const RUN_STOP_MESSAGE: &str =
    "streaming stopped; the broadcast auto-completes on YouTube after ingestion ends";

/// Pre-streaming stop line (D-T3-7), printed when the startup ended on a
/// cancelled wait (the broadcast never went live, so no auto-complete
/// claim is made).
const RUN_STARTUP_STOP_MESSAGE: &str = "run stopped by the stop request before streaming started";

pub(super) fn run_command(app_config: &AppConfig) -> u8 {
    match run_outcome(app_config) {
        Ok(()) => {
            println!("{RUN_STOP_MESSAGE}");
            RUN_SUCCESS_EXIT
        }
        Err(error) if error.is_stop_request_cancellation() => {
            println!("{RUN_STARTUP_STOP_MESSAGE}");
            RUN_SUCCESS_EXIT
        }
        Err(error) => report_run_failure(&error),
    }
}

/// Reports a fatal run-origin failure as exactly one `error:` line on
/// standard error (the demo/auth funnel convention) and returns the
/// mapped exit code.
fn report_run_failure(error: &RunError) -> u8 {
    eprintln!("error: {error}");
    error.exit_code()
}

/// The ordered production startup after the Ctrl+C pair install, against
/// the CONCRETE seams (D-T3-2 stages 2–10). Tests never call this (the
/// ctrlc handler is process-global); they exercise `run_runtime`.
fn run_outcome(app_config: &AppConfig) -> Result<(), RunError> {
    let stops = install_stop_pair()?;
    let production = establish_credentials(app_config)?;
    let service = ServiceStops {
        client_config: &production.client_config,
        store: &production.store,
        endpoint: &production.endpoint,
        verifier: &production.verifier,
    };
    let start = RuntimeStart {
        config: app_config,
        stops,
        service,
        build_transport: Box::new(|access_token| {
            Arc::new(GoogleLiveApiTransport::new(access_token))
        }),
    };
    run_runtime(start, &PreparedStreaming)
}

/// D-T3-2 stage 1 (the one documented refinement's precondition): the
/// app's SINGLE Ctrl+C handler flips BOTH the frame-loop `StopFlag` AND a
/// `WaitCanceller` clone (D-T2-10's Task-3 contract — both flags, one
/// handler; no youtube types in `pipeline`).
fn install_stop_pair() -> Result<RunStops, RunError> {
    let stops = RunStops {
        stop_flag: StopFlag::new(),
        canceller: WaitCanceller::new(),
    };
    let handler_flag = stops.stop_flag.clone();
    let handler_canceller = stops.canceller.clone();
    install_stop_request_handler(Box::new(move || {
        handler_flag.request_stop();
        handler_canceller.request_cancel();
    }))
    .map_err(RunError::Interrupt)?;
    Ok(stops)
}

/// The WHOLE run startup after the Ctrl+C pair install, with injected
/// collaborators + the streaming-stage runner (D-T3-9). Production
/// `run_outcome` supplies the concrete seams; tests supply fakes
/// (offline). Private (not `pub(super)`): its parameter/return types are
/// `run`-module-internal, and both callers live in this module's subtree.
fn run_runtime(start: RuntimeStart<'_>, runner: &dyn StreamingRunner) -> Result<(), RunError> {
    let service = CredentialService {
        store: start.service.store,
        endpoint: start.service.endpoint,
        verifier: start.service.verifier,
        client_config: start.service.client_config,
    };
    let verified = authorize_from_service(&service)?;
    authorize_scope_evidence(&verified)?;
    let transport = (start.build_transport)(verified.access_token);
    let stage = StageRuntime {
        transport: transport.as_ref(),
        cancellation: &start.stops.canceller,
    };
    let prepared = prepare_broadcast(start.config, &stage).map_err(RunError::Broadcast)?;
    info!(
        broadcast_id = prepared.broadcast_id(),
        "broadcast prepared for streaming"
    );
    let stream_id = prepared.stream_id().to_string();
    let destination = RtmpsDestination::new(prepared.into_ingestion().into_url());
    let gate = spawn_startup_gate(GatePlan {
        transport: Arc::clone(&transport),
        cancellation: start.stops.canceller.clone(),
        stream_id: stream_id.clone(),
        stop_flag: start.stops.stop_flag.clone(),
    });
    runner.run(
        start.config,
        StreamingExecution {
            stop_flag: start.stops.stop_flag,
            canceller: start.stops.canceller,
            destination,
            stream_id,
            gate,
        },
    )
}

#[cfg(test)]
#[path = "runtime_test_fixtures.rs"]
mod runtime_test_fixtures;

#[cfg(test)]
#[path = "runtime_credential_fakes.rs"]
mod runtime_credential_fakes;

#[cfg(test)]
#[path = "runtime_runner_fixture.rs"]
mod runtime_runner_fixture;

#[cfg(test)]
#[path = "mod_tests.rs"]
mod mod_tests;

#[cfg(test)]
#[path = "credentials_tests.rs"]
mod credentials_tests;

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;
