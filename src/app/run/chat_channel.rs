//! The bounded worker/frame-loop channel bundle and the ONE confined Tokio
//! runtime (split from `chat_worker` per the line-cap precedent): the frame
//! side receives a 64-entry `SyncSender`/`Receiver` pair plus the shared
//! terminal cell, and the worker thread owns the current-thread runtime.

use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};

use crate::app::pipeline::ChatRetrievalError;
use crate::chat::ChatMessage;

use super::CHAT_CHANNEL_CAPACITY;

/// The bounded channel bundle: the frame-loop side (source) and the
/// worker side (plan) of one 64-entry channel plus the shared terminal cell.
pub(in crate::app::run) struct ChatChannel {
    pub(in crate::app::run) sender: SyncSender<ChatMessage>,
    pub(in crate::app::run) receiver: Receiver<ChatMessage>,
    pub(in crate::app::run) terminal: Arc<Mutex<Option<ChatRetrievalError>>>,
}

pub(in crate::app::run) fn chat_channel() -> ChatChannel {
    let (sender, receiver) = sync_channel(CHAT_CHANNEL_CAPACITY);
    ChatChannel {
        sender,
        receiver,
        terminal: Arc::new(Mutex::new(None)),
    }
}

/// ONE runtime instance for the whole app (record §10): the current-thread
/// scheduler confined to this worker thread, built once per run.
pub(super) fn chat_runtime() -> Result<tokio::runtime::Runtime, ()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| ())
}
