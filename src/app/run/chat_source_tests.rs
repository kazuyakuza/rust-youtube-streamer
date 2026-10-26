//! Roster C — the production `ChatSource` contract (plan §5.3):
//! non-blocking `try_recv` so the frame loop NEVER waits on chat, the
//! one-shot terminal error, the graceful-empty cell, and the full
//! store/renderer integration through the frame pipeline. Offline only.

use crate::app::pipeline::{
    ChatRetrievalError, ChatSourceError, FakeClock, FramePipeline, MessageSource, PipelinePacing,
    PipelineParts, RecordingSink, StopFlag, StubRenderer, TEST_FRAME_BYTES, TEST_INTERVAL_NANOS,
};
use crate::chat::{ChatMessage, ChatStore, ChatStoreSettings};
use crate::config::valid_config;

use super::ChatSource;
use crate::app::run::chat_worker::{chat_channel, ChatChannel};

fn source_of(channel: ChatChannel) -> ChatSource {
    ChatSource::new(channel.receiver, channel.terminal)
}

#[test]
fn source_yields_none_while_the_channel_is_open_and_empty() {
    let channel = chat_channel();
    let sender_kept_alive = channel.sender.clone();
    let mut source = source_of(channel);

    assert!(matches!(source.next_message(), Ok(None)));
    drop(sender_kept_alive);
}

#[test]
fn source_delivers_queued_messages_in_order_without_blocking() {
    let channel = chat_channel();
    channel
        .sender
        .send(ChatMessage::new("alice", "first"))
        .expect("the bounded channel accepts");
    channel
        .sender
        .send(ChatMessage::new("bob", "second"))
        .expect("the bounded channel accepts");
    let mut source = source_of(channel);

    let first = source.next_message().expect("delivery succeeds");
    let second = source.next_message().expect("delivery succeeds");
    assert_eq!(first.expect("message 1").to_string(), "alice: first");
    assert_eq!(second.expect("message 2").to_string(), "bob: second");
    assert!(matches!(source.next_message(), Ok(None)));
}

#[test]
fn source_surfaces_the_terminal_error_once_then_yields_none() {
    let channel = chat_channel();
    drop(channel.sender);
    *channel.terminal.lock().expect("cell mutex") = Some(ChatRetrievalError::Transport {
        detail: "fixture".to_string(),
    });
    let mut source = ChatSource::new(channel.receiver, channel.terminal);

    assert!(matches!(
        source.next_message(),
        Err(ChatSourceError::Retrieval(
            ChatRetrievalError::Transport { .. }
        ))
    ));
    assert!(matches!(source.next_message(), Ok(None)));
}

#[test]
fn source_disconnected_without_terminal_error_yields_none_forever() {
    let channel = chat_channel();
    drop(channel.sender);
    let mut source = ChatSource::new(channel.receiver, channel.terminal);

    for _attempt in 0..3 {
        assert!(matches!(source.next_message(), Ok(None)));
    }
}

#[test]
fn chat_source_integrates_through_store_and_renderer_in_the_frame_pipeline() {
    let stop_flag = StopFlag::new();
    let (sink, events) = RecordingSink::fresh();
    let sink = sink.stop_after_writes(3, stop_flag.clone());
    let (clock, _sleeps) = FakeClock::fresh();
    let (renderer, inputs) = StubRenderer::fresh();
    let channel = chat_channel();
    channel
        .sender
        .send(ChatMessage::new("alice", "Hello!"))
        .expect("the bounded channel accepts");
    channel
        .sender
        .send(ChatMessage::new("bob", "Hi!"))
        .expect("the bounded channel accepts");
    let source = source_of(channel);

    let temp = tempfile::tempdir().expect("create temp dir");
    let mut config = valid_config();
    config.chat.log_file = temp.path().join("chat.log").to_string_lossy().into_owned();
    let settings = ChatStoreSettings::from_config(&config).expect("derive settings");
    let chat_store = ChatStore::new(settings).expect("open store");
    let pipeline = FramePipeline::new(PipelineParts {
        chat_store,
        renderer: Box::new(renderer),
        sink,
        source: Box::new(source),
        clock: Box::new(clock),
        pacing: PipelinePacing::raw(TEST_INTERVAL_NANOS, TEST_FRAME_BYTES),
        stop_flag,
    });
    pipeline.run().expect("the three-frame run completes");
    drop(temp);

    let rendered = inputs.borrow().clone();
    assert_eq!(
        rendered,
        [
            "alice: Hello!",
            "alice: Hello!\nbob: Hi!",
            "alice: Hello!\nbob: Hi!"
        ],
        "worker-fed messages reach the renderer through the store"
    );
    assert!(events.borrow().finish_called);
}
