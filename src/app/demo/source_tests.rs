//! Clock-free source tests: exact call sequences against the real demo
//! timeline. One call equals one frame, so assertions drive frame indices
//! directly with no sleeps and no wall clock.

use super::DemoMessageSource;
use crate::app::demo::schedule::{
    scheduled_messages_for_tests, DemoTimeline, ScheduledMessage, LEAD_IN_FRAMES,
};
use crate::app::pipeline::{MessageSource, StopFlag};
use crate::chat::ChatMessage;

fn source_with_stop_flag() -> (DemoMessageSource, StopFlag) {
    let stop_flag = StopFlag::new();
    let timeline = DemoTimeline::raw(scheduled_messages_for_tests());
    (
        DemoMessageSource::new(timeline, stop_flag.clone()),
        stop_flag,
    )
}

fn drain_frames(source: &mut DemoMessageSource, frames: u64) {
    for _frame_index in 0..frames {
        source.next_message().expect("demo source never fails");
    }
}

fn next_display(source: &mut DemoMessageSource) -> Option<String> {
    source
        .next_message()
        .expect("demo source never fails")
        .map(|message| format!("{message}"))
}

#[test]
fn no_message_is_due_during_the_black_lead_in() {
    let (mut source, _stop_flag) = source_with_stop_flag();
    for _frame_index in 0..LEAD_IN_FRAMES {
        assert_eq!(next_display(&mut source), None);
    }
}

#[test]
fn first_message_is_due_at_its_slot() {
    let (mut source, _stop_flag) = source_with_stop_flag();
    drain_frames(&mut source, LEAD_IN_FRAMES);
    assert_eq!(
        next_display(&mut source).as_deref(),
        Some("alice: demo message 01")
    );
    assert_eq!(next_display(&mut source), None);
}

#[test]
fn at_most_one_message_is_delivered_per_call() {
    let stop_flag = StopFlag::new();
    let overdue = vec![
        ScheduledMessage {
            due_frame_index: 5,
            message: ChatMessage::new("alice", "first overdue"),
        },
        ScheduledMessage {
            due_frame_index: 5,
            message: ChatMessage::new("bob", "second overdue"),
        },
        ScheduledMessage {
            due_frame_index: 5,
            message: ChatMessage::new("charlie", "third overdue"),
        },
    ];
    let mut source = DemoMessageSource::new(DemoTimeline::raw(overdue), stop_flag);
    for _frame_index in 0..5 {
        assert_eq!(next_display(&mut source), None);
    }
    assert_eq!(
        next_display(&mut source).as_deref(),
        Some("alice: first overdue")
    );
    assert_eq!(
        next_display(&mut source).as_deref(),
        Some("bob: second overdue")
    );
    assert_eq!(
        next_display(&mut source).as_deref(),
        Some("charlie: third overdue")
    );
    assert_eq!(next_display(&mut source), None);
}

#[test]
fn timeline_delivers_all_messages_in_order_without_any_clock() {
    let (mut source, _stop_flag) = source_with_stop_flag();
    let expected: Vec<String> = scheduled_messages_for_tests()
        .iter()
        .map(|entry| format!("{}", entry.message))
        .collect();
    let mut delivered = Vec::new();
    for _frame_index in 0..=435 {
        if let Some(display) = next_display(&mut source) {
            delivered.push(display);
        }
    }
    assert_eq!(delivered, expected);
}

#[test]
fn timeline_requests_the_stop_at_its_last_frame() {
    let (mut source, stop_flag) = source_with_stop_flag();
    let timeline = DemoTimeline::raw(scheduled_messages_for_tests());
    assert!(!stop_flag.is_requested());
    drain_frames(&mut source, timeline.final_frame_index());
    assert!(!stop_flag.is_requested());
    assert_eq!(next_display(&mut source), None);
    assert!(stop_flag.is_requested());
}

#[test]
fn timeline_ends_after_its_final_delivered_message() {
    let (mut source, stop_flag) = source_with_stop_flag();
    let timeline = DemoTimeline::raw(scheduled_messages_for_tests());
    let mut delivered = 0;
    for _frame_index in 0..timeline.final_frame_index() {
        if next_display(&mut source).is_some() {
            delivered += 1;
        }
        assert!(!stop_flag.is_requested());
    }
    assert_eq!(delivered, scheduled_messages_for_tests().len());
}
