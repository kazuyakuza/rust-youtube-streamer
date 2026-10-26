//! The delivery/poll half of the Phase-06 chat worker (AMENDMENT 2
//! split of `chat_worker`): the session's D-6 poll cycle (server-interval
//! honor loop, baseline discard, chat-ended grace, transient backoff),
//! dedup → convert → stop-aware send, the bounded stop-sliced sleeps, the
//! backoff delay, and the terminal-cell writer. Everything here is
//! stop-aware (D-9): sends are `try_send`-only and sleeps wake within one
//! 1 s slice, so neither a full channel nor a long server interval can
//! prevent bounded shutdown. Test builds never sleep (the zero-sleep
//! test rule).

use std::sync::mpsc::TrySendError;
use std::sync::Mutex;

use tracing::info;

use crate::app::pipeline::{ChatRetrievalError, StopFlag};
use crate::chat::ChatMessage;
use crate::youtube::{
    map_page, ChatListCommand, ChatTextMessage, LiveApiError, LiveChatMessageList,
};

use super::chat_failure::{is_transient_failure, terminal_retrieval_error};
use super::chat_worker::{
    ChatWorkerPlan, ChatWorkerSession, CHAT_BACKOFF_MAX_MS, CHAT_BASELINE_MESSAGE,
    CHAT_ENDED_MESSAGE, CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES, CHAT_SLEEP_SLICE_MS,
};

impl ChatWorkerSession {
    /// One D-6 poll cycle; `false` = the worker returns (stop, graceful
    /// end, or terminal cell write).
    pub(super) fn poll_cycle(&mut self) -> bool {
        if self.plan.stop_flag.is_requested() {
            return false;
        }
        let command = ChatListCommand {
            live_chat_id: self.live_chat_id.clone(),
            page_token: self.page_token.clone(),
        };
        match self.plan.transport.list_messages(&command) {
            Ok(page) => self.handle_page(page),
            Err(error) => self.handle_list_failure(error),
        }
    }

    fn handle_page(&mut self, page: LiveChatMessageList) -> bool {
        self.consecutive_failures = 0;
        self.poll_interval_ms = page
            .polling_interval_millis
            .filter(|ms| *ms > 0)
            .unwrap_or(self.poll_interval_ms);
        let outcome = map_page(&page);
        self.page_token = page.next_page_token;
        if self.baseline_pending {
            self.baseline_pending = false;
            info!(
                discarded_messages = outcome.messages.len(),
                "{}", CHAT_BASELINE_MESSAGE
            );
        } else {
            deliver(self, &outcome.messages);
        }
        if outcome.chat_ended {
            info!("{}", CHAT_ENDED_MESSAGE);
            return false;
        }
        sleep_in_stop_slices(self.poll_interval_ms, &self.plan.stop_flag);
        true
    }

    fn handle_list_failure(&mut self, error: LiveApiError) -> bool {
        if is_graceful_end(&error) {
            info!("{}", CHAT_ENDED_MESSAGE);
            return false;
        }
        if is_transient_failure(&error) {
            return self.retry_transient(error);
        }
        record_terminal(&self.plan.terminal, terminal_retrieval_error(&error));
        false
    }

    fn retry_transient(&mut self, error: LiveApiError) -> bool {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES {
            record_terminal(
                &self.plan.terminal,
                ChatRetrievalError::Transport {
                    detail: error.to_string(),
                },
            );
            return false;
        }
        let delay = backoff_delay_ms(self.consecutive_failures, self.poll_interval_ms);
        sleep_in_stop_slices(delay, &self.plan.stop_flag);
        true
    }
}

fn is_graceful_end(error: &LiveApiError) -> bool {
    matches!(error, LiveApiError::ChatEnded)
}

pub(super) fn deliver(session: &mut ChatWorkerSession, messages: &[ChatTextMessage]) {
    for carrier in messages {
        let unseen = match carrier.message_id.as_deref() {
            Some(id) => session.dedup.record(id),
            None => true,
        };
        if !unseen {
            continue;
        }
        let message = into_chat_message(carrier);
        send_stop_aware(&session.plan, message);
    }
}

/// The C-1 conversion point (record D-4/D-8): the app side builds
/// `ChatMessage`; normalization runs in `ChatMessage::new`.
fn into_chat_message(carrier: &ChatTextMessage) -> ChatMessage {
    let mut message = ChatMessage::new(&carrier.author, &carrier.text);
    if let Some(id) = carrier.message_id.as_deref() {
        message = message.with_message_id(id);
    }
    if let Some(seconds) = carrier.published_epoch {
        message = message.with_timestamp(Some(seconds));
    }
    message
}

/// Stop-aware bounded send (D-9): `try_send` only; a full channel waits a
/// stop-slice and retries; the stop flag or a disconnected receiver ends
/// the loop and the remaining page is dropped (never blocks shutdown).
fn send_stop_aware(plan: &ChatWorkerPlan, mut message: ChatMessage) {
    loop {
        if plan.stop_flag.is_requested() {
            return;
        }
        match plan.sender.try_send(message) {
            Ok(()) => return,
            Err(TrySendError::Full(returned)) => {
                message = returned;
                sleep_slice(CHAT_SLEEP_SLICE_MS);
            }
            Err(TrySendError::Disconnected(_)) => return,
        }
    }
}

/// Bounded stop-sliced sleep (D-6): ≤1 s slices, the flag checked BEFORE
/// each slice so a stop ends the wait within one slice (zero-sleep in
/// test builds; tests never sleep).
pub(super) fn sleep_in_stop_slices(total_ms: u64, stop_flag: &StopFlag) {
    let mut remaining_ms = total_ms;
    while remaining_ms > 0 {
        if stop_flag.is_requested() {
            return;
        }
        let slice_ms = remaining_ms.min(CHAT_SLEEP_SLICE_MS);
        sleep_slice(slice_ms);
        remaining_ms -= slice_ms;
    }
}

pub(super) fn backoff_delay_ms(attempt: u32, base_ms: u64) -> u64 {
    // Phase-06 Task 3: the D-6 pinned max/min body (record §4.3.7) — the
    // clippy-named allow keeps the plan's shape (plan §6(f)).
    #[allow(clippy::manual_clamp)]
    let base = base_ms.max(CHAT_SLEEP_SLICE_MS).min(CHAT_BACKOFF_MAX_MS);
    let doubled = base.saturating_mul(2u64.saturating_pow(attempt - 1));
    doubled.min(CHAT_BACKOFF_MAX_MS)
}

/// Stores the terminal failure exactly once (poisoned locks ignored — the
/// existing `if let Ok` convention).
pub(super) fn record_terminal(cell: &Mutex<Option<ChatRetrievalError>>, error: ChatRetrievalError) {
    if let Ok(mut recorded) = cell.lock() {
        *recorded = Some(error);
    }
}

#[cfg(not(test))]
fn sleep_slice(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

#[cfg(test)]
fn sleep_slice(_ms: u64) {}
