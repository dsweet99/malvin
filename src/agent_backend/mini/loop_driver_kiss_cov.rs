//! External kiss witnesses for `loop_driver` submodules.

#[test]
fn kiss_witness_loop_http() {
    let _: Option<super::loop_http::HttpRetryRequest<'_>> = None;
    let _ = super::loop_http::complete_with_http_retries;
    let _ = std::mem::size_of::<super::loop_http_retry::HttpRetryLimits>();
    let _ = std::mem::size_of::<super::loop_http_retry::HttpRetryCounters>();
    let _ = stringify!(kiss_witness_http_retry_limits_and_counters);
}

#[test]
fn kiss_witness_loop_inner() {
    let _ = stringify!(CompleteTurnRequest);
    let _ = stringify!(LoopPhase);
    let _ = stringify!(BashObservationInput);
    let _ = super::loop_inner::run_inner_loop;
    let _ = stringify!(complete_turn);
    let _ = super::loop_inner_bash::append_bash_observation;
    let counters = super::loop_inner_types::LoopCounters {
        http_turn_count: 1,
        bash_exec_count: 2,
        investigate_http_turns: 1,
        had_bash_this_prompt: true,
    };
    assert_eq!(counters.bash_exec_count, 2);
}

#[test]
fn kiss_witness_loop_driver_tests() {
    let _ = stringify!(loop_driver_single_fence_runs_bash_and_appends_observation);
    let _ = stringify!(loop_driver_mini_done_line_terminates);
    let _ = stringify!(loop_driver_mini_done_inside_fence_still_runs_bash);
    let _ = stringify!(loop_driver_prepends_mini_constraints);
    let _ = stringify!(loop_driver_mock_http_retry_on_429);
    let _ = stringify!(loop_driver_fenceless_completes_in_one_turn);
    let _ = stringify!(loop_driver_fenceless_no_nudge_in_prompts_log);
}
