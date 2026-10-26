//! Seam-fallout pin (plan §5.1 item 5, AMENDMENT 1 ruling 1): a typed
//! chat-retrieval failure propagates through the runner as
//! `FramePipelineError::ChatRetrieval` — no render input, no sink finish.

use crate::app::pipeline::error::FramePipelineError;
use crate::app::pipeline::interrupt::StopFlag;
use crate::app::pipeline::pacing::PipelinePacing;
use crate::app::pipeline::runner::{FramePipeline, PipelineParts};
use crate::app::pipeline::source_helpers::*;
use crate::app::pipeline::test_helpers::*;
use crate::app::pipeline::ChatRetrievalError;
use crate::chat::ChatStoreSettings;
use crate::config::valid_config;

#[test]
fn run_maps_retrieval_failure_to_the_chat_retrieval_variant() {
    let stop_flag = StopFlag::new();
    let (sink, events) = RecordingSink::fresh();
    let (clock, _sleeps) = FakeClock::fresh();
    let (renderer, inputs) = StubRenderer::fresh();
    let temp = tempfile::tempdir().expect("create temp dir");
    let mut config = valid_config();
    config.chat.log_file = temp.path().join("chat.log").to_string_lossy().into_owned();
    let settings = ChatStoreSettings::from_config(&config).expect("derive settings");
    let chat_store = crate::chat::ChatStore::new(settings).expect("open store");
    let source = ScriptedSource::failing_retrieval(ChatRetrievalError::Transport {
        detail: "fixture".into(),
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
        panic!("the retrieval failure must surface");
    };

    assert!(matches!(error, FramePipelineError::ChatRetrieval(_)));
    assert!(inputs.borrow().is_empty(), "no render input");
    assert!(!events.borrow().finish_called, "no sink finish");
}
