//! The offline chat fakes for the Phase-07 run rosters (the
//! `runtime_test_fixtures` Mutex-based precedent): a lookup-only scripted
//! O6 transport (the O7 list arm retired with the streamList switch) plus
//! the shared youtube-side run stream ledger, bundled as the
//! [`ChatFakes`] parameter object so every helper stays ≤2 args. Fixture
//! material is `-fixture` shaped only — never real-shaped, never reaching
//! Google.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::app::pipeline::StopFlag;
use crate::youtube::{
    response_offline, response_with_token, text_item, yielded, LiveApiError, LiveChatTransport,
    RunChatStreamLedger, ScriptedRunStreamTransport, RESUME_TOKEN_FIXTURE,
};

type ChatLookup = Result<Option<String>, LiveApiError>;

pub(crate) const LIVE_CHAT_ID_FIXTURE: &str = "live-chat-id-fixture";
const BROADCAST_ID_FIXTURE: &str = "broadcast-id-fixture";
const NO_POISON: &str = "fixture mutex";

#[derive(Default)]
pub(crate) struct ScriptedRunChatTransport {
    lookup_results: Mutex<VecDeque<ChatLookup>>,
    operations: Mutex<Vec<&'static str>>,
}

impl ScriptedRunChatTransport {
    pub(crate) fn script_lookup(&self, outcome: ChatLookup) {
        self.lookup_results
            .lock()
            .expect(NO_POISON)
            .push_back(outcome);
    }

    pub(crate) fn operations(&self) -> Vec<&'static str> {
        self.operations.lock().expect(NO_POISON).clone()
    }
}

impl LiveChatTransport for ScriptedRunChatTransport {
    fn lookup_broadcast_chat_id(
        &self,
        _broadcast_id: &str,
    ) -> Result<Option<String>, LiveApiError> {
        self.operations
            .lock()
            .expect(NO_POISON)
            .push("lookup_broadcast_chat_id");
        self.lookup_results
            .lock()
            .expect(NO_POISON)
            .pop_front()
            .expect("the test scripted a queued lookup")
    }
}

/// The chat-side fakes bundle (≤2-args rule): the O6 lookup fake and the
/// run stream ledger are separate because the worker owns both seams.
#[derive(Default)]
pub(crate) struct ChatFakes {
    pub(crate) lookup: Arc<ScriptedRunChatTransport>,
    pub(crate) stream_ledger: Arc<RunChatStreamLedger>,
}

impl ChatFakes {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn operations(&self) -> Vec<&'static str> {
        self.lookup.operations()
    }

    /// Queues the §E terminator session: one offline-ended empty batch, then the clean close.
    pub(crate) fn script_graceful_end_session(&self) {
        self.stream_ledger
            .script_session(vec![Some(Ok(yielded(&response_offline(vec![]))))]);
    }

    /// Queues a session delivering one empty batch carrying the resume token;
    /// the token clears the pending history and opens the live window (§A.3).
    pub(crate) fn script_token_batch_session(&self) {
        self.stream_ledger
            .script_session(vec![Some(Ok(yielded(&response_with_token(
                vec![],
                RESUME_TOKEN_FIXTURE,
            ))))]);
    }

    /// Queues a session delivering the token-bearing one-message history batch —
    /// the batch the §A.3 baseline rule discards whole while history is pending.
    pub(crate) fn script_history_batch_session(&self) {
        self.stream_ledger
            .script_session(vec![Some(Ok(yielded(&response_with_token(
                vec![text_item("h-1")],
                RESUME_TOKEN_FIXTURE,
            ))))]);
    }
}

pub(crate) fn worker_plan(
    fakes: &ChatFakes,
    channel: &super::ChatChannel,
) -> super::ChatWorkerPlan {
    super::ChatWorkerPlan {
        lookup: fakes.lookup.clone(),
        stream: Box::new(ScriptedRunStreamTransport::new(Arc::clone(
            &fakes.stream_ledger,
        ))),
        broadcast_id: BROADCAST_ID_FIXTURE.to_string(),
        sender: channel.sender.clone(),
        terminal: Arc::clone(&channel.terminal),
        stop_flag: StopFlag::new(),
    }
}

/// Runs the chat worker to completion against the scripted fakes and
/// returns the channel bundle for the caller's drain/terminal asserts.
pub(crate) fn run_worker(fakes: &ChatFakes) -> super::ChatChannel {
    let channel = super::chat_channel();
    super::run_chat_worker(worker_plan(fakes, &channel));
    channel
}
