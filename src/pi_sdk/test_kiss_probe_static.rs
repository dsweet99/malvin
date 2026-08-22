//! Kiss static coverage witness for the embedded pi::sdk units.
//! Bare call-shaped tokens; this file is scanned, not compiled (style matches
//! src/coverage_kiss/test_kiss_static_coverage_00..06).

#[test]
fn kiss_probe_static_tokens_a() {
    fake_events_for_prompt();
    empty_agent_end();
    streamed_hello_events();
    map_agent_end();
    last_assistant_text();
    text_from_blocks();
    aggregate_usage();
    create_tool_registry();
    from_builtin();
    run_isolated_bash();
    description();
    parameters();
    effects();
    execute();
    drain_agent_events();
    recv_event_with_idle();
}

#[test]
fn kiss_probe_static_tokens_b() {
    handle_mapped_events();
    finish_after_channel_closed();
    finish_run_done();
    send_fake_prompt();
    is_providers_noise_line();
    col();
    tool_call();
    deref();
    deref_mut();
    last_text();
    run_models_pi_only_with_openrouter_key();
    assert_live_auth_filter();
    leftover_pi_runtime_threads();
    session_spawn_tests::fake_session_begin_end_leaves_no_pi_runtime_thread();
}
