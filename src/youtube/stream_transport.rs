//! The mockable stream seam (record P7-12, names fixed): one async open and
//! one async batch tick, `Send`-bounded for the confined runtime thread
//! (Task 3 owns the thread; the seam never touches `std::thread`). The real
//! implementer is `stream_concrete`; the scripted fakes live in the
//! `#[cfg(test)] stream_test_fixtures` sibling. The Option on `next_batch`
//! is EXACTLY the clean server close (record §A.6) — the worker loop maps
//! it to the bounded transient re-establishment branch.

use tonic::async_trait;

use super::live_chat_mapping::ChatTextMessage;
use super::live_error::LiveApiError;

/// The establishment request (record §A.6): the chat id + the resume token
/// when any; the part set is the module constant in `stream_proto`.
#[derive(Clone, Debug)]
pub(crate) struct ChatStreamRequest {
    pub(crate) live_chat_id: String,
    pub(crate) page_token: Option<String>,
}

/// One delivered batch (record §A.6/P7-7): the mapped text carriers in API
/// order + the resume key + the graceful `offlineAt`/`chatEndedEvent`
/// marker. The mapping produced these through the REUSED policy layer.
#[derive(Debug)]
pub(crate) struct YieldedChatStreamBatch {
    pub(crate) messages: Vec<ChatTextMessage>,
    pub(crate) next_page_token: Option<String>,
    pub(crate) offline_ended: bool,
}

#[async_trait]
pub(crate) trait ChatStreamTransport: Send {
    /// Opens (or re-opens) the stream; ONE quota-billed call per invocation
    /// (record P7-5: never called periodically — only establishment and
    /// re-establishment).
    async fn open(
        &mut self,
        request: ChatStreamRequest,
    ) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError>;
}

#[async_trait]
pub(crate) trait LiveChatStreamSession: Send {
    async fn next_batch(&mut self) -> Option<Result<YieldedChatStreamBatch, LiveApiError>>;
}

#[cfg(test)]
#[path = "stream_transport_tests.rs"]
mod stream_transport_tests;
