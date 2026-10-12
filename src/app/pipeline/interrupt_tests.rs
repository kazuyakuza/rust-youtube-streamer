use crate::app::pipeline::interrupt::StopFlag;

#[test]
fn stop_flag_starts_clear_and_flips_on_request() {
    let flag = StopFlag::new();
    assert!(!flag.is_requested());
    flag.request_stop();
    assert!(flag.is_requested());
}

#[test]
fn stop_flag_clone_shares_one_request_across_holders() {
    let original = StopFlag::new();
    let clone = original.clone();
    clone.request_stop();
    assert!(original.is_requested());
}
