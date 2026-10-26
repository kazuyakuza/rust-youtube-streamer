use crate::app::pipeline::error::FramePipelineError;
use crate::app::pipeline::ChatRetrievalError;
use std::error::Error;

#[test]
fn frame_size_mismatch_display_states_expected_and_actual_byte_counts() {
    let error = FramePipelineError::FrameSizeMismatch {
        expected: 12,
        actual: 9,
    };
    assert_eq!(
        error.to_string(),
        "rendered frame holds 9 bytes but the pipeline requires exactly 12 (width*height*3)"
    );
}

#[test]
fn frame_byte_overrun_display_states_offending_dimensions() {
    let error = FramePipelineError::FrameByteOverrun {
        width: u32::MAX,
        height: u32::MAX,
    };
    assert_eq!(
        error.to_string(),
        "frame byte count for 4294967295x4294967295 overflows usize; the pipeline refused to start"
    );
}

#[test]
fn chat_retrieval_pipeline_error_display_and_source_are_pinned() {
    let error = FramePipelineError::ChatRetrieval(ChatRetrievalError::ChatUnavailable {
        detail: "X".to_string(),
    });
    assert_eq!(
        error.to_string(),
        "chat retrieval failed: live chat is unavailable: X"
    );
    assert!(error.source().is_some());
}
