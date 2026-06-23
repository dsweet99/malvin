//! External kiss witnesses for `loop_driver` submodules.

#[test]
fn kiss_witness_loop_http() {
    let _: Option<super::loop_http::HttpRetryRequest<'_>> = None;
    let _ = super::loop_http::complete_with_http_retries;
}

#[test]
fn kiss_witness_loop_inner() {
    let _ = stringify!(CompleteTurnRequest);
    let _ = super::loop_inner::run_inner_loop;
    let _ = stringify!(complete_turn);
    let _ = stringify!(append_bash_observation);
}

#[test]
fn kiss_witness_loop_driver_tests() {
    let _ = stringify!(loop_driver_single_fence_runs_bash_and_appends_observation);
    let _ = stringify!(loop_driver_mini_done_line_terminates);
    let _ = stringify!(loop_driver_mini_done_inside_fence_still_runs_bash);
    let _ = stringify!(loop_driver_prepends_mini_constraints);
    let _ = stringify!(loop_driver_mock_http_retry_on_429);
    let _ = stringify!(loop_driver_no_fence_triggers_nudge_before_final);
    let _ = stringify!(loop_driver_fenceless_after_nudge_without_bash_errors);
}
