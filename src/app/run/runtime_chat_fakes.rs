//! The offline chat fake for the Phase-06 run tests (the
//! `runtime_test_fixtures` Mutex-based precedent): scripted O6/O7
//! outcomes, an operation ledger, the received-`ChatListCommand`
//! recorder, an optional stop-flag flip hook and an optional stall for
//! the bounded-shutdown roster. Fixture material is `-fixture` shaped
//! only — never real-shaped, never reaching Google.

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::app::pipeline::StopFlag;
use crate::youtube::{ChatListCommand, LiveApiError, LiveChatMessageList, LiveChatTransport};

type ChatLookup = Result<Option<String>, LiveApiError>;
type ChatPage = Result<LiveChatMessageList, LiveApiError>;

pub(crate) const LIVE_CHAT_ID_FIXTURE: &str = "live-chat-id-fixture";
const BROADCAST_ID_FIXTURE: &str = "broadcast-id-fixture";

#[derive(Default)]
pub(crate) struct ScriptedRunChatTransport {
    lookup_results: Mutex<VecDeque<ChatLookup>>,
    list_results: Mutex<VecDeque<ChatPage>>,
    operations: Mutex<Vec<&'static str>>,
    commands: Mutex<Vec<ChatListCommand>>,
    stop_on_list_from: Mutex<Option<(usize, StopFlag)>>,
    stall_millis: Mutex<Option<u64>>,
    list_calls: Mutex<usize>,
}

impl ScriptedRunChatTransport {
    pub(crate) fn script_lookup(&self, outcome: ChatLookup) {
        self.lookup_results
            .lock()
            .expect("fixture mutex")
            .push_back(outcome);
    }

    pub(crate) fn script_list(&self, outcome: ChatPage) {
        self.list_results
            .lock()
            .expect("fixture mutex")
            .push_back(outcome);
    }

    /// Flips the flag inside `list_messages` from the given 1-based call
    /// index onward (the stop-during-delivery hook).
    pub(crate) fn flip_stop_on_list_from(&self, from_call: usize, flag: StopFlag) {
        *self.stop_on_list_from.lock().expect("fixture mutex") = Some((from_call, flag));
    }

    /// Stalls every `list_messages` call (the blocked-call roster).
    pub(crate) fn stall_list_calls_for(&self, millis: u64) {
        *self.stall_millis.lock().expect("fixture mutex") = Some(millis);
    }

    pub(crate) fn operations(&self) -> Vec<&'static str> {
        self.operations.lock().expect("fixture mutex").clone()
    }

    pub(crate) fn commands(&self) -> Vec<ChatListCommand> {
        self.commands.lock().expect("fixture mutex").clone()
    }

    fn record(&self, label: &'static str) {
        self.operations.lock().expect("fixture mutex").push(label);
    }

    fn before_list(&self) {
        let mut calls = self.list_calls.lock().expect("fixture mutex");
        *calls += 1;
        let call_index = *calls;
        drop(calls);
        if let Some(millis) = *self.stall_millis.lock().expect("fixture mutex") {
            std::thread::sleep(std::time::Duration::from_millis(millis));
        }
        let armed = self
            .stop_on_list_from
            .lock()
            .expect("fixture mutex")
            .clone();
        if let Some((from_call, flag)) = armed {
            if call_index >= from_call {
                flag.request_stop();
            }
        }
    }
}

impl LiveChatTransport for ScriptedRunChatTransport {
    fn lookup_broadcast_chat_id(
        &self,
        _broadcast_id: &str,
    ) -> Result<Option<String>, LiveApiError> {
        self.record("lookup_broadcast_chat_id");
        pop(&self.lookup_results)
    }

    fn list_messages(
        &self,
        command: &ChatListCommand,
    ) -> Result<LiveChatMessageList, LiveApiError> {
        self.before_list();
        self.record("list_messages");
        self.commands
            .lock()
            .expect("fixture mutex")
            .push(command.clone());
        pop(&self.list_results)
    }
}

fn pop<T>(queue: &Mutex<VecDeque<Result<T, LiveApiError>>>) -> Result<T, LiveApiError> {
    queue
        .lock()
        .expect("fixture mutex")
        .pop_front()
        .expect("the test scripted a queued result")
}

/// Builds one `liveChatMessages.list` page (camelCase wire shape; null
/// optionals decode to `None`).
pub(crate) fn chat_page(
    items: Vec<serde_json::Value>,
    token: Option<&str>,
    interval: Option<u64>,
    offline_at: Option<&str>,
) -> LiveChatMessageList {
    serde_json::from_value(serde_json::json!({
        "items": items,
        "nextPageToken": token,
        "pollingIntervalMillis": interval,
        "offlineAt": offline_at,
    }))
    .expect("the chat page fixture parses")
}

/// One accepted `textMessageEvent` item (D-4's accept policy shape).
pub(crate) fn text_message_item(
    id: &str,
    author: &str,
    text: &str,
    published_at: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "snippet": {
            "type": "textMessageEvent",
            "publishedAt": published_at,
            "textMessageDetails": {"messageText": text}
        },
        "authorDetails": {"displayName": author}
    })
}

pub(crate) fn worker_plan(
    fake: std::sync::Arc<ScriptedRunChatTransport>,
    channel: &super::ChatChannel,
) -> super::ChatWorkerPlan {
    super::ChatWorkerPlan {
        transport: fake,
        broadcast_id: BROADCAST_ID_FIXTURE.to_string(),
        sender: channel.sender.clone(),
        terminal: std::sync::Arc::clone(&channel.terminal),
        stop_flag: StopFlag::new(),
    }
}
