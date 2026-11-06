//! The batch delivery and failure half of the Phase-07 chat worker (the
//! AMENDMENT-2 split precedent): the §A.3 history/baseline rule, the
//! dedup → convert → stop-aware `try_send` delivery moved verbatim from the
//! retired poll delivery, the bounded stop-sliced sleeps, the capped
//! doubling backoff, the terminal-cell writer, and the §A.5 failure policy.
//! Everything here is stop-aware: sends are `try_send`-only and sleeps wake
//! within one 1 s slice, so neither a full channel nor a stalled read can
//! prevent bounded shutdown. Test builds never sleep (the zero-sleep test
//! rule).

use std::sync::mpsc::TrySendError;

use tracing::info;

use crate::chat::ChatMessage;
use crate::youtube::{ChatTextMessage, LiveApiError, YieldedChatStreamBatch};

use super::chat_failure::{is_transient_failure, terminal_retrieval_error};
use super::chat_stream_worker::{StreamWorker, STREAM_CLEAN_CLOSE_DETAIL};
use super::chat_worker::{
    CHAT_BASELINE_MESSAGE, CHAT_ENDED_MESSAGE, CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES,
    CHAT_SLEEP_SLICE_MS,
};

#[path = "chat_worker_helpers.rs"]
mod chat_worker_helpers;
pub(super) use chat_worker_helpers::{
    backoff_delay_ms, record_terminal, sleep_in_stop_slices, sleep_slice, transport_terminal,
    CHAT_WORKER_PANIC_DETAIL,
};

impl StreamWorker {
    /// §A.3 exact rule: a history-pending batch is discarded whole (the
    /// token it carries is retained and clears the flag); the ONE-ever
    /// baseline line charges on the first discarded batch; every later
    /// batch is the live per-item path.
    pub(super) fn process_batch(&mut self, batch: YieldedChatStreamBatch) {
        if self.history_pending {
            self.retain_token(batch.next_page_token);
            if !self.baseline_logged {
                self.baseline_logged = true;
                info!(
                    discarded_messages = batch.messages.len(),
                    "{}", CHAT_BASELINE_MESSAGE
                );
            }
            return;
        }
        self.retain_token(batch.next_page_token);
        if let Some(diagnostics) = &self.plan.diagnostics {
            diagnostics.report_batch_received(batch.messages.len() as u64);
        }
        self.deliver(batch.messages);
    }

    /// P7-7: the resume token is the first retained `nextPageToken`; a
    /// delivered token clears the pending-history flag exactly once.
    fn retain_token(&mut self, next_page_token: Option<String>) {
        if next_page_token.is_some() {
            self.page_token = next_page_token;
            self.history_pending = false;
        }
    }

    /// The greedy per-item path MOVED VERBATIM from the retired poll
    /// delivery (dedup → convert → stop-aware try_send); `None` ids are
    /// always-unseen (the mapper can produce them — stream roster pin 12).
    fn deliver(&mut self, messages: Vec<ChatTextMessage>) {
        for carrier in messages {
            let unseen = match carrier.message_id.as_deref() {
                Some(id) => self.dedup.record(id),
                None => true,
            };
            if !unseen {
                continue;
            }
            if let Some(diagnostics) = &self.plan.diagnostics {
                diagnostics.note_worker_receipt();
            }
            let message = into_chat_message(&carrier);
            self.send_stop_aware(message);
        }
    }

    /// Stop-aware bounded send (P7-9 byte-unchanged D-9 shape): try_send
    /// only; full → retain and retry once per slice; stop/disconnect →
    /// return (never blocks shutdown).
    fn send_stop_aware(&self, mut message: ChatMessage) {
        loop {
            if self.plan.stop_flag.is_requested() {
                return;
            }
            let reported_correlation = self
                .plan
                .diagnostics
                .as_ref()
                .and_then(|_| message.message_id().map(str::to_owned));
            let provider_stamp = self
                .plan
                .diagnostics
                .as_ref()
                .and_then(|_| message.timestamp());
            match self.plan.sender.try_send(message) {
                Ok(()) => {
                    let correlation = reported_correlation.as_deref();
                    self.report_enqueue_success_if_enabled(correlation, provider_stamp);
                    return;
                }
                Err(TrySendError::Full(returned)) => {
                    self.report_enqueue_full_if_enabled(&returned);
                    message = returned;
                    sleep_slice(CHAT_SLEEP_SLICE_MS);
                }
                Err(TrySendError::Disconnected(_)) => return,
            }
        }
    }

    fn report_enqueue_success_if_enabled(&self, correlation: Option<&str>, stamp: Option<u64>) {
        if let Some(diagnostics) = &self.plan.diagnostics {
            diagnostics.report_enqueue_success(correlation, stamp);
        }
    }

    fn report_enqueue_full_if_enabled(&self, message: &ChatMessage) {
        if let Some(diagnostics) = &self.plan.diagnostics {
            diagnostics.report_enqueue_full(message.message_id());
        }
    }
}

/// §A.5: FAILED_PRECONDITION is graceful EXACTLY when the chat had been
/// alive long enough to deliver a batch (named single-section predicate).
fn is_mid_session_chat_end(error: &LiveApiError, delivered_batches: u32) -> bool {
    matches!(error, LiveApiError::ChatPrecondition { .. }) && delivered_batches > 0
}

impl StreamWorker {
    /// One establishment/consume failure; `true` = the loop returns
    /// (terminal written, or graceful end). `is_transient_failure` STAYS
    /// CLOSED (record §7) — no arm is added.
    pub(super) fn handle_failure(&mut self, error: LiveApiError) -> bool {
        if is_mid_session_chat_end(&error, self.delivered_batches) {
            info!("{}", CHAT_ENDED_MESSAGE);
            return true;
        }
        if !is_transient_failure(&error) {
            record_terminal(&self.plan.terminal, terminal_retrieval_error(&error));
            return true;
        }
        self.await_reestablishment(error.to_string())
    }

    /// Clean close = the transient re-establishment arm (§A.4/A.5).
    pub(super) fn handle_clean_close(&mut self) -> bool {
        self.await_reestablishment(STREAM_CLEAN_CLOSE_DETAIL.to_string())
    }

    /// The shared transient path (P7-5.3): budget exhaustion writes the
    /// Transport terminal keyed on the LAST transient detail; otherwise the
    /// capped 1 s × 2^(n−1) backoff precedes re-establishment. The poll
    /// worker's budget test (>=) is kept byte-exact.
    fn await_reestablishment(&mut self, last_detail: String) -> bool {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES {
            record_terminal(&self.plan.terminal, transport_terminal(last_detail));
            return true;
        }
        let delay = backoff_delay_ms(self.consecutive_failures, CHAT_SLEEP_SLICE_MS);
        if let Some(diagnostics) = &self.plan.diagnostics {
            diagnostics.report_reconnect(delay);
        }
        sleep_in_stop_slices(delay, &self.plan.stop_flag);
        false
    }
}

/// The C-1 conversion point (record D-4/D-8) — byte-verbatim from the
/// retired file: the app side builds `ChatMessage`; normalization runs in
/// `ChatMessage::new`.
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

#[cfg(test)]
#[path = "chat_delivery_diagnostics_tests.rs"]
mod chat_delivery_diagnostics_tests;

#[cfg(test)]
#[path = "latency_load_fixtures.rs"]
mod latency_load_fixtures;
