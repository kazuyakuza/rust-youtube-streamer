use std::collections::VecDeque;

use crate::app::pipeline::source::MessageSource;
use crate::app::pipeline::source_error::{ChatRetrievalError, ChatSourceError};
use crate::chat::{ChatError, ChatMessage};

/// Message-source fake: scheduled messages in order; `failing` errors first.
pub(crate) struct ScriptedSource {
    pending: VecDeque<ChatMessage>,
    failure: Option<ChatSourceError>,
}

impl ScriptedSource {
    pub(crate) fn schedule(messages: Vec<ChatMessage>) -> Self {
        Self {
            pending: messages.into_iter().collect(),
            failure: None,
        }
    }

    pub(crate) fn failing(failure: ChatError) -> Self {
        Self {
            pending: VecDeque::new(),
            failure: Some(ChatSourceError::Chat(failure)),
        }
    }

    pub(crate) fn failing_retrieval(failure: ChatRetrievalError) -> Self {
        Self {
            pending: VecDeque::new(),
            failure: Some(ChatSourceError::Retrieval(failure)),
        }
    }
}

impl MessageSource for ScriptedSource {
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatSourceError> {
        if let Some(failure) = self.failure.take() {
            return Err(failure);
        }
        Ok(self.pending.pop_front())
    }
}
