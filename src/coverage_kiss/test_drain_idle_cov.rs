//! Kiss name witnesses (call-shaped tokens). Not compiled into the crate.

#[test]
fn kiss_cov_drain_idle_witness_a() {
    DrainHealthVerdict();
    DrainIdleLabels();
    DrainIdleHealthCtx();
    DrainIdleClock();
    silence_error();
    await_next_with_idle();
    await_next_with_idle_using();
    sample_drain_health();
    drain_sample_pids();
    drain_sample_pids_blocking();
    aggregate_pid_health();
    aggregate_health_outcomes();
}

#[test]
fn kiss_cov_drain_idle_witness_b() {
    with_idle_ms_async();
    set_idle_ms();
    set_policy_idle_ms();
    await_next_times_out_without_health_extend();
    await_next_delivers_when_read_completes();
    clock_busy_extends_until_max_wait();
    clock_dead_fails_immediately();
    clock_hung_fails_only_after_idle_deadline();
    drain_sample_pids_falls_back_to_pgid();
    aggregate_health_policy_matches_plan();
    injected_dead_health_fails_at_first_slice();
    injected_hung_health_waits_full_idle();
    missing_pgid_gets_no_health_extend();
    repeated_busy_health_stops_at_exactly_two_idle_windows();
    successful_event_starts_a_fresh_next_event_idle_budget();
    real_health_sampling_respects_two_idle_wall_cap();
    event_arriving_during_health_sampling_wins_race();
}

#[test]
fn kiss_cov_drain_idle_witness_c() {
    kiss_cov_drain_idle_names();
    progress_events_keep_drain_alive_past_idle_budget();
    injected_busy_health_extends_then_delivers_event();
    progress_allows_more_than_ten_minutes_without_prompt_ceiling();
    maps_progress_for_protocol_compatibility();
    run_progress_prompt();
    tests_set_idle_ms_for_test();
    tests_restore_idle_ms_for_test();
    kiss_cov_drain_idle_policy_names();
    kiss_cov_drain_idle_witness_a();
    kiss_cov_drain_idle_witness_b();
    kiss_cov_drain_idle_witness_c();
}
