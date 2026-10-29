//! Shared offline stream fixtures (the `live_chat_test_fixtures` precedent,
//! record D-14 style): the scripted ChatStream-transport fake with per-
//! session `VecDeque` scripting, the response/carrier builders, and the
//! tiny current-thread test runtime helper (the TODO's sanctioned
//! test-constructed runtime — zero sleeps; fakes return immediately).
//! Never built in production; fixture strings are `*-fixture` shaped only;
//! no test reaches Google.

use std::collections::VecDeque;
use std::future::Future;
use std::sync::Mutex;

use tonic::async_trait;

use super::live_chat_stream_mapping::map_stream_batch;
use super::live_error::LiveApiError;
use super::stream_proto::{
    DisplayedContent, LiveChatMessage, LiveChatMessageAuthorDetails, LiveChatMessageListResponse,
    LiveChatMessageSnippet, LiveChatTextMessageDetails, SnippetType,
};
use super::stream_transport::{
    ChatStreamRequest, ChatStreamTransport, LiveChatStreamSession, YieldedChatStreamBatch,
};

pub(crate) const ACCESS_TOKEN_FIXTURE: &str = "access-token-fixture";
pub(crate) const LIVE_CHAT_ID_FIXTURE: &str = "live-chat-id-fixture";
pub(crate) const RESUME_TOKEN_FIXTURE: &str = "token-fixture";
pub(crate) const PUBLISHED_AT_FIXTURE: &str = "2026-10-25T12:30:45Z";

const NO_POISON: &str = "the scripted stream mutex is not poisoned";

pub(crate) fn stream_test_block_on<T>(future: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("the stream test runtime builds")
        .block_on(future)
}

/// One scripted session: a queue of delivered states; popping past the end
/// = the clean close `None`.
pub(crate) struct ScriptedSessionSpec(
    VecDeque<Option<Result<YieldedChatStreamBatch, LiveApiError>>>,
);

impl ScriptedSessionSpec {
    pub(crate) fn new(
        deliveries: Vec<Option<Result<YieldedChatStreamBatch, LiveApiError>>>,
    ) -> Self {
        Self(deliveries.into())
    }
}

/// The scripted stream transport: queued session specs, a recorded
/// operation-label sequence and the captured open requests. `Mutex`
/// (never `RefCell`) so the fake satisfies the seam's `Send` superbound.
#[derive(Default)]
pub(crate) struct ScriptedChatStreamTransport {
    pub(crate) sessions: Mutex<VecDeque<Result<ScriptedSessionSpec, LiveApiError>>>,
    pub(crate) operations: Mutex<Vec<&'static str>>,
    pub(crate) captured_requests: Mutex<Vec<ChatStreamRequest>>,
}

impl ScriptedChatStreamTransport {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn enqueue_session(&self, spec: Result<ScriptedSessionSpec, LiveApiError>) {
        self.sessions.lock().expect(NO_POISON).push_back(spec);
    }

    pub(crate) fn invokes(&self) -> Vec<&'static str> {
        std::mem::take(&mut *self.operations.lock().expect(NO_POISON))
    }

    pub(crate) fn captured(&self) -> Vec<ChatStreamRequest> {
        std::mem::take(&mut *self.captured_requests.lock().expect(NO_POISON))
    }
}

#[async_trait]
impl ChatStreamTransport for ScriptedChatStreamTransport {
    async fn open(
        &mut self,
        request: ChatStreamRequest,
    ) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError> {
        self.captured_requests
            .lock()
            .expect(NO_POISON)
            .push(request);
        self.operations.lock().expect(NO_POISON).push("open");
        let next_session = self.sessions.lock().expect(NO_POISON).pop_front();
        match next_session {
            Some(Ok(spec)) => Ok(Box::new(ScriptedChatStreamSession {
                deliveries: Mutex::new(spec.0),
            })),
            Some(Err(error)) => Err(error),
            None => Err(LiveApiError::Network {
                reason: "no scripted session remained".to_string(),
            }),
        }
    }
}

pub(crate) struct ScriptedChatStreamSession {
    deliveries: Mutex<VecDeque<Option<Result<YieldedChatStreamBatch, LiveApiError>>>>,
}

impl ScriptedChatStreamSession {
    pub(crate) fn new_empty() -> Self {
        Self {
            deliveries: Mutex::new(VecDeque::new()),
        }
    }
}

#[async_trait]
impl LiveChatStreamSession for ScriptedChatStreamSession {
    async fn next_batch(&mut self) -> Option<Result<YieldedChatStreamBatch, LiveApiError>> {
        self.deliveries
            .lock()
            .expect(NO_POISON)
            .pop_front()
            .flatten()
    }
}

pub(crate) fn text_item(id: &str) -> LiveChatMessage {
    LiveChatMessage {
        id: Some(id.to_string()),
        snippet: Some(LiveChatMessageSnippet {
            message_type: Some(SnippetType::TextMessageEvent as i32),
            published_at: Some(PUBLISHED_AT_FIXTURE.to_string()),
            displayed_content: Some(DisplayedContent::TextMessage(LiveChatTextMessageDetails {
                message_text: Some("text-fixture".to_string()),
            })),
            ..Default::default()
        }),
        author_details: Some(LiveChatMessageAuthorDetails {
            display_name: Some("author-fixture".to_string()),
            channel_id: Some("channel-id-fixture".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub(crate) fn tombstone_item(id: &str) -> LiveChatMessage {
    LiveChatMessage {
        id: Some(id.to_string()),
        snippet: Some(LiveChatMessageSnippet {
            message_type: Some(SnippetType::Tombstone as i32),
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub(crate) fn ended_item() -> LiveChatMessage {
    LiveChatMessage {
        snippet: Some(LiveChatMessageSnippet {
            message_type: Some(SnippetType::ChatEndedEvent as i32),
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub(crate) fn response_plain(items: Vec<LiveChatMessage>) -> LiveChatMessageListResponse {
    LiveChatMessageListResponse {
        items,
        ..Default::default()
    }
}

pub(crate) fn response_with_token(
    items: Vec<LiveChatMessage>,
    next_page_token: &str,
) -> LiveChatMessageListResponse {
    LiveChatMessageListResponse {
        items,
        next_page_token: Some(next_page_token.to_string()),
        ..Default::default()
    }
}

pub(crate) fn response_offline(items: Vec<LiveChatMessage>) -> LiveChatMessageListResponse {
    LiveChatMessageListResponse {
        items,
        offline_at: Some(PUBLISHED_AT_FIXTURE.to_string()),
        ..Default::default()
    }
}

pub(crate) fn yielded(response: &LiveChatMessageListResponse) -> YieldedChatStreamBatch {
    map_stream_batch(response)
}
