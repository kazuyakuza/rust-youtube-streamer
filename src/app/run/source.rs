//! The production chat-backed message source (Phase 06 Task 3; record
//! D-5/D-8/D-9): it owns the bounded channel's receiver + the worker's
//! shared terminal cell and drains NON-blocking (`try_recv`), so the
//! frame loop NEVER waits on chat: `Empty` → `Ok(None)` (frames keep
//! flowing through idle/slow/reconnecting chat), a queued message →
//! `Some(message)` (one per frame), `Disconnected` → consult the cell —
//! a stored retrieval error surfaces ONCE (`take`), then `Ok(None)`
//! forever; an empty cell (graceful `ChatEnded`/`offlineAt` end, or the
//! pre-spawn/post-shutdown states) yields `Ok(None)` forever.

use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};

use crate::app::pipeline::{ChatRetrievalError, ChatSourceError, MessageSource};
use crate::chat::ChatMessage;

pub(super) struct ChatSource {
    receiver: Receiver<ChatMessage>,
    terminal: Arc<Mutex<Option<ChatRetrievalError>>>,
}

impl ChatSource {
    pub(super) fn new(
        receiver: Receiver<ChatMessage>,
        terminal: Arc<Mutex<Option<ChatRetrievalError>>>,
    ) -> Self {
        Self { receiver, terminal }
    }

    fn take_terminal_failure(&self) -> Option<ChatRetrievalError> {
        self.terminal.lock().ok().and_then(|mut cell| cell.take())
    }
}

impl MessageSource for ChatSource {
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatSourceError> {
        match self.receiver.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => match self.take_terminal_failure() {
                Some(error) => Err(ChatSourceError::Retrieval(error)),
                None => Ok(None),
            },
        }
    }
}

#[cfg(test)]
#[path = "chat_source_tests.rs"]
mod chat_source_tests;
