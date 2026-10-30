//! The worker panic funnel roster (2026-10-30 muteness fix): a stream
//! transport whose `open` panics is contained by the worker's catch
//! boundary — the typed terminal is written with the FIXED detail, the
//! calling thread resumes (the process does not abort), the sender drops,
//! and `ChatSource` surfaces the typed retrieval error to the frame
//! funnel (once, then `Ok(None)` forever). Offline only; fixture-shaped.

use std::sync::Arc;

use tonic::async_trait;

use crate::app::pipeline::{ChatRetrievalError, ChatSourceError, MessageSource, StopFlag};
use crate::youtube::{ChatStreamRequest, ChatStreamTransport, LiveApiError, LiveChatStreamSession};

use super::super::source::ChatSource;
use super::runtime_chat_fakes::{ScriptedRunChatTransport, LIVE_CHAT_ID_FIXTURE};
use super::{chat_channel, run_chat_worker, ChatChannel, ChatWorkerPlan, CHAT_WORKER_PANIC_DETAIL};

const BROADCAST_ID_FIXTURE: &str = "broadcast-id-fixture";

#[derive(Default)]
struct PanickingStreamTransport;

#[async_trait]
impl ChatStreamTransport for PanickingStreamTransport {
    async fn open(
        &mut self,
        _request: ChatStreamRequest,
    ) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError> {
        panic!("panic-fixture: the scripted stream open panics");
    }
}

fn panicking_stream_plan(channel: &ChatChannel) -> ChatWorkerPlan {
    let lookup = ScriptedRunChatTransport::default();
    lookup.script_lookup(Ok(Some(LIVE_CHAT_ID_FIXTURE.to_string())));
    ChatWorkerPlan {
        lookup: Arc::new(lookup),
        stream: Box::new(PanickingStreamTransport),
        broadcast_id: BROADCAST_ID_FIXTURE.to_string(),
        sender: channel.sender.clone(),
        terminal: Arc::clone(&channel.terminal),
        stop_flag: StopFlag::new(),
    }
}

fn assert_terminal_carries_the_panic_detail(channel: &ChatChannel) {
    let cell = channel.terminal.lock().expect("cell mutex");
    let stored = cell
        .as_ref()
        .expect("a contained worker panic writes a terminal");
    let ChatRetrievalError::Transport { detail } = stored else {
        panic!("the contained panic writes the Transport terminal class");
    };
    assert_eq!(detail.as_str(), CHAT_WORKER_PANIC_DETAIL);
}

#[test]
fn worker_panicking_transport_writes_the_terminal() {
    let channel = chat_channel();
    run_chat_worker(panicking_stream_plan(&channel));
    assert_terminal_carries_the_panic_detail(&channel);
    drop(channel.sender);

    let ChatChannel {
        receiver, terminal, ..
    } = channel;
    let mut source = ChatSource::new(receiver, Arc::clone(&terminal));
    assert!(matches!(
        source.next_message(),
        Err(ChatSourceError::Retrieval(
            ChatRetrievalError::Transport { .. }
        ))
    ));
    assert!(matches!(source.next_message(), Ok(None)));
}
