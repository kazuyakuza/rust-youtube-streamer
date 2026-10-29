//! Roster C — the burst/backpressure pin (TODO Task 4 coverage audit §A.5):
//! a live burst lands into a FULL 64-entry channel; the stop-aware
//! `try_send` retain-retry must retain every item and deliver them in
//! order — nothing silently dropped, no reorder. Offline only; zero
//! sleeps (the test-build `sleep_slice` is a no-op, so retries spin until
//! the draining receiver frees capacity — deterministic progress).

use crate::app::run::chat_worker::runtime_chat_fakes::{worker_plan, ChatFakes};
use crate::app::run::chat_worker::{spawn_chat_worker, CHAT_CHANNEL_CAPACITY};
use crate::chat::ChatMessage;
use crate::youtube::{response_offline, response_plain, text_item, yielded};

use super::chat_channel;
use super::chat_worker_tests::{drain, lookup_found, terminal_of};

#[test]
fn worker_full_channel_retains_the_burst_without_loss_or_reorder() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.script_token_batch_session();
    fakes.stream_ledger.script_session(vec![
        Some(Ok(yielded(&response_plain(vec![
            text_item("burst-1"),
            text_item("burst-2"),
        ])))),
        Some(Ok(yielded(&response_offline(vec![])))),
    ]);
    let channel = chat_channel();
    for index in 0..CHAT_CHANNEL_CAPACITY {
        let filler = ChatMessage::new(&format!("prefill-{index}"), "capacity");
        channel
            .sender
            .send(filler)
            .expect("the fresh channel holds the prefill");
    }
    let plan = worker_plan(&fakes, &channel);
    let handle = spawn_chat_worker(plan);

    let total = CHAT_CHANNEL_CAPACITY + 2;
    let mut drained: Vec<ChatMessage> = Vec::new();
    while drained.len() < total {
        drained.push(
            channel
                .receiver
                .recv()
                .expect("the pinned burst drains message by message"),
        );
    }
    handle.join().expect("the worker thread joins");

    for (index, message) in drained.iter().enumerate().take(CHAT_CHANNEL_CAPACITY) {
        assert_eq!(
            message.to_string(),
            format!("prefill-{index}: capacity"),
            "the prefill drains first, in order"
        );
    }
    let burst_one = format!("{:?}", drained[CHAT_CHANNEL_CAPACITY]);
    let burst_two = format!("{:?}", drained[CHAT_CHANNEL_CAPACITY + 1]);
    assert!(
        burst_one.contains("burst-1"),
        "the first burst item arrives: {burst_one}"
    );
    assert!(
        burst_two.contains("burst-2"),
        "the second burst item arrives: {burst_two}"
    );
    assert!(
        terminal_of(&channel).is_none(),
        "a retained burst is not an error"
    );
    assert!(
        drain(&channel.receiver).is_empty(),
        "nothing extra is queued beyond the pinned total"
    );
}
