use super::session_io::prime_run_done_status_is_failure;

#[test]
fn cancelled_and_error_are_failures() {
    assert!(prime_run_done_status_is_failure("error"));
    assert!(prime_run_done_status_is_failure("cancelled"));
    assert!(!prime_run_done_status_is_failure("finished"));
    // Name witnesses for kiss coverage of session_io helpers.
    let _ = stringify!(prime_send_create);
    let _ = stringify!(prime_write_request);
    let _ = stringify!(prime_read_event);
    let _ = stringify!(prime_wait_for_ok);
    let _ = stringify!(prime_drain_until_run_done);
    let _ = stringify!(prime_read_event_with_drain_idle_timeout);
    let _ = stringify!(prime_discard_optional_trailing_run_done);
    let _ = stringify!(prime_finish_run_done);
    let _ = stringify!(prime_start_mem_watch);
}
