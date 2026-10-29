//! Golden-byte pins for the hand-authored `stream_list.proto` wire types
//! (plan §C.1/appendix; record §A.9): every field number is locked by
//! encoding a populated struct and matching the pre-computed sequence, and
//! unknown fields / oneof arms / enum values are pinned as tolerant-and-
//! skipped. Offline only — no session is ever dialed here.

use prost::Message;

use super::*;

const ANCHOR_A: &[u8] = &[
    0x0A, 0x0F, 0x63, 0x68, 0x61, 0x74, 0x2D, 0x69, 0x64, 0x2D, 0x66, 0x69, 0x78, 0x74, 0x75, 0x72,
    0x65, 0xA2, 0x06, 0x02, 0x69, 0x64, 0xA2, 0x06, 0x07, 0x73, 0x6E, 0x69, 0x70, 0x70, 0x65, 0x74,
    0xA2, 0x06, 0x0D, 0x61, 0x75, 0x74, 0x68, 0x6F, 0x72, 0x44, 0x65, 0x74, 0x61, 0x69, 0x6C, 0x73,
];

const ANCHOR_B: &[u8] = &[
    0x0A, 0x0F, 0x63, 0x68, 0x61, 0x74, 0x2D, 0x69, 0x64, 0x2D, 0x66, 0x69, 0x78, 0x74, 0x75, 0x72,
    0x65, 0x9A, 0x06, 0x0D, 0x74, 0x6F, 0x6B, 0x65, 0x6E, 0x2D, 0x66, 0x69, 0x78, 0x74, 0x75, 0x72,
    0x65, 0xA2, 0x06, 0x02, 0x69, 0x64, 0xA2, 0x06, 0x07, 0x73, 0x6E, 0x69, 0x70, 0x70, 0x65, 0x74,
    0xA2, 0x06, 0x0D, 0x61, 0x75, 0x74, 0x68, 0x6F, 0x72, 0x44, 0x65, 0x74, 0x61, 0x69, 0x6C, 0x73,
];

const ANCHOR_C: &[u8] = &[0x28, 0x01, 0xBA, 0x06, 0x03, 0x42, 0x6F, 0x62];
const ANCHOR_D: &[u8] = &[0x0A, 0x05, 0x68, 0x65, 0x6C, 0x6C, 0x6F];
const ANCHOR_E: &[u8] = &[
    0x08, 0x01, 0x9A, 0x01, 0x07, 0x0A, 0x05, 0x68, 0x65, 0x6C, 0x6C, 0x6F,
];
const ANCHOR_F: &[u8] = &[
    0x12, 0x0C, 0x08, 0x01, 0x9A, 0x01, 0x07, 0x0A, 0x05, 0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x1A, 0x06,
    0xBA, 0x06, 0x03, 0x42, 0x6F, 0x62, 0xAA, 0x06, 0x05, 0x6D, 0x73, 0x67, 0x2D, 0x31,
];
const ANCHOR_G: &[u8] = &[
    0xFA, 0x3E, 0x1E, 0x12, 0x0C, 0x08, 0x01, 0x9A, 0x01, 0x07, 0x0A, 0x05, 0x68, 0x65, 0x6C, 0x6C,
    0x6F, 0x1A, 0x06, 0xBA, 0x06, 0x03, 0x42, 0x6F, 0x62, 0xAA, 0x06, 0x05, 0x6D, 0x73, 0x67, 0x2D,
    0x31, 0xD2, 0x8F, 0x31, 0x0D, 0x74, 0x6F, 0x6B, 0x65, 0x6E, 0x2D, 0x66, 0x69, 0x78, 0x74, 0x75,
    0x72, 0x65,
];
const ANCHOR_H: &[u8] = &[0x08, 0x01, 0x10, 0x02];
const TOLERANCE_T1: &[u8] = &[0xDA, 0x22, 0x01, 0x01];
const TOLERANCE_T2: &[u8] = &[0xB2, 0x01, 0x03, 0x0A, 0x01, 0x01];
const TOLERANCE_U1: &[u8] = &[0xE3, 0x01];

fn request(page_token: Option<&str>) -> ChatStreamRequest {
    ChatStreamRequest {
        live_chat_id: "chat-id-fixture".to_string(),
        page_token: page_token.map(str::to_string),
    }
}

fn text_details() -> LiveChatTextMessageDetails {
    LiveChatTextMessageDetails {
        message_text: Some("hello".to_string()),
    }
}

fn text_snippet() -> LiveChatMessageSnippet {
    LiveChatMessageSnippet {
        message_type: Some(SnippetType::TextMessageEvent as i32),
        displayed_content: Some(DisplayedContent::TextMessage(text_details())),
        ..Default::default()
    }
}

fn display_name_author() -> LiveChatMessageAuthorDetails {
    LiveChatMessageAuthorDetails {
        display_name: Some("Bob".to_string()),
        ..Default::default()
    }
}

fn text_message() -> LiveChatMessage {
    LiveChatMessage {
        id: Some("msg-1".to_string()),
        snippet: Some(text_snippet()),
        author_details: Some(display_name_author()),
        ..Default::default()
    }
}

fn pinned_response() -> LiveChatMessageListResponse {
    LiveChatMessageListResponse {
        next_page_token: Some("token-fixture".to_string()),
        items: vec![text_message()],
        ..Default::default()
    }
}

#[test]
fn stream_list_request_encodes_the_documented_wire_form() {
    let encoded = stream_list_request(&request(None)).encode_to_vec();
    assert_eq!(encoded, ANCHOR_A);
}

#[test]
fn stream_list_request_with_resume_token_appends_field_99() {
    let encoded = stream_list_request(&request(Some("token-fixture"))).encode_to_vec();
    assert_eq!(encoded, ANCHOR_B);
}

#[test]
fn unset_request_encodes_zero_bytes() {
    assert!(LiveChatMessageListRequest::default()
        .encode_to_vec()
        .is_empty());
}

#[test]
fn text_message_details_encode_pin() {
    assert_eq!(text_details().encode_to_vec(), ANCHOR_D);
}

#[test]
fn snippet_text_message_oneof_encode_pin() {
    assert_eq!(text_snippet().encode_to_vec(), ANCHOR_E);
}

#[test]
fn author_details_display_name_and_owner_flag_encode_pin() {
    let author = LiveChatMessageAuthorDetails {
        display_name: Some("Bob".to_string()),
        is_chat_owner: Some(true),
        ..Default::default()
    };
    assert_eq!(author.encode_to_vec(), ANCHOR_C);
}

#[test]
fn live_chat_message_encode_pin() {
    assert_eq!(text_message().encode_to_vec(), ANCHOR_F);
}

#[test]
fn response_encode_pin() {
    assert_eq!(pinned_response().encode_to_vec(), ANCHOR_G);
}

#[test]
fn page_info_encode_pin() {
    let page_info = PageInfo {
        total_results: Some(1),
        results_per_page: Some(2),
    };
    assert_eq!(page_info.encode_to_vec(), ANCHOR_H);
}

#[test]
fn response_decodes_the_pinned_wire_bytes() {
    let decoded =
        LiveChatMessageListResponse::decode(ANCHOR_G).expect("the pinned response decodes");
    assert_eq!(decoded, pinned_response());
}

#[test]
fn unknown_response_fields_are_tolerated_and_skipped() {
    let mut bytes = ANCHOR_G.to_vec();
    bytes.extend_from_slice(TOLERANCE_T1);
    let decoded =
        LiveChatMessageListResponse::decode(&bytes[..]).expect("unknown fields are skipped");
    assert_eq!(decoded, pinned_response());
}

#[test]
fn unknown_snippet_oneof_arm_is_tolerated_and_skipped() {
    let mut bytes = vec![0x08, 0x02];
    bytes.extend_from_slice(TOLERANCE_T2);
    let decoded =
        LiveChatMessageSnippet::decode(&bytes[..]).expect("the unknown oneof arm is skipped");
    assert!(decoded.displayed_content.is_none());
    assert_eq!(decoded.message_type, Some(SnippetType::Tombstone as i32));
}

#[test]
fn unknown_snippet_type_value_decodes_without_failure() {
    let mut bytes = vec![0x08];
    bytes.extend_from_slice(TOLERANCE_U1);
    let decoded =
        LiveChatMessageSnippet::decode(&bytes[..]).expect("the unknown enum value is skipped");
    assert_ne!(
        decoded.message_type,
        Some(SnippetType::TextMessageEvent as i32)
    );
}

#[test]
fn full_response_round_trips() {
    let decoded =
        LiveChatMessageListResponse::decode(ANCHOR_G).expect("the pinned response decodes");
    assert_eq!(decoded.encode_to_vec(), ANCHOR_G);
}
