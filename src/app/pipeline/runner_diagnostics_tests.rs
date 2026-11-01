//! End-to-end pipeline diagnostics pins: the REAL frame loop records the
//! dequeue/render/write cycle into the shared aggregate when a collector is
//! attached, and a `None` collector runs the identical loop with no artifact.

use std::path::Path;

use crate::app::diagnostics::diagnostics_test_fixtures::enabled_collector_for_tests;
use crate::app::pipeline::pacing::PipelinePacing;
use crate::app::pipeline::runner::{FramePipeline, PipelineParts};
use crate::app::pipeline::source_helpers::ScriptedSource;
use crate::app::pipeline::test_helpers::{
    FakeClock, RecordingSink, StubRenderer, TEST_FRAME_BYTES, TEST_INTERVAL_NANOS,
};
use crate::app::pipeline::StopFlag;
use crate::chat::{ChatMessage, ChatStore, ChatStoreSettings};
use crate::config::valid_config;

fn temp_store() -> (ChatStore, tempfile::TempDir) {
    let temp = tempfile::tempdir().expect("create temp dir");
    let mut config = valid_config();
    config.chat.log_file = temp.path().join("chat.log").to_string_lossy().into_owned();
    let settings = ChatStoreSettings::from_config(&config).expect("derive settings");
    let store = ChatStore::new(settings).expect("open store");
    (store, temp)
}

#[test]
fn pipeline_records_stage_cycle_for_one_message() {
    let stop_flag = StopFlag::new();
    let (sink, _events) = RecordingSink::fresh();
    let sink = sink.stop_after_writes(3, stop_flag.clone());
    let (clock, _sleeps) = FakeClock::fresh();
    let (chat_store, _temp) = temp_store();
    let (renderer, _inputs) = StubRenderer::fresh();
    let source = ScriptedSource::schedule(vec![
        ChatMessage::new("alice", "Hello!").with_message_id("m-1")
    ]);
    let harness = enabled_collector_for_tests();
    let pipeline = FramePipeline::new(PipelineParts {
        chat_store,
        renderer: Box::new(renderer),
        sink,
        source: Box::new(source),
        clock: Box::new(clock),
        pacing: PipelinePacing::raw(TEST_INTERVAL_NANOS, TEST_FRAME_BYTES),
        stop_flag,
        diagnostics: Some(harness.collector.clone()),
    });

    pipeline.run().expect("three-frame run completes");

    let snapshot = harness
        .state
        .lock()
        .expect("diagnostics state")
        .snapshot(harness.now());
    assert_eq!(snapshot.dequeued_total, 1);
    assert_eq!(snapshot.message_render_ns.count, 1);
    assert_eq!(snapshot.render_ns.count, 3);
    assert_eq!(snapshot.write_ns.count, 3);
    assert_eq!(snapshot.frame_interval_ns.count, 2);
}

#[test]
fn pipeline_with_none_collector_does_no_io_and_no_state() {
    let stop_flag = StopFlag::new();
    let (sink, _events) = RecordingSink::fresh();
    let sink = sink.stop_after_writes(2, stop_flag.clone());
    let (clock, _sleeps) = FakeClock::fresh();
    let (chat_store, temp) = temp_store();
    let (renderer, _inputs) = StubRenderer::fresh();
    let pipeline = FramePipeline::new(PipelineParts {
        chat_store,
        renderer: Box::new(renderer),
        sink,
        source: Box::new(ScriptedSource::schedule(Vec::new())),
        clock: Box::new(clock),
        pacing: PipelinePacing::raw(TEST_INTERVAL_NANOS, TEST_FRAME_BYTES),
        stop_flag,
        diagnostics: None,
    });

    pipeline.run().expect("two-frame run completes");

    assert!(
        !temp.path().join("diagnostics.log").exists(),
        "the disabled collector never creates a diagnostics file"
    );
    let produced: Vec<_> = std::fs::read_dir(temp.path())
        .expect("temp dir readable")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name())
        .filter(|name| name != Path::new("chat.log"))
        .collect();
    assert!(produced.is_empty(), "no other artifact: {produced:?}");
}
