use super::backend_error_tracker::{
    BackendErrorTracker, MAX_CONSECUTIVE_SAME_BACKEND_ERRORS,
    format_backend_consecutive_error_message,
};

#[test]
fn tracker_starts_with_zero_count_and_no_error() {
    let tracker = BackendErrorTracker::default();
    assert_eq!(tracker.consecutive_count(), 0);
    assert_eq!(tracker.last_error(), None);
    assert!(!tracker.should_stop_and_exit());
}

#[test]
fn tracker_increments_for_same_error_and_triggers_at_three() {
    let mut tracker = BackendErrorTracker::new();
    assert!(!tracker.record_error("timeout"));
    assert_eq!(tracker.consecutive_count(), 1);
    assert_eq!(tracker.last_error(), Some("timeout"));

    assert!(!tracker.record_error("timeout"));
    assert_eq!(tracker.consecutive_count(), 2);

    let stop = tracker.record_error("timeout");
    assert!(stop);
    assert_eq!(tracker.consecutive_count(), 3);
    assert!(tracker.should_stop_and_exit());
    assert_eq!(MAX_CONSECUTIVE_SAME_BACKEND_ERRORS, 3);
}

#[test]
fn tracker_resets_counter_on_different_error() {
    let mut tracker = BackendErrorTracker::new();
    assert!(!tracker.record_error("error_a"));
    assert_eq!(tracker.consecutive_count(), 1);
    assert!(!tracker.record_error("error_a"));
    assert_eq!(tracker.consecutive_count(), 2);

    assert!(!tracker.record_error("error_b"));
    assert_eq!(tracker.consecutive_count(), 1);
    assert_eq!(tracker.last_error(), Some("error_b"));
    assert!(!tracker.should_stop_and_exit());

    assert!(!tracker.record_error("error_b"));
    assert_eq!(tracker.consecutive_count(), 2);
    assert!(tracker.record_error("error_b"));
    assert_eq!(tracker.consecutive_count(), 3);
    assert!(tracker.should_stop_and_exit());
}

#[test]
fn tracker_resets_counter_on_success() {
    let mut tracker = BackendErrorTracker::new();
    assert!(!tracker.record_error("failure"));
    assert!(!tracker.record_error("failure"));
    assert_eq!(tracker.consecutive_count(), 2);

    tracker.record_success();
    assert_eq!(tracker.consecutive_count(), 0);
    assert_eq!(tracker.last_error(), None);
    assert!(!tracker.should_stop_and_exit());

    assert!(!tracker.record_error("failure"));
    assert_eq!(tracker.consecutive_count(), 1);
}

#[test]
fn tracker_treats_trimmed_whitespace_variation_as_same_error() {
    let mut tracker = BackendErrorTracker::new();
    assert!(!tracker.record_error("internal error\n"));
    assert_eq!(tracker.consecutive_count(), 1);
    assert!(!tracker.record_error("  internal error  "));
    assert_eq!(tracker.consecutive_count(), 2);
    assert!(tracker.record_error("internal error"));
    assert_eq!(tracker.consecutive_count(), 3);
    assert!(tracker.should_stop_and_exit());
}

#[test]
fn format_backend_consecutive_error_message_contains_details() {
    let msg = format_backend_consecutive_error_message("cursor", "connection lost");
    assert!(msg.contains("cursor backend error repeated 3 times"));
    assert!(msg.contains("stopping and exiting"));
    assert!(msg.contains("connection lost"));
}

#[tokio::test]
async fn client_error_tracking_stops_and_exits_on_three_same_errors() {
    let model = crate::model_id::parse_model_id("cursor:auto").expect("model");
    let mut client = crate::agent_backend::new_cursor(model, crate::agent_backend::test_support::test_io());
    client.max_acp_retries = 5;

    assert!(!client.record_backend_error("same error"));
    assert!(!client.record_backend_error("same error"));
    assert_eq!(client.backend_error_tracker().consecutive_count(), 2);
    assert!(client.record_backend_error("same error"));
    assert_eq!(client.backend_error_tracker().consecutive_count(), 3);
    assert!(client.backend_error_tracker().should_stop_and_exit());

    client.record_backend_success();
    assert_eq!(client.backend_error_tracker().consecutive_count(), 0);
    assert!(!client.backend_error_tracker().should_stop_and_exit());
}
