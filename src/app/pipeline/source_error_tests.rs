//! Seam-widening pins (plan §5.1): the three exact `ChatRetrievalError`
//! Display lines stay stable and secret-free, `ChatSourceError` delegates
//! both classes verbatim, and the pure mapper preserves the class.

use crate::app::pipeline::source_error::{
    pipeline_source_failure, ChatRetrievalError, ChatSourceError,
};
use crate::app::pipeline::FramePipelineError;
use crate::chat::ChatError;
use std::io;

#[test]
fn chat_retrieval_error_display_lines_are_stable_and_secret_free() {
    let unavailable = ChatRetrievalError::ChatUnavailable {
        detail: "fixture-detail".to_string(),
    }
    .to_string();
    assert_eq!(unavailable, "live chat is unavailable: fixture-detail");
    let quota = ChatRetrievalError::Quota {
        detail: "fixture-detail".to_string(),
    }
    .to_string();
    assert_eq!(
        quota,
        "live chat retrieval stopped by YouTube quota: fixture-detail"
    );
    let transport = ChatRetrievalError::Transport {
        detail: "fixture-detail".to_string(),
    }
    .to_string();
    assert_eq!(
        transport,
        "live chat transport failed after repeated attempts: fixture-detail"
    );
    for rendered in [unavailable, quota, transport] {
        assert!(!rendered.contains("rtmps"), "no RTMPS in: {rendered}");
        assert!(!rendered.contains("Bearer"), "no Bearer in: {rendered}");
        assert!(!rendered.contains("token"), "no token in: {rendered}");
    }
}

#[test]
fn chat_source_error_delegates_both_classes_verbatim() {
    let chat = ChatError::LogAppend {
        path: std::path::PathBuf::from("chat.log"),
        source: io::Error::other("log write failed"),
    };
    let chat_line = chat.to_string();
    let wrapped = ChatSourceError::Chat(chat);
    assert_eq!(wrapped.to_string(), chat_line);
    assert!(std::error::Error::source(&wrapped).is_some());

    let retrieval = ChatRetrievalError::Transport {
        detail: "fixture".to_string(),
    };
    let retrieval_line = retrieval.to_string();
    let wrapped = ChatSourceError::Retrieval(retrieval);
    assert_eq!(wrapped.to_string(), retrieval_line);
    assert!(std::error::Error::source(&wrapped).is_some());
}

#[test]
fn pipeline_source_failure_maps_chat_and_retrieval_classes() {
    let chat = ChatSourceError::Chat(ChatError::LogAppend {
        path: std::path::PathBuf::from("chat.log"),
        source: io::Error::other("log write failed"),
    });
    assert!(matches!(
        pipeline_source_failure(chat),
        FramePipelineError::Chat(_)
    ));

    let retrieval = ChatSourceError::Retrieval(ChatRetrievalError::Transport {
        detail: "fixture".to_string(),
    });
    assert!(matches!(
        pipeline_source_failure(retrieval),
        FramePipelineError::ChatRetrieval(_)
    ));
}
