//! The RUN-side offline stream double (plan §C.7): the Arc-shared
//! [`RunChatStreamLedger`] scripts one session spec per `open` and records
//! every open request + operation label, so the app-side rosters prove the
//! establish-only/re-establish loop without any network. The deterministic
//! 250 ms batch-read stall hook (the documented real-sleep precedent) and
//! the stop-flip-on-open hook live here too. Script exhaustion PANICS on
//! the sister fixtures' `expect` precedent — every scripted run therefore
//! terminates deterministically and never fabricates a state. Fixture
//! strings are `*-fixture` shaped only; no test reaches Google.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tonic::async_trait;

use crate::youtube::live_error::LiveApiError;
use crate::youtube::stream_transport::{
    ChatStreamRequest, ChatStreamTransport, LiveChatStreamSession, YieldedChatStreamBatch,
};

const NO_POISON: &str = "the scripted run ledger mutex is not poisoned";
const NO_SCRIPTED_SESSION: &str = "the test scripted a queued session";

type SessionScript = VecDeque<Option<Result<YieldedChatStreamBatch, LiveApiError>>>;
type StopHook = Box<dyn Fn() + Send + Sync + 'static>;

/// One scripted run: queued session specs, the captured opens, the
/// operation labels, the stop-flip hook and the one-shot batch-read stall.
#[derive(Default)]
pub(crate) struct RunChatStreamLedger {
    sessions: Mutex<VecDeque<Result<SessionScript, LiveApiError>>>,
    opened_requests: Mutex<Vec<ChatStreamRequest>>,
    operations: Mutex<Vec<&'static str>>,
    stop_on_open_from: Mutex<Option<(usize, StopHook)>>,
    open_calls: Mutex<usize>,
    stall_next_batch: Mutex<Option<u64>>,
}

impl RunChatStreamLedger {
    /// Queues one session that delivers these states, then the clean close.
    pub(crate) fn script_session(
        &self,
        deliveries: Vec<Option<Result<YieldedChatStreamBatch, LiveApiError>>>,
    ) {
        self.sessions
            .lock()
            .expect(NO_POISON)
            .push_back(Ok(deliveries.into()));
    }

    /// Queues one open that fails with the given classified error.
    pub(crate) fn script_open_failure(&self, error: LiveApiError) {
        self.sessions.lock().expect(NO_POISON).push_back(Err(error));
    }

    /// Flips the stop flag inside `open` from the given 1-based call index
    /// on (the hook is a closure so youtube need not name the app's flag).
    pub(crate) fn flip_stop_on_open_from(
        &self,
        from_call: usize,
        stop: impl Fn() + Send + Sync + 'static,
    ) {
        *self.stop_on_open_from.lock().expect(NO_POISON) = Some((from_call, Box::new(stop)));
    }

    /// Real-stalls the NEXT `next_batch` call once (the documented pin).
    pub(crate) fn stall_next_batch_for(&self, millis: u64) {
        *self.stall_next_batch.lock().expect(NO_POISON) = Some(millis);
    }

    pub(crate) fn operations(&self) -> Vec<&'static str> {
        self.operations.lock().expect(NO_POISON).clone()
    }

    pub(crate) fn opened_requests(&self) -> Vec<ChatStreamRequest> {
        self.opened_requests.lock().expect(NO_POISON).clone()
    }

    fn record_open(&self, request: ChatStreamRequest) {
        self.opened_requests.lock().expect(NO_POISON).push(request);
        self.operations.lock().expect(NO_POISON).push("open");
        let call_index = {
            let mut calls = self.open_calls.lock().expect(NO_POISON);
            *calls += 1;
            *calls
        };
        let armed = self.stop_on_open_from.lock().expect(NO_POISON);
        if let Some((from_call, stop)) = armed.as_ref() {
            if call_index >= *from_call {
                stop();
            }
        }
    }

    fn next_session(&self) -> Result<SessionScript, LiveApiError> {
        self.sessions
            .lock()
            .expect(NO_POISON)
            .pop_front()
            .expect(NO_SCRIPTED_SESSION)
    }

    fn take_stall_millis(&self) -> Option<u64> {
        self.stall_next_batch.lock().expect(NO_POISON).take()
    }
}

pub(crate) struct ScriptedRunStreamTransport {
    ledger: Arc<RunChatStreamLedger>,
}

impl ScriptedRunStreamTransport {
    pub(crate) fn new(ledger: Arc<RunChatStreamLedger>) -> Self {
        Self { ledger }
    }
}

#[async_trait]
impl ChatStreamTransport for ScriptedRunStreamTransport {
    async fn open(
        &mut self,
        request: ChatStreamRequest,
    ) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError> {
        self.ledger.record_open(request);
        let deliveries = self.ledger.next_session()?;
        Ok(Box::new(ScriptedRunStreamSession {
            ledger: Arc::clone(&self.ledger),
            deliveries,
        }))
    }
}

struct ScriptedRunStreamSession {
    ledger: Arc<RunChatStreamLedger>,
    deliveries: SessionScript,
}

#[async_trait]
impl LiveChatStreamSession for ScriptedRunStreamSession {
    async fn next_batch(&mut self) -> Option<Result<YieldedChatStreamBatch, LiveApiError>> {
        if let Some(millis) = self.ledger.take_stall_millis() {
            std::thread::sleep(Duration::from_millis(millis));
        }
        self.deliveries.pop_front().flatten()
    }
}
