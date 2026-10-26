//! Gate↔worker spawn contract (plan §5.4; registered from `chat_worker`
//! per AMENDMENT 2): a confirmed gate spawns AND joins the chat worker
//! (the D-9 spawn-at-confirmation pin), a cancelled gate never spawns it.
//! Offline only — fakes for both the O4 gate lookup and the chat seam.

use std::sync::Arc;

use crate::app::pipeline::StopFlag;
use crate::app::run::broadcast::public_watch_url;
use crate::app::run::chat_worker::runtime_chat_fakes::{
    chat_page, worker_plan, ScriptedRunChatTransport,
};
use crate::app::run::gate::{spawn_startup_gate, GateFinish, GatePlan};
use crate::app::run::runtime_test_fixtures::{
    stream_with_status, ScriptedTransport, BROADCAST_ID_FIXTURE,
};
use crate::youtube::WaitCanceller;

use super::chat_channel;
use super::chat_worker_tests::{lookup_found, OFFLINE_AT};

const FIXTURE_STREAM_ID: &str = "stream-id-fixture";

fn gate_plan(transport: Arc<ScriptedTransport>, cancellation: WaitCanceller) -> GatePlan {
    GatePlan {
        transport,
        cancellation,
        stream_id: FIXTURE_STREAM_ID.to_string(),
        watch_url: public_watch_url(BROADCAST_ID_FIXTURE),
        stop_flag: StopFlag::new(),
        chat_worker: None,
    }
}

#[test]
fn confirmed_gate_spawns_and_joins_the_chat_worker() {
    let gate_transport = Arc::new(ScriptedTransport::default());
    gate_transport.script_gate_lookup(Ok(Some(stream_with_status("active"))));
    let chat_fake = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&chat_fake);
    chat_fake.script_list(Ok(chat_page(vec![], None, None, Some(OFFLINE_AT))));
    let channel = chat_channel();
    let mut plan = gate_plan(gate_transport, WaitCanceller::new());
    plan.chat_worker = Some(worker_plan(chat_fake.clone(), &channel));

    let finish = spawn_startup_gate(plan).finish();

    assert!(
        matches!(finish, GateFinish::Confirmed),
        "the O4 confirmation is unchanged"
    );
    assert_eq!(
        chat_fake.operations(),
        ["lookup_broadcast_chat_id", "list_messages"],
        "the worker ran post-confirmation and joined in finish()"
    );
}

#[test]
fn cancelled_gate_never_spawns_the_chat_worker() {
    let gate_transport = Arc::new(ScriptedTransport::default());
    let cancellation = WaitCanceller::new();
    cancellation.request_cancel();
    let chat_fake = Arc::new(ScriptedRunChatTransport::default());
    let channel = chat_channel();
    let mut plan = gate_plan(gate_transport, cancellation);
    plan.chat_worker = Some(worker_plan(chat_fake.clone(), &channel));

    let finish = spawn_startup_gate(plan).finish();

    assert!(matches!(finish, GateFinish::Cancelled));
    assert!(
        chat_fake.operations().is_empty(),
        "no worker is spawned outside the Confirmed arm"
    );
}
