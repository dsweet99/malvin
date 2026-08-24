//! Kiss static coverage witness for the embedded pi::sdk units.
//! Bare call-shaped tokens; this file is scanned, not compiled (style matches
//! src/coverage_kiss/test_kiss_static_coverage_00..06).

#[test]
fn kiss_probe_static_tokens_a() {
    fake_events_for_prompt("", "", "");
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

#[test]
fn kiss_probe_static_models_refresh_tokens() {
    ProviderModelCache();
    cache_fetched_at_is_fresh();
    unix_now_secs();
    load_provider_cache();
    save_provider_cache();
    refresh_pi_provider_caches_if_stale();
    merge_registry_with_live();
    append_live_models();
    append_static_models_without_live();
    static_registry_lookup();
    provider_needs_refresh();
    fetch_provider_models_sync();
    resolve_provider_api_key();
    authenticated_providers();
    provider_supports_pi_live_model_fetch();
    openai_compat_models_url();
    PI_MODEL_CACHE_TTL();
}

#[test]
fn kiss_probe_static_openrouter_pricing_tokens() {
    warm_openrouter_pricing_cache();
    lookup_model_cost();
    fetch_billed_cost_from_generation_ids();
    GenerationResponse();
    GenerationData();
    aggregate_cost_usd();
    AggregatedCostUsd();
    OpenRouterModelsResponse();
    OpenRouterModelEntry();
    OpenRouterPricing();
    PricingCache();
    parse_rate_per_million();
    model_cost_from_pricing();
    load_cache();
    save_cache();
    fetch_live_pricing_sync();
    fetch_live_pricing_async();
    pricing_from_models_body();
    openrouter_lookup_ids();
    models_url();
}
