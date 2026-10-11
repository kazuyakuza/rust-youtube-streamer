//! `ChatMessage` display format, normalization, and metadata tests.

use super::*;

#[test]
fn display_format_is_username_then_message() {
    let message = ChatMessage::new("alice", "Hello!");
    assert_eq!(message.to_string(), "alice: Hello!");
}

#[test]
fn metadata_never_appears_in_display() {
    let message = ChatMessage::new("alice", "Hello!")
        .with_message_id("SECRET-ID-123")
        .with_timestamp(Some(1728381296));
    let display = message.to_string();
    assert_eq!(display, "alice: Hello!");
    assert!(!display.contains("SECRET-ID-123"));
    assert!(!display.contains("1728381296"));
}

#[test]
fn normalization_trims_surrounding_whitespace() {
    let message = ChatMessage::new("  alice  ", "  Hello!  ");
    assert_eq!(message.to_string(), "alice: Hello!");
}

#[test]
fn normalization_flattens_all_line_breaks() {
    let crlf = ChatMessage::new("alice", "a\r\nb").to_string();
    let lone_carriage_return = ChatMessage::new("alice", "a\rb").to_string();
    let lone_line_feed = ChatMessage::new("alice", "a\nb").to_string();
    assert_eq!(crlf, "alice: a b");
    assert_eq!(lone_carriage_return, "alice: a b");
    assert_eq!(lone_line_feed, "alice: a b");
    assert!(!crlf.contains("  "));
}

#[test]
fn metadata_accessors_return_input_verbatim() {
    let with_metadata = ChatMessage::new("alice", "Hello!")
        .with_message_id("msg-123")
        .with_timestamp(Some(1728381296));
    assert_eq!(with_metadata.message_id.as_deref(), Some("msg-123"));
    assert_eq!(with_metadata.timestamp(), Some(1728381296));
    let without_metadata = ChatMessage::new("bob", "Hi!");
    assert_eq!(without_metadata.message_id, None);
    assert_eq!(without_metadata.timestamp, None);
}
