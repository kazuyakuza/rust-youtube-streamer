//! The resumable streamList loop of the Phase-07 chat worker (§A.1..A.4):
//! establish → consume → bounded transient re-establishment with the last
//! token; history discard keyed on the first delivered batch; worker-owned
//! token/dedup/history state; the only cross-thread outputs are the
//! bounded channel, the terminal cell, and the stop-flag read.

use std::time::Duration;

use tracing::info;

use crate::youtube::{ChatDedupWindow, ChatStreamRequest, LiveApiError, LiveChatStreamSession};

use super::chat_worker::{ChatWorkerPlan, CHAT_ENDED_MESSAGE};

pub(super) const CHAT_STOP_TICK_SLICE_MS: u64 = 1_000;

#[cfg(not(test))]
fn stop_tick_slice_ms() -> u64 {
    CHAT_STOP_TICK_SLICE_MS
}

#[cfg(test)]
fn stop_tick_slice_ms() -> u64 {
    0
}

/// The clean close carries no status to classify (record §A.6); this is the
/// ONLY sanctioned non-secret detail for its budget-exhaustion terminal.
pub(super) const STREAM_CLEAN_CLOSE_DETAIL: &str =
    "the stream connection closed without an error status";

pub(super) struct StreamWorker {
    pub(super) plan: ChatWorkerPlan,
    pub(super) dedup: ChatDedupWindow,
    pub(super) live_chat_id: String,
    pub(super) page_token: Option<String>,
    pub(super) history_pending: bool,
    pub(super) baseline_logged: bool,
    pub(super) delivered_batches: u32,
    pub(super) consecutive_failures: u32,
}

impl StreamWorker {
    pub(super) fn new(plan: ChatWorkerPlan, live_chat_id: String) -> Self {
        Self {
            plan,
            dedup: ChatDedupWindow::new(),
            live_chat_id,
            page_token: None,
            history_pending: true,
            baseline_logged: false,
            delivered_batches: 0,
            consecutive_failures: 0,
        }
    }
}

/// How one consuming session ended (record §A.6 Option semantics).
enum ConsumeVerdict {
    GracefulEnd,
    Stopped,
    CleanClose,
    Failed(LiveApiError),
}

/// The re-establishment loop (P7-5): establish → consume → classify the
/// end; transient paths count the failure, wait the capped backoff, and
/// re-establish with the last token. Terminal/graceful/stop exit.
pub(super) async fn run_stream_loop(mut worker: StreamWorker) {
    loop {
        if worker.plan.stop_flag.is_requested() {
            return;
        }
        let session = match worker.establish().await {
            Ok(session) => session,
            Err(error) => {
                if worker.handle_failure(error) {
                    return;
                }
                continue;
            }
        };
        match worker.consume(session).await {
            ConsumeVerdict::GracefulEnd => {
                info!("{}", CHAT_ENDED_MESSAGE);
                return;
            }
            ConsumeVerdict::Stopped => return,
            ConsumeVerdict::Failed(error) => {
                if worker.handle_failure(error) {
                    return;
                }
                continue;
            }
            ConsumeVerdict::CleanClose => {
                if worker.handle_clean_close() {
                    return;
                }
                continue;
            }
        }
    }
}

impl StreamWorker {
    /// One quota-billed open (P7-5 step 1): the chat id ALWAYS; the retained
    /// token ONLY when one exists (never fabricated).
    async fn establish(&mut self) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError> {
        self.plan
            .stream
            .open(ChatStreamRequest {
                live_chat_id: self.live_chat_id.clone(),
                page_token: self.page_token.clone(),
            })
            .await
    }

    /// The consuming loop (P7-10 as adjudicated in §A.7.1): biased select over
    /// the pending batch read and the one-slice stop-tick branch; the SAME
    /// pending future is re-polled on every tick (never restarted).
    async fn consume(&mut self, mut session: Box<dyn LiveChatStreamSession>) -> ConsumeVerdict {
        loop {
            if self.plan.stop_flag.is_requested() {
                return ConsumeVerdict::Stopped;
            }
            let delivery = tokio::select! {
                biased;
                batch = session.next_batch() => match batch {
                    Some(delivery) => delivery,
                    None => return ConsumeVerdict::CleanClose,
                },
                _ = tokio::time::sleep(Duration::from_millis(stop_tick_slice_ms())) => {
                    if self.plan.stop_flag.is_requested() {
                        return ConsumeVerdict::Stopped;
                    }
                    continue;
                }
            };
            match delivery {
                Ok(batch) => {
                    self.delivered_batches = self.delivered_batches.saturating_add(1);
                    self.consecutive_failures = 0;
                    let offline_ended = batch.offline_ended;
                    self.process_batch(batch);
                    if offline_ended {
                        return ConsumeVerdict::GracefulEnd;
                    }
                }
                Err(error) => return ConsumeVerdict::Failed(error),
            }
        }
    }
}
