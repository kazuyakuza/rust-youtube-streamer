use std::io;
use std::path::{Path, PathBuf};

use crate::app::pipeline::error::FramePipelineError;
use crate::app::pipeline::interrupt::StopFlag;
use crate::app::pipeline::pacing::PipelinePacing;
use crate::app::pipeline::runner::{FramePipeline, PipelineParts};
use crate::app::pipeline::test_helpers::*;
use crate::chat::{ChatError, ChatMessage, ChatStore, ChatStoreSettings};
use crate::config::valid_config;
use crate::renderer::RenderError;
use crate::streaming::CompletedOutput;

/// Chat store for pipeline tests: rides the crate's `valid_config` fixture
/// with `chat.log_file` pointed at `<tempdir>/chat.log`, so scripted
/// messages stay far below the visible capacity.
fn temp_store() -> (ChatStore, tempfile::TempDir) {
    let temp = tempfile::tempdir().expect("create temp dir");
    let mut config = valid_config();
    config.chat.log_file = temp.path().join("chat.log").to_string_lossy().into_owned();
    let settings = ChatStoreSettings::from_config(&config).expect("derive settings");
    let store = ChatStore::new(settings).expect("open store");
    (store, temp)
}

#[test]
fn run_returns_completed_video_without_writing_when_stopped_before_start() {
    let stop_flag = StopFlag::new();
    stop_flag.request_stop();
    let (sink, events) = RecordingSink::fresh();
    let (clock, sleeps) = FakeClock::fresh();
    let (chat_store, _temp) = temp_store();
    let (renderer, _inputs) = StubRenderer::fresh();
    let pipeline = FramePipeline::new(PipelineParts {
        chat_store,
        renderer: Box::new(renderer),
        sink,
        source: Box::new(ScriptedSource::schedule(Vec::new())),
        clock: Box::new(clock),
        pacing: PipelinePacing::raw(TEST_INTERVAL_NANOS, TEST_FRAME_BYTES),
        stop_flag,
    });
    let CompletedOutput::Recording(completed) =
        pipeline.run().expect("stopped run finalizes the sink")
    else {
        panic!("the stopped run must complete as a local recording");
    };
    assert_eq!(completed.path(), Path::new("videos/test-recording.mp4"));
    let log = events.borrow();
    assert!(log.writes.is_empty());
    assert!(log.finish_called);
    assert_eq!(
        log.finish_path.as_deref(),
        Some(Path::new("videos/test-recording.mp4"))
    );
    assert!(sleeps.borrow().is_empty());
}

#[test]
fn run_emits_immediate_empty_first_frames_with_exact_byte_counts() {
    let stop_flag = StopFlag::new();
    let (sink, events) = RecordingSink::fresh();
    let sink = sink.stop_after_writes(3, stop_flag.clone());
    let (clock, sleeps) = FakeClock::fresh();
    let (chat_store, _temp) = temp_store();
    let (renderer, inputs) = StubRenderer::fresh();
    let pipeline = FramePipeline::new(PipelineParts {
        chat_store,
        renderer: Box::new(renderer),
        sink,
        source: Box::new(ScriptedSource::schedule(Vec::new())),
        clock: Box::new(clock),
        pacing: PipelinePacing::raw(TEST_INTERVAL_NANOS, TEST_FRAME_BYTES),
        stop_flag,
    });
    pipeline.run().expect("three-frame run completes");
    let log = events.borrow();
    assert_eq!(log.writes.len(), 3);
    for write in &log.writes {
        assert_eq!(write.len(), TEST_FRAME_BYTES);
    }
    assert_eq!(*inputs.borrow(), ["", "", ""]);
    assert_eq!(*sleeps.borrow(), [TEST_INTERVAL_NANOS; 3]);
    assert!(log.finish_called);
}

#[test]
fn run_rotates_new_messages_into_visible_text() {
    let stop_flag = StopFlag::new();
    let (sink, events) = RecordingSink::fresh();
    let sink = sink.stop_after_writes(3, stop_flag.clone());
    let (clock, _sleeps) = FakeClock::fresh();
    let (chat_store, _temp) = temp_store();
    let (renderer, inputs) = StubRenderer::fresh();
    let source = ScriptedSource::schedule(vec![
        ChatMessage::new("alice", "Hello!"),
        ChatMessage::new("bob", "Hi!"),
    ]);
    let pipeline = FramePipeline::new(PipelineParts {
        chat_store,
        renderer: Box::new(renderer),
        sink,
        source: Box::new(source),
        clock: Box::new(clock),
        pacing: PipelinePacing::raw(TEST_INTERVAL_NANOS, TEST_FRAME_BYTES),
        stop_flag,
    });
    pipeline.run().expect("three-frame run completes");
    let both = "alice: Hello!\nbob: Hi!";
    assert_eq!(*inputs.borrow(), ["alice: Hello!", both, both]);
    assert!(events.borrow().finish_called);
}

#[test]
fn run_propagates_render_failure_without_finalizing_sink() {
    let stop_flag = StopFlag::new();
    let (sink, events) = RecordingSink::fresh();
    let (clock, _sleeps) = FakeClock::fresh();
    let (chat_store, _temp) = temp_store();
    let renderer = StubRenderer::failing(RenderError::Geometry {
        width: 2,
        height: 2,
    });
    let pipeline = FramePipeline::new(PipelineParts {
        chat_store,
        renderer: Box::new(renderer),
        sink,
        source: Box::new(ScriptedSource::schedule(Vec::new())),
        clock: Box::new(clock),
        pacing: PipelinePacing::raw(TEST_INTERVAL_NANOS, TEST_FRAME_BYTES),
        stop_flag,
    });
    let Err(error) = pipeline.run() else {
        panic!("render failure must surface");
    };
    assert!(matches!(error, FramePipelineError::Render(_)));
    let log = events.borrow();
    assert!(log.writes.is_empty());
    assert!(!log.finish_called);
}

#[test]
fn run_propagates_sink_write_failure_without_finalizing() {
    let stop_flag = StopFlag::new();
    let (sink, events) = FailingWriteSink::fresh();
    let (clock, _sleeps) = FakeClock::fresh();
    let (chat_store, _temp) = temp_store();
    let (renderer, _inputs) = StubRenderer::fresh();
    let pipeline = FramePipeline::new(PipelineParts {
        chat_store,
        renderer: Box::new(renderer),
        sink,
        source: Box::new(ScriptedSource::schedule(Vec::new())),
        clock: Box::new(clock),
        pacing: PipelinePacing::raw(TEST_INTERVAL_NANOS, TEST_FRAME_BYTES),
        stop_flag,
    });
    let Err(error) = pipeline.run() else {
        panic!("sink write failure must surface");
    };
    let display = error.to_string();
    assert!(display.contains("frame output failed"));
    assert!(display.contains("exited while receiving frames"));
    assert!(!events.borrow().finish_called);
}

#[test]
fn run_propagates_message_source_failure_without_rendering() {
    let stop_flag = StopFlag::new();
    let (sink, events) = RecordingSink::fresh();
    let (clock, _sleeps) = FakeClock::fresh();
    let (chat_store, _temp) = temp_store();
    let (renderer, inputs) = StubRenderer::fresh();
    let source = ScriptedSource::failing(ChatError::LogAppend {
        path: PathBuf::from("chat.log"),
        source: io::Error::other("log write failed"),
    });
    let pipeline = FramePipeline::new(PipelineParts {
        chat_store,
        renderer: Box::new(renderer),
        sink,
        source: Box::new(source),
        clock: Box::new(clock),
        pacing: PipelinePacing::raw(TEST_INTERVAL_NANOS, TEST_FRAME_BYTES),
        stop_flag,
    });
    let Err(error) = pipeline.run() else {
        panic!("source failure must surface");
    };
    assert!(matches!(error, FramePipelineError::Chat(_)));
    assert!(inputs.borrow().is_empty());
    assert!(!events.borrow().finish_called);
}
