//! Made-for-kids O2 body pins: the optional `status.selfDeclaredMadeForKids`
//! field is omitted when absent and serialized under the exact documented
//! name when declared. Fixtures are fake-shaped; no test contacts Google.

use serde_json::json;

use super::*;

#[test]
fn made_for_kids_none_omits_self_declared_field_from_o2_body() {
    let body = BroadcastRequestBody::from_command(&broadcast_command(None));
    assert_eq!(
        serde_json::to_value(&body).expect("body serializes"),
        json!({
            "snippet": {
                "title": "Fixture broadcast",
                "description": "Fixture description",
                "scheduledStartTime": "2026-10-14T12:01:00Z"
            },
            "contentDetails": {
                "enableAutoStart": true,
                "enableAutoStop": true,
                "monitorStream": {"enableMonitorStream": false}
            },
            "status": {"privacyStatus": "unlisted"}
        })
    );
}

#[test]
fn made_for_kids_true_appears_in_o2_status_object() {
    let body = BroadcastRequestBody::from_command(&broadcast_command(Some(true)));
    assert_eq!(
        serde_json::to_value(&body).expect("body serializes"),
        json!({
            "snippet": {
                "title": "Fixture broadcast",
                "description": "Fixture description",
                "scheduledStartTime": "2026-10-14T12:01:00Z"
            },
            "contentDetails": {
                "enableAutoStart": true,
                "enableAutoStop": true,
                "monitorStream": {"enableMonitorStream": false}
            },
            "status": {
                "privacyStatus": "unlisted",
                "selfDeclaredMadeForKids": true
            }
        })
    );
}

#[test]
fn made_for_kids_false_appears_in_o2_status_object() {
    let body = BroadcastRequestBody::from_command(&broadcast_command(Some(false)));
    assert_eq!(
        serde_json::to_value(&body).expect("body serializes"),
        json!({
            "snippet": {
                "title": "Fixture broadcast",
                "description": "Fixture description",
                "scheduledStartTime": "2026-10-14T12:01:00Z"
            },
            "contentDetails": {
                "enableAutoStart": true,
                "enableAutoStop": true,
                "monitorStream": {"enableMonitorStream": false}
            },
            "status": {
                "privacyStatus": "unlisted",
                "selfDeclaredMadeForKids": false
            }
        })
    );
}

fn broadcast_command(made_for_kids: Option<bool>) -> CreateBroadcastCommand {
    CreateBroadcastCommand {
        title: "Fixture broadcast".to_string(),
        description: "Fixture description".to_string(),
        privacy_status: "unlisted".to_string(),
        scheduled_start_time: "2026-10-14T12:01:00Z".to_string(),
        made_for_kids,
    }
}
