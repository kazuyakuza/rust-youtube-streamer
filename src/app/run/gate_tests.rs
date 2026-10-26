//! Confirmation-gate tests (plan §8 TB; zero-sleep outcomes ONLY — the
//! first scripted attempt resolves every case, so no unit test ever
//! sleeps a poll interval): Confirmed on `active`, Cancelled on a
//! pre-requested stop with zero lookups (O4-only proof), and a fatal
//! stream status recording `Fatal` AND flipping the shared stop flag.

use std::sync::Arc;

use crate::app::pipeline::StopFlag;
use crate::app::run::broadcast::public_watch_url;
use crate::app::run::runtime_test_fixtures::{
    inserted_stream, stream_with_status, ScriptedTransport, BROADCAST_ID_FIXTURE,
};
use crate::youtube::{LiveApiError, WaitCanceller};

use super::{observed_status_label, spawn_startup_gate, GateFinish, GatePlan};

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
fn confirmed_active_finishes_confirmed() {
    let transport = Arc::new(ScriptedTransport::default());
    transport.script_gate_lookup(Ok(Some(stream_with_status("active"))));
    let gate = spawn_startup_gate(gate_plan(Arc::clone(&transport), WaitCanceller::new()));

    let finish = gate.finish();

    assert!(matches!(finish, GateFinish::Confirmed));
    assert_eq!(
        transport.operations(),
        ["lookup_stream"],
        "the confirmation gate is O4-only (no broadcast lookup)"
    );
}

#[test]
fn pre_requested_cancel_finishes_cancelled_with_zero_lookups() {
    let transport = Arc::new(ScriptedTransport::default());
    let cancellation = WaitCanceller::new();
    cancellation.request_cancel();
    let gate = spawn_startup_gate(gate_plan(Arc::clone(&transport), cancellation));

    let finish = gate.finish();

    assert!(matches!(finish, GateFinish::Cancelled));
    assert!(transport.operations().is_empty());
}

#[test]
fn fatal_stream_status_requests_the_stop_and_is_recorded() {
    let transport = Arc::new(ScriptedTransport::default());
    transport.script_gate_lookup(Ok(Some(stream_with_status("error"))));
    let stop_flag = StopFlag::new();
    let mut plan = gate_plan(Arc::clone(&transport), WaitCanceller::new());
    plan.stop_flag = stop_flag.clone();
    let gate = spawn_startup_gate(plan);

    let finish = gate.finish();

    assert!(matches!(
        finish,
        GateFinish::Fatal(LiveApiError::BroadcastState { .. })
    ));
    assert!(
        stop_flag.is_requested(),
        "a gate fatal must flip the frame-loop stop flag"
    );
}

#[test]
fn confirmation_timeout_performs_exactly_one_final_status_lookup() {
    let transport = Arc::new(ScriptedTransport::default());
    transport.script_gate_lookup(Ok(Some(stream_with_status("ready"))));
    transport.script_gate_lookup(Ok(Some(stream_with_status("ready"))));
    let stop_flag = StopFlag::new();
    let mut plan = gate_plan(Arc::clone(&transport), WaitCanceller::new());
    plan.stop_flag = stop_flag.clone();
    let gate = spawn_startup_gate(plan);

    let finish = gate.finish();

    assert!(matches!(
        finish,
        GateFinish::Fatal(LiveApiError::ReadinessTimeout { .. })
    ));
    assert_eq!(
        transport.operations(),
        ["lookup_stream", "lookup_stream"],
        "the timeout path performs exactly ONE extra tolerated status lookup"
    );
    assert!(stop_flag.is_requested());
}

#[test]
fn confirmation_timeout_tolerates_a_failing_final_status_lookup() {
    let transport = Arc::new(ScriptedTransport::default());
    transport.script_gate_lookup(Ok(Some(stream_with_status("ready"))));
    transport.script_gate_lookup(Err(LiveApiError::Network {
        reason: "final-lookup-fixture".to_string(),
    }));
    let gate = spawn_startup_gate(gate_plan(Arc::clone(&transport), WaitCanceller::new()));

    let finish = gate.finish();

    assert!(matches!(
        finish,
        GateFinish::Fatal(LiveApiError::ReadinessTimeout { .. })
    ));
    assert_eq!(
        transport.operations(),
        ["lookup_stream", "lookup_stream"],
        "a failing final lookup is tolerated and never becomes the reported variant"
    );
}

#[test]
fn non_timeout_gate_fatal_performs_no_extra_status_lookup() {
    let transport = Arc::new(ScriptedTransport::default());
    transport.script_gate_lookup(Ok(Some(stream_with_status("error"))));
    let gate = spawn_startup_gate(gate_plan(Arc::clone(&transport), WaitCanceller::new()));

    let finish = gate.finish();

    assert!(matches!(
        finish,
        GateFinish::Fatal(LiveApiError::BroadcastState { .. })
    ));
    assert_eq!(
        transport.operations(),
        ["lookup_stream"],
        "no extra tolerated lookup outside the timeout shape"
    );
}

#[test]
fn observed_status_label_reads_the_raw_stream_status_label() {
    assert_eq!(
        observed_status_label(&stream_with_status("ready")),
        Some("ready")
    );
    assert_eq!(observed_status_label(&inserted_stream()), None);
}
