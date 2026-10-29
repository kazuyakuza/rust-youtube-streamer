//! The bounded Phase-07 chat worker (records P7-5..P7-12; §2.2
//! spawn-at-confirmation topology — the gate thread spawns this worker on
//! `GateFinish::Confirmed`, its FIRST act being the D-2 chat-id
//! resolution; §2.3 join-position refinement — `gate.finish()` joins it
//! after the sink EOF because the pinned one-message-per-frame pipeline
//! owns loop break + `sink.finish()`; the Ctrl+C handler flag-stops it
//! concurrently). Retrieval is the streamList gRPC client behind ONE
//! confined current-thread runtime on THIS worker thread (record §10);
//! the resumable loop lives in `chat_stream_worker`, the delivery and
//! failure machinery in `chat_stream_delivery` (the AMENDMENT-2 split
//! precedent). Boundedness: 64-entry channel drained by the source's
//! `try_recv`, 1 s stop-slices, backoff cap 60 s, 10-transient budget, the
//! CLOSED transient list (`chat_failure`), and one session per
//! establishment (never periodic). Shutdown losses: queued-but-unconsumed
//! messages (≤64) are never logged (log/video never diverge); a Ctrl+C
//! during the one-shot ≤30 s resolution window delays the join by at most
//! the remaining budget (raw 5 s slices, Task-2's no-canceller decision).
//! D-13 redaction: message text/authors/user-message ids are NEVER logged;
//! only the chat id appears, as the `live_chat_id` tracing field of the
//! resolution line.

use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use tracing::info;

use crate::app::pipeline::{ChatRetrievalError, StopFlag};
use crate::chat::ChatMessage;
use crate::youtube::{LiveApiError, LiveChatTransport};

use super::chat_failure::terminal_retrieval_error;
use super::chat_stream_delivery::record_terminal;
use super::chat_stream_worker::{run_stream_loop, StreamWorker};
use super::seams::{ChatStageSources, ChatStreamHandle, ChatTransport};
use super::source::ChatSource;

pub(super) const CHAT_CHANNEL_CAPACITY: usize = 64;
pub(super) const CHAT_SLEEP_SLICE_MS: u64 = 1_000;
pub(super) const CHAT_BACKOFF_MAX_MS: u64 = 60_000;
pub(super) const CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES: u32 = 10;

pub(super) const CHAT_ID_RESOLVED_MESSAGE: &str = "live chat id resolved";
pub(super) const CHAT_BASELINE_MESSAGE: &str =
    "live chat baseline established; the history page was discarded";
pub(super) const CHAT_ENDED_MESSAGE: &str =
    "live chat ended; the stream continues without new chat messages";

/// The confined runtime start failure reason (a typed Transport detail; no
/// panic — a runtime that cannot start is a typed terminal).
const CHAT_RUNTIME_START_FAILURE: &str = "the confined chat streaming runtime failed to start";

/// The bounded channel bundle: the frame-loop side (source) and the
/// worker side (plan) of one 64-entry channel plus the shared terminal cell.
pub(super) struct ChatChannel {
    pub(super) sender: SyncSender<ChatMessage>,
    pub(super) receiver: Receiver<ChatMessage>,
    pub(super) terminal: Arc<Mutex<Option<ChatRetrievalError>>>,
}

pub(super) fn chat_channel() -> ChatChannel {
    let (sender, receiver) = sync_channel(CHAT_CHANNEL_CAPACITY);
    ChatChannel {
        sender,
        receiver,
        terminal: Arc::new(Mutex::new(None)),
    }
}

/// The gate-carried worker payload (spawned on `GateFinish::Confirmed`);
/// `stop_flag` is the SAME shared frame-loop flag (D-T3-C).
pub(super) struct ChatWorkerPlan {
    pub(super) lookup: ChatTransport,
    pub(super) stream: ChatStreamHandle,
    pub(super) broadcast_id: String,
    pub(super) sender: SyncSender<ChatMessage>,
    pub(super) terminal: Arc<Mutex<Option<ChatRetrievalError>>>,
    pub(super) stop_flag: StopFlag,
}

pub(super) fn spawn_chat_worker(plan: ChatWorkerPlan) -> JoinHandle<()> {
    thread::spawn(move || run_chat_worker(plan))
}

/// The stage-7.5 constructor consumed by `run_runtime` (3-arg sanctioned
/// exception family, record D-T3-15): splits the channel bundle into the
/// pipeline source and the gate-carried worker plan.
pub(super) fn build_chat_stage(
    chat: ChatStageSources,
    broadcast_id: &str,
    stop_flag: StopFlag,
) -> (ChatSource, ChatWorkerPlan) {
    let ChatStageSources { lookup, stream } = chat;
    let channel = chat_channel();
    let source = ChatSource::new(channel.receiver, Arc::clone(&channel.terminal));
    let plan = ChatWorkerPlan {
        lookup,
        stream,
        broadcast_id: broadcast_id.to_string(),
        sender: channel.sender,
        terminal: channel.terminal,
        stop_flag,
    };
    (source, plan)
}

fn run_chat_worker(plan: ChatWorkerPlan) {
    if plan.stop_flag.is_requested() {
        return;
    }
    let live_chat_id = match resolve_chat_id(plan.lookup.as_ref(), &plan.broadcast_id) {
        Ok(id) => {
            info!(live_chat_id = %id, "{}", CHAT_ID_RESOLVED_MESSAGE);
            id
        }
        Err(error) => {
            record_terminal(&plan.terminal, terminal_retrieval_error(&error));
            return;
        }
    };
    let Ok(runtime) = chat_runtime() else {
        record_terminal(
            &plan.terminal,
            ChatRetrievalError::Transport {
                detail: CHAT_RUNTIME_START_FAILURE.to_string(),
            },
        );
        return;
    };
    runtime.block_on(run_stream_loop(StreamWorker::new(plan, live_chat_id)));
}

/// ONE runtime instance for the whole app (record §10): the current-thread
/// scheduler confined to this worker thread, built once per run.
fn chat_runtime() -> Result<tokio::runtime::Runtime, ()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| ())
}

#[cfg(not(test))]
fn resolve_chat_id(
    transport: &dyn LiveChatTransport,
    broadcast_id: &str,
) -> Result<String, LiveApiError> {
    crate::youtube::resolve_live_chat_id(transport, broadcast_id)
}

#[cfg(test)]
const CHAT_ID_RESOLUTION_TEST_INTERVAL_MS: u64 = 0;

#[cfg(test)]
fn resolve_chat_id(
    transport: &dyn LiveChatTransport,
    broadcast_id: &str,
) -> Result<String, LiveApiError> {
    crate::youtube::resolve_live_chat_id_with_interval(
        transport,
        broadcast_id,
        CHAT_ID_RESOLUTION_TEST_INTERVAL_MS,
    )
}

#[cfg(test)]
#[path = "runtime_chat_fakes.rs"]
pub(super) mod runtime_chat_fakes;

#[cfg(test)]
#[path = "chat_worker_tests.rs"]
mod chat_worker_tests;

#[cfg(test)]
#[path = "chat_worker_flow_tests.rs"]
mod chat_worker_flow_tests;

#[cfg(test)]
#[path = "chat_worker_budget_tests.rs"]
mod chat_worker_budget_tests;

#[cfg(test)]
#[path = "gate_chat_tests.rs"]
mod gate_chat_tests;
