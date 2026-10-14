//! The chat-free message source of the Phase-05 production run path
//! (plan D-T3-2 stage 9): the frame loop stays continuous and independent
//! of chat activity — it yields no message, ever. The chat-source seam
//! (`app::pipeline::source`) stays BYTE-unchanged for Phase 06, which
//! replaces this with a real YouTube chat listener; no chat source is
//! constructed anywhere in Phase 05 (user ruling, global plan §1.3).

use crate::app::pipeline::MessageSource;
use crate::chat::{ChatError, ChatMessage};

pub(super) struct ChatFreeSource;

impl MessageSource for ChatFreeSource {
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatError> {
        Ok(None)
    }
}
