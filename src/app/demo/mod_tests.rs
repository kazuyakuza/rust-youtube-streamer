//! Handler-level demo tests: exit-code/Display mapping for demo-origin
//! failures, and one full composition run of the real demo timeline through
//! the shared frame pipeline with test doubles (no FFmpeg, no real sleeps).

use std::io;
use std::path::Path;

use crate::app::demo::schedule::{scheduled_messages_for_tests, DemoTimeline};
use crate::app::demo::source::DemoMessageSource;
use crate::app::demo::DemoError;
use crate::app::pipeline::{
    FakeClock, FramePipeline, FramePipelineError, InterruptHandlerError, PipelinePacing,
    PipelineParts, RecordingSink, StopFlag, StubRenderer, TEST_FRAME_BYTES, TEST_INTERVAL_NANOS,
};
use crate::chat::{ChatStore, ChatStoreSettings};
use crate::config::valid_config;
use crate::streaming::StreamingError;

/// Chat store for the composition test: rides the crate's `valid_config`
/// fixture with `chat.log_file` pointed at `<tempdir>/chat.log`.
fn temp_store() -> (ChatStore, tempfile::TempDir) {
    let temp = tempfile::tempdir().expect("create temp dir");
    let mut config = valid_config();
    config.chat.log_file = temp.path().join("chat.log").to_string_lossy().into_owned();
    let settings = ChatStoreSettings::from_config(&config).expect("derive settings");
    let store = ChatStore::new(settings).expect("open store");
    (store, temp)
}

#[test]
fn demo_origin_failures_map_to_the_startup_failure_code() {
    let interrupt_error =
        DemoError::Interrupt(InterruptHandlerError::test_failure("already taken"));
    assert_eq!(interrupt_error.exit_code(), 1);
    let interrupt_display = interrupt_error.to_string();
    assert!(interrupt_display.contains("unable to install the Ctrl+C interrupt handler"));
    assert!(interrupt_display.contains("already taken"));
    let pipeline_error =
        DemoError::Pipeline(FramePipelineError::Sink(StreamingError::WaitFailed {
            source: io::Error::other("wait failed"),
        }));
    assert_eq!(pipeline_error.exit_code(), 1);
    assert!(pipeline_error.to_string().contains("frame output failed"));
}

#[test]
fn demo_timeline_drives_the_frame_pipeline_to_full_completion() {
    let stop_flag = StopFlag::new();
    let timeline = DemoTimeline::raw(scheduled_messages_for_tests());
    let (sink, events) = RecordingSink::fresh();
    let (renderer, inputs) = StubRenderer::fresh();
    let (chat_store, _temp) = temp_store();
    let source = DemoMessageSource::new(
        DemoTimeline::raw(scheduled_messages_for_tests()),
        stop_flag.clone(),
    );
    let pipeline = FramePipeline::new(PipelineParts {
        chat_store,
        renderer: Box::new(renderer),
        sink,
        source: Box::new(source),
        clock: Box::new(FakeClock::fresh().0),
        pacing: PipelinePacing::raw(TEST_INTERVAL_NANOS, TEST_FRAME_BYTES),
        stop_flag,
    });
    let completed = pipeline.run().expect("demo timeline run finalizes");
    assert_eq!(completed.path(), Path::new("videos/test-recording.mp4"));
    let log = events.borrow();
    assert_eq!(log.writes.len() as u64, timeline.rendered_frame_count());
    assert!(log.finish_called);
    assert_eq!(
        log.finish_path.as_deref(),
        Some(Path::new("videos/test-recording.mp4"))
    );
    let recorded_inputs = inputs.borrow();
    assert_eq!(recorded_inputs.first().map(String::as_str), Some(""));
    let last_input = recorded_inputs.last().expect("rendered frames");
    assert_eq!(last_input.matches('\n').count(), 25);
    assert!(last_input.contains("charlie: demo message 03"));
    assert!(last_input.contains("demo message 28"));
    assert!(!last_input.contains("demo message 01"));
    assert!(!last_input.contains("demo message 02"));
    assert!(recorded_inputs.windows(2).any(|pair| pair[0] == pair[1]));
}
