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
fn tracker_restarts_streak_when_same_error_returns_after_reset_window() {
    use super::backend_error_tracker::SAME_BACKEND_ERROR_STREAK_RESET;
    use std::time::Duration;
    let mut tracker = BackendErrorTracker::new();
    let t0 = std::time::Instant::now();
    assert!(!tracker.record_error_at("rare", t0));
    assert!(!tracker.record_error_at("rare", t0 + Duration::from_secs(1)));
    assert_eq!(tracker.consecutive_count(), 2);
    let late = t0 + Duration::from_secs(1) + SAME_BACKEND_ERROR_STREAK_RESET + Duration::from_secs(1);
    assert!(!tracker.record_error_at("rare", late));
    assert_eq!(tracker.consecutive_count(), 1);
    assert_eq!(tracker.last_error(), Some("rare"));
    assert!(!tracker.record_error_at("rare", late + Duration::from_secs(2)));
    assert!(tracker.record_error_at("rare", late + Duration::from_secs(4)));
    assert!(tracker.should_stop_and_exit());
}

#[test]
fn tracker_keeps_streak_when_each_gap_is_within_reset_window() {
    use super::backend_error_tracker::SAME_BACKEND_ERROR_STREAK_RESET;
    let mut tracker = BackendErrorTracker::new();
    let t0 = std::time::Instant::now();
    assert!(!tracker.record_error_at("slow", t0));
    assert!(!tracker.record_error_at("slow", t0 + SAME_BACKEND_ERROR_STREAK_RESET));
    assert!(tracker.record_error_at("slow", t0 + SAME_BACKEND_ERROR_STREAK_RESET * 2));
    assert_eq!(tracker.consecutive_count(), 3);
}

#[test]
fn format_backend_consecutive_error_message_contains_details() {
    let msg = format_backend_consecutive_error_message("cursor", "connection lost", 3);
    assert!(msg.contains("cursor backend error repeated 3 times"));
    assert!(msg.contains("stopping and exiting"));
    assert!(msg.contains("connection lost"));
}

#[test]
fn config_zero_is_one_try_for_the_client_and_the_tracker() {
    use super::backend_error_tracker::AcpRetryCount;
    let retries = AcpRetryCount::at_least_one(0);
    assert_eq!(retries.get(), 1);
    assert_eq!(AcpRetryCount::at_least_one(5).get(), 5);
    let mut tracker = BackendErrorTracker::with_limit(retries.as_consecutive_limit());
    assert_eq!(tracker.max_consecutive(), 1);
    tracker.set_max_consecutive(retries.as_consecutive_limit());
    assert_eq!(tracker.max_consecutive(), 1);
    let model = crate::config::model_id::parse_model_id("cursor:auto").expect("model");
    let client = crate::backends::agent_backend::SdkClient::with_max_retries(
        model,
        crate::backends::agent_backend::test_support::test_io(),
        0,
    );
    assert_eq!(client.max_acp_retries.get(), 1);
    assert_eq!(client.backend_error_tracker().max_consecutive(), 1);
}

#[test]
fn tracker_respects_custom_max_consecutive() {
    let mut tracker = BackendErrorTracker::with_limit(
        super::backend_error_tracker::AcpRetryCount::at_least_one(2).as_consecutive_limit(),
    );
    assert_eq!(tracker.max_consecutive(), 2);
    assert!(!tracker.record_error("timeout"));
    assert_eq!(tracker.consecutive_count(), 1);
    assert!(tracker.record_error("timeout"));
    assert_eq!(tracker.consecutive_count(), 2);
    assert!(tracker.should_stop_and_exit());
}

#[tokio::test]
async fn client_error_tracking_stops_and_exits_on_consecutive_same_errors() {
    let model = crate::config::model_id::parse_model_id("cursor:auto").expect("model");
    let mut client =
        crate::backends::agent_backend::SdkClient::new(model, crate::backends::agent_backend::test_support::test_io());
    client.max_acp_retries = super::backend_error_tracker::AcpRetryCount::at_least_one(5);

    for i in 1..5 {
        assert!(!client.record_backend_error("same error"));
        assert_eq!(client.backend_error_tracker().consecutive_count(), i);
        assert!(!client.backend_error_tracker().should_stop_and_exit());
    }
    assert!(client.record_backend_error("same error"));
    assert_eq!(client.backend_error_tracker().consecutive_count(), 5);
    assert!(client.backend_error_tracker().should_stop_and_exit());

    client.record_backend_success();
    assert_eq!(client.backend_error_tracker().consecutive_count(), 0);
    assert!(!client.backend_error_tracker().should_stop_and_exit());
}

#[tokio::test]
async fn spawn_style_success_clears_consecutive_streak_before_next_error() {
    let model = crate::config::model_id::parse_model_id("cursor:auto").expect("model");
    let mut client =
        crate::backends::agent_backend::SdkClient::new(model, crate::backends::agent_backend::test_support::test_io());
    client.max_acp_retries = super::backend_error_tracker::AcpRetryCount::at_least_one(3);
    assert!(!client.record_backend_error("spawn failed"));
    assert!(!client.record_backend_error("spawn failed"));
    assert_eq!(client.backend_error_tracker().consecutive_count(), 2);
    client.record_backend_success();
    assert!(!client.record_backend_error("spawn failed"));
    assert_eq!(
        client.backend_error_tracker().consecutive_count(),
        1,
        "post-success failure must start a new streak at 1"
    );
    assert!(!client.backend_error_tracker().should_stop_and_exit());
}
