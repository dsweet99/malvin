use super::backend_error_stop::{
    LOCAL_BACKEND_GPU_MEMORY_HINT, backend_error_stop, error_suggests_local_gpu_memory,
};
use super::backend_error_tracker::{
    BackendErrorTracker, LOCAL_MAX_BACKEND_ERRORS, format_local_backend_retry_cap_message,
};
use super::test_support::test_io;

fn client_for(model: &str) -> super::SdkClient {
    let model = crate::model_id::parse_model_id(model).expect("model");
    super::SdkClient::with_max_retries(model, test_io(), 3)
}

#[test]
fn tracker_counts_differing_errors_toward_local_cap() {
    let mut tracker = BackendErrorTracker::with_limit(
        super::backend_error_tracker::AcpRetryCount::at_least_one(3).as_consecutive_limit(),
    );
    for i in 1..LOCAL_MAX_BACKEND_ERRORS {
        assert!(!tracker.record_error(&format!("error {i}")));
        assert_eq!(tracker.local_retry_cap_errors(), None);
    }
    assert!(!tracker.record_error("error last"));
    assert_eq!(
        tracker.local_retry_cap_errors(),
        Some(LOCAL_MAX_BACKEND_ERRORS)
    );
    tracker.record_success();
    assert_eq!(tracker.local_retry_cap_errors(), None);
}

#[test]
fn local_client_stops_on_differing_errors_at_cap_with_hint() {
    let mut client = client_for("rpi:ollama/tiny");
    for i in 1..LOCAL_MAX_BACKEND_ERRORS {
        assert!(backend_error_stop(&mut client, &format!("Compute error {i}")).is_none());
    }
    let stop = backend_error_stop(&mut client, "Compute error last").expect("local cap stops");
    assert_eq!(stop.fault, crate::acp::AgentFault::BackendRetryLimit);
    assert!(!stop.requires_coder_session_teardown());
    assert!(stop.message.contains("without a successful turn"), "{stop}");
    assert!(
        stop.message.contains(LOCAL_BACKEND_GPU_MEMORY_HINT),
        "{stop}"
    );
}

#[test]
fn cloud_client_has_no_local_cap_or_hint() {
    let mut client = client_for("rpi:openai/gpt-4o");
    for i in 1..=LOCAL_MAX_BACKEND_ERRORS * 2 {
        assert!(backend_error_stop(&mut client, &format!("Compute error {i}")).is_none());
    }
    for _ in 1..3 {
        assert!(backend_error_stop(&mut client, "Compute error").is_none());
    }
    let stop = backend_error_stop(&mut client, "Compute error").expect("same-error stop");
    assert!(stop.message.contains("repeated 3 times in a row"), "{stop}");
    assert!(
        !stop.message.contains(LOCAL_BACKEND_GPU_MEMORY_HINT),
        "{stop}"
    );
}

#[test]
fn local_same_error_stop_carries_hint_only_for_matching_errors() {
    let mut client = client_for("rpi:ollama/tiny");
    for _ in 1..3 {
        assert!(backend_error_stop(&mut client, "HTTP 500").is_none());
    }
    let stop = backend_error_stop(&mut client, "HTTP 500").expect("same-error stop");
    assert!(
        !stop.message.contains(LOCAL_BACKEND_GPU_MEMORY_HINT),
        "{stop}"
    );
}

#[test]
fn gpu_memory_needles_match_reported_errors() {
    assert!(error_suggests_local_gpu_memory("Compute error"));
    assert!(error_suggests_local_gpu_memory(
        "OpenAI API protocol error (HTTP 200): unexpected Content-Type application/x-ndjson (expected text/event-stream)"
    ));
    assert!(!error_suggests_local_gpu_memory(
        "HTTP 500 internal server error"
    ));
}

#[test]
fn local_retry_cap_message_names_limits() {
    let msg = format_local_backend_retry_cap_message("rpi", "boom", 10);
    assert!(msg.contains("failed 10 times"), "{msg}");
    assert!(msg.contains("300 s"), "{msg}");
    assert!(msg.ends_with("Last error:\nboom"), "{msg}");
}
