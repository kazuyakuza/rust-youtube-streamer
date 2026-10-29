//! Gate↔worker spawn contract (plan §E.4; registered from `chat_worker`):
//! a confirmed gate spawns AND joins the chat worker (the D-9
//! spawn-at-confirmation pin), a cancelled gate never spawns it. Offline
//! only — fakes for both the O4 gate lookup and the chat seams.

use std::sync::Arc;

use crate::app::pipeline::StopFlag;
use crate::app::run::broadcast::public_watch_url;
use crate::app::run::chat_worker::runtime_chat_fakes::{worker_plan, ChatFakes};
use crate::app::run::gate::{spawn_startup_gate, GateFinish, GatePlan};
use crate::app::run::runtime_test_fixtures::{
    stream_with_status, ScriptedTransport, BROADCAST_ID_FIXTURE,
};
use crate::youtube::WaitCanceller;

use super::chat_channel;
use super::chat_worker_tests::lookup_found;

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
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.script_graceful_end_session();
    let channel = chat_channel();
    let mut plan = gate_plan(gate_transport, WaitCanceller::new());
    plan.chat_worker = Some(worker_plan(&fakes, &channel));

    let finish = spawn_startup_gate(plan).finish();

    assert!(
        matches!(finish, GateFinish::Confirmed),
        "the O4 confirmation is unchanged"
    );
    assert_eq!(
        fakes.stream_ledger.operations(),
        ["open"],
        "the worker ran post-confirmation and joined in finish()"
    );
}

#[test]
fn cancelled_gate_never_spawns_the_chat_worker() {
    let gate_transport = Arc::new(ScriptedTransport::default());
    let cancellation = WaitCanceller::new();
    cancellation.request_cancel();
    let fakes = ChatFakes::new();
    let channel = chat_channel();
    let mut plan = gate_plan(gate_transport, cancellation);
    plan.chat_worker = Some(worker_plan(&fakes, &channel));

    let finish = spawn_startup_gate(plan).finish();

    assert!(matches!(finish, GateFinish::Cancelled));
    assert!(
        fakes.stream_ledger.operations().is_empty(),
        "no worker is spawned outside the Confirmed arm"
    );
    assert!(
        fakes.operations().is_empty(),
        "no lookup happens outside the Confirmed arm"
    );
}
