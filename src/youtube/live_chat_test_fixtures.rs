//! Shared Phase-06 chat fixtures (record D-14, plan §3.10): the scripted
//! [`ScriptedLiveChatTransport`] fake, the canned list-body builder and the
//! `*-fixture` constants — registered `#[cfg(test)]` via `#[path]` from
//! `live_chat_resolution` and from the wire roster, so BOTH instantiations
//! own their copy and every path inside is crate-rooted. Never built in
//! production; fixture strings are `*-fixture` shaped only — never
//! real-shaped credentials; no test reaches Google.

use std::cell::RefCell;
use std::collections::VecDeque;

use crate::youtube::api_chat_transport::LiveChatTransport;
use crate::youtube::live_chat_models::{ChatListCommand, LiveChatMessageList};
use crate::youtube::live_error::LiveApiError;

pub(super) const BROADCAST_ID_FIXTURE: &str = "broadcast-id-fixture";
pub(super) const LIVE_CHAT_ID_FIXTURE: &str = "live-chat-id-fixture";
pub(super) const ACCESS_TOKEN_FIXTURE: &str = "access-token-fixture";

/// Scripted `LiveChatTransport` fake (consumed ONLY by the resolution
/// tests): per-call queued `Result`s plus the recorded operation sequence
/// the roster's "single call" pins read back.
pub(super) struct ScriptedLiveChatTransport {
    pub(super) lookup_results: RefCell<VecDeque<Result<Option<String>, LiveApiError>>>,
    pub(super) list_results: RefCell<VecDeque<Result<LiveChatMessageList, LiveApiError>>>,
    pub(super) operations: RefCell<Vec<&'static str>>,
}

impl LiveChatTransport for ScriptedLiveChatTransport {
    fn lookup_broadcast_chat_id(
        &self,
        _broadcast_id: &str,
    ) -> Result<Option<String>, LiveApiError> {
        self.operations
            .borrow_mut()
            .push("lookup_broadcast_chat_id");
        next_result(&self.lookup_results)
    }

    fn list_messages(
        &self,
        _command: &ChatListCommand,
    ) -> Result<LiveChatMessageList, LiveApiError> {
        self.operations.borrow_mut().push("list_messages");
        next_result(&self.list_results)
    }
}

fn next_result<T>(queue: &RefCell<VecDeque<Result<T, LiveApiError>>>) -> Result<T, LiveApiError> {
    queue
        .borrow_mut()
        .pop_front()
        .expect("the test scripted a queued result")
}

/// Canned O7 page body: the wire roster's JSON-items segment wrapped as a
/// full (token-free, interval-free) list answer.
pub(super) fn live_chat_list_body(json_items: &str) -> String {
    format!(r#"{{"items":{json_items}}}"#)
}
