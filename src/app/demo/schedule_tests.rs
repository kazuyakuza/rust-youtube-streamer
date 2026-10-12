//! Pure schedule tests: ordering, pinned long-line slot, one-per-frame
//! budget, the planning formula (495/496), delivery-vs-final slack, and the
//! visible-capacity comparison. No clock, sleep, or pipeline anywhere.

use super::*;
use crate::config::valid_config;

#[test]
fn fixed_message_set_is_completely_ordered() {
    let deliveries = scheduled_messages_for_tests();
    assert_eq!(deliveries.len(), DEMO_MESSAGE_TOTAL);
    let due_frames: Vec<u64> = deliveries
        .iter()
        .map(|entry| entry.due_frame_index)
        .collect();
    assert!(due_frames.windows(2).all(|pair| pair[0] < pair[1]));
    for (index, entry) in deliveries.iter().enumerate() {
        let number = index + 1;
        assert_eq!(
            entry.message.author(),
            MESSAGE_AUTHORS[index % MESSAGE_AUTHORS.len()]
        );
        if is_long_line_slot(number) {
            assert_eq!(
                entry.message.text(),
                LONG_LINE_PIECE.repeat(LONG_LINE_PIECES)
            );
        } else {
            assert_eq!(entry.message.text(), format!("{TEXT_PREFIX} {number:02}"));
        }
    }
}

#[test]
fn long_line_message_carries_the_pinned_text_and_hidden_metadata() {
    let deliveries = scheduled_messages_for_tests();
    let entry = &deliveries[LONG_LINE_MESSAGE_INDEX - 1];
    assert_eq!(entry.message.author(), "charlie");
    let long_text = LONG_LINE_PIECE.repeat(LONG_LINE_PIECES);
    assert_eq!(entry.message.text(), long_text);
    assert_eq!(entry.message.text().chars().count(), 250);
    let display = format!("{}", entry.message);
    assert_eq!(display, format!("charlie: {long_text}"));
    assert!(!display.contains("demo-15"));
}

#[test]
fn due_frames_respect_the_one_per_frame_budget() {
    let deliveries = scheduled_messages_for_tests();
    for pair in deliveries.windows(2) {
        assert_eq!(
            pair[1].due_frame_index - pair[0].due_frame_index,
            MESSAGE_GAP_FRAMES
        );
    }
    let first = deliveries.first().expect("non-empty schedule");
    assert_eq!(first.due_frame_index, LEAD_IN_FRAMES);
    let last = deliveries.last().expect("non-empty schedule");
    let timeline = DemoTimeline::fixed_timeline();
    assert!(last.due_frame_index <= timeline.final_frame_index());
}

#[test]
fn final_frame_index_and_rendered_count_match_the_planning_formula() {
    let timeline = DemoTimeline::fixed_timeline();
    assert_eq!(timeline.final_frame_index(), 495);
    assert_eq!(timeline.rendered_frame_count(), 496);
    assert_eq!(
        timeline.final_frame_index(),
        LEAD_IN_FRAMES + ((DEMO_MESSAGE_TOTAL as u64) - 1) * MESSAGE_GAP_FRAMES + TAIL_FRAMES
    );
    assert_eq!(
        timeline.rendered_frame_count(),
        timeline.final_frame_index() + 1
    );
}

#[test]
fn every_due_frame_sits_at_or_before_the_final_frame() {
    let timeline = DemoTimeline::fixed_timeline();
    let deliveries = scheduled_messages_for_tests();
    let last_due = deliveries
        .last()
        .expect("non-empty schedule")
        .due_frame_index;
    assert!(last_due <= timeline.final_frame_index());
    assert_eq!(timeline.final_frame_index() - last_due, TAIL_FRAMES);
}

#[test]
fn message_set_exceeds_the_default_visible_capacity() {
    let config = valid_config();
    let capacity = crate::chat::derive_visible_capacity(&config.video, &config.renderer)
        .expect("valid geometry derives capacity");
    assert_eq!(capacity, 26);
    let timeline = DemoTimeline::fixed_timeline();
    assert!(timeline.total_deliveries() > capacity);
}
