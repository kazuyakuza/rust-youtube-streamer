//! Provider-stamp stage-row pins: the dequeued, rendered, and written rows
//! carry the provider `publishedAt` epoch when it reached the bounded
//! in-flight staging, and stay null otherwise. Offline, fake clocks, no
//! sleeps.

use crate::chat::ChatMessage;

use super::diagnostics_test_fixtures::{
    collector_with_output_file, drive_stage_cycle, PROVIDER_EPOCH_FIXTURE,
};

fn fixture_message(id: &str) -> ChatMessage {
    ChatMessage::new("author-fixture", "text-fixture").with_message_id(id)
}

#[test]
fn timestamp_bearing_message_populates_all_three_stage_rows() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let harness = collector_with_output_file(&path);
    let message = fixture_message("m-9").with_timestamp(Some(PROVIDER_EPOCH_FIXTURE));

    drive_stage_cycle(&harness, &message);

    let content = std::fs::read_to_string(&path).expect("stage rows must be written");
    let stamped_row = format!("\"provider_epoch_seconds\":{PROVIDER_EPOCH_FIXTURE}");
    assert_eq!(content.matches(stamped_row.as_str()).count(), 3);
}

#[test]
fn timestamp_absent_rows_stay_null_on_all_three_stages() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let harness = collector_with_output_file(&path);
    let message = fixture_message("m-absent");

    drive_stage_cycle(&harness, &message);

    let content = std::fs::read_to_string(&path).expect("stage rows must be written");
    assert_eq!(
        content.matches("\"provider_epoch_seconds\":null").count(),
        3
    );
}
