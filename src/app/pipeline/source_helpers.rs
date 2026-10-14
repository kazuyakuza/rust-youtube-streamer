use std::collections::VecDeque;

use crate::app::pipeline::source::MessageSource;
use crate::chat::{ChatError, ChatMessage};

/// Message-source fake: scheduled messages in order; `failing` errors first.
pub(crate) struct ScriptedSource {
    pending: VecDeque<ChatMessage>,
    failure: Option<ChatError>,
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
            failure: Some(failure),
        }
    }
}

impl MessageSource for ScriptedSource {
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatError> {
        if let Some(failure) = self.failure.take() {
            return Err(failure);
        }
        Ok(self.pending.pop_front())
    }
}
