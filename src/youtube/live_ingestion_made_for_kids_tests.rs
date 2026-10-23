//! Made-for-kids plumbing pins: the optional per-stream declaration flows
//! verbatim from the validated config into the captured O2 command, both
//! for true and false. All against the shared scripted transport fake; no
//! network, no Google.

use crate::config::valid_config;
use crate::youtube::live_test_fixtures::*;

use super::live_ingestion_tests::{queue_preparation, FIXED_NOW_EPOCH_SECS};

use super::*;

#[test]
fn declared_true_flows_from_config_to_the_broadcast_command() {
    assert_eq!(captured_declaration_for(true), Some(true));
}

#[test]
fn declared_false_flows_from_config_to_the_broadcast_command() {
    assert_eq!(captured_declaration_for(false), Some(false));
}

fn captured_declaration_for(declared: bool) -> Option<bool> {
    let mut config = valid_config();
    config.youtube.broadcast.made_for_kids = Some(declared);
    let transport = ScriptedLiveApiTransport::default();
    queue_preparation(&transport);
    LiveBroadcastPreparer::from_config(&config, FIXED_NOW_EPOCH_SECS)
        .prepare(&transport, &WaitCanceller::new())
        .expect("the happy script prepares");
    let captured_broadcast = transport.captured_create_broadcast.borrow();
    captured_broadcast
        .as_ref()
        .expect("the O2 command was captured")
        .made_for_kids
}
