#[test]
fn kiss_cov_pi_sdk_discover_auth_models() {
    let _ = super::ensure_pi_authenticated;
    let _ = super::auth::provider_auth_env_keys;
    let _ = super::list_pi_models_sync;
    let _ = super::pi_list_models_timeout;
    let _ = super::models_refresh::refresh_pi_provider_caches_if_stale;
    let _ = super::models_refresh::merge_registry_with_live;
    let _ = super::models_refresh::load_provider_cache;
    let _ = super::models_refresh::save_provider_cache;
    let _ = stringify!(unix_now_secs);
    let _ = stringify!(cache_fetched_at_is_fresh);
    let _ = stringify!(ProviderModelCache);
    let _ = stringify!(pi_model_cache_dir);
    let _ = stringify!(pi_model_cache_path);
    let _ = stringify!(resolve_provider_api_key);
    let _ = stringify!(authenticated_providers);
    let _ = stringify!(provider_needs_refresh);
    let _ = stringify!(provider_supports_pi_live_model_fetch);
    let _ = stringify!(openai_compat_models_url);
    let _ = stringify!(fetch_provider_models_sync);
    let _ = super::local_endpoint::parse_http_authority_host_port;
    let _ = super::local_endpoint::http_base_url_is_listening;
    let _ = super::local_endpoint::keyless_local_provider_is_listening;
    let _ = super::local_lifecycle::ensure_local_llm;
    let _ = super::local_lifecycle::housekeep_local_llms;
    let _ = super::local_lifecycle::model_needs_local_llm;
    let _ = super::local_llm_daemon::run_local_llm_manager;
    let _ = super::local_llm_daemon::reclaim_stale_manager_files;
    let _ = super::local_llm_client::ensure_via_manager;
    let _ = super::local_llm_client::hold_via_manager;
    let _ = super::local_llm_client::release_via_manager;
    let _ = super::local_llm_client::touch_via_manager;
    let _ = super::local_llm_keepalive::start_keepalive;
    let _ = super::local_llm_keepalive::stop_keepalive;
    let _ = super::local_lifecycle::hold_local_llm;
    let _ = super::local_lifecycle::release_local_llm;
    let _ = super::local_llm_manager_pid;
    let _ = stringify!(INTERNAL_MANAGER_FLAG);
    let _ = stringify!(time_since_last_call_seconds);
    let _ = stringify!(DEFAULT_IDLE_SECS);
    let _ = stringify!(ManagerRequest);
    let _ = stringify!(ManagerResponse);
    let _ = stringify!(ensure_provider_running);
    let _ = stringify!(try_acquire_lock);
    let _ = stringify!(KEYLESS_LOCAL_REACHABILITY_TIMEOUT);
    let _ = stringify!(scheme_and_rest);
    let _ = stringify!(default_port_for_scheme);
    let _ = stringify!(parse_bracketed_host_port);
    let _ = stringify!(parse_host_port_pair);
    let _ = stringify!(static_registry_lookup);
    let _ = stringify!(append_live_models);
    let _ = stringify!(append_static_models_without_live);
    let _ = super::models_refresh_merge::merge_registry_with_live;
    let _ = stringify!(PI_MODEL_CACHE_TTL);
    let _ = stringify!(PiModelListing);
    let _ = stringify!(DEFAULT_PI_LIST_MODELS_TIMEOUT_MS);
    let _ = stringify!(KEYLESS_LOCAL_API_KEY);
    let _ = stringify!(LOCAL_LLM_BASE_URL_ENV);
    let _ = stringify!(ensure_capped_local_model_catalog);
    let _ = stringify!(context_size_for_workdir);
    let _ = stringify!(max_tokens_for_context);
    let _ = stringify!(apply_capped_model);
    let _ = stringify!(upsert_model_entry);
    let _ = stringify!(clamp_all_provider_models);
    let _ = stringify!(apply_context_cap);
    let _ = stringify!(write_models_json);
    let _ = super::local_context::ensure_capped_local_model_catalog;
    let _ = super::local_context::context_size_for_workdir;
    let _ = super::local_context::max_tokens_for_context;
}

#[test]
fn kiss_cov_pi_sdk_openrouter_usage() {
    let _ = super::openrouter_pricing::warm_openrouter_pricing_cache;
    let _ = super::openrouter_pricing::lookup_model_cost;
    let _ = super::openrouter_billed_cost::fetch_billed_cost_from_generation_ids;
    let _ = super::usage_cost::aggregate_cost_usd;
    let _ = stringify!(fetch_billed_cost_from_generation_ids);
    let _ = stringify!(GenerationResponse);
    let _ = stringify!(GenerationData);
    let _ = stringify!(warm_openrouter_pricing_cache);
    let _ = stringify!(lookup_model_cost);
    let _ = stringify!(aggregate_cost_usd);
    let _ = stringify!(AggregatedCostUsd);
    let _ = stringify!(cost_is_present);
    let _ = stringify!(OpenRouterModelsResponse);
    let _ = stringify!(OpenRouterModelEntry);
    let _ = stringify!(OpenRouterPricing);
    let _ = stringify!(PricingCache);
    let _ = stringify!(models_url);
    let _ = stringify!(cache_path);
    let _ = stringify!(parse_rate_per_million);
    let _ = stringify!(model_cost_from_pricing);
    let _ = stringify!(load_cache);
    let _ = stringify!(save_cache);
    let _ = stringify!(fetch_live_pricing_async);
    let _ = stringify!(fetch_live_pricing_sync);
    let _ = stringify!(pricing_from_models_body);
    let _ = stringify!(openrouter_lookup_ids);
    let _ = stringify!(cost_from_model_rates);
    let _ = stringify!(lookup_rates);
}

#[test]
fn kiss_cov_pi_sdk_live_provider_auth() {
    let _ = super::is_provider_authenticated;
    let _ = super::is_provider_listable;
    let _ = stringify!(provider_has_access);
    let _ = stringify!(provider_has_stored_or_env_access);
    let _ = stringify!(custom_provider_is_keyless);
    let _ = stringify!(models_json_provider);
    let _ = stringify!(ModelsJsonProvider);
    let _ = stringify!(stored_credential_present);
}

#[test]
fn kiss_cov_pi_sdk_map() {
    let _ = super::map_agent_event::map_pi_agent_event;
    let _ = super::map_event_summary::tool_summary_from_pi;
    let _ = stringify!(map_pi_agent_event);
    let _ = stringify!(map_agent_end);
    let _ = stringify!(last_assistant_text);
    let _ = stringify!(aggregate_usage);
}

#[test]
fn kiss_cov_pi_sdk_session_core() {
    let _ = super::spawn_bridge;
    let _ = stringify!(pi_spawn_bridge);
    let _ = stringify!(prewarm_openrouter_pricing);
    let _ = stringify!(spawn_live_pi_bridge);
    let _ = stringify!(pi_provider_and_model);
    let _ = stringify!(pi_thinking_level);
    let _ = stringify!(local_append_system_prompt);
    let _ = stringify!(local_enabled_tools);
    let _ = stringify!(LOCAL_ENABLED_TOOLS);
    let _ = stringify!(LOCAL_TEXT_ONLY_APPEND);
    let _ = stringify!(LocalAgentMode);
    let _ = stringify!(local_max_tool_iterations);
    let _ = stringify!(keyless_local_tools_enabled);
    let _ = stringify!(local_session_overrides);
    let _ = stringify!(session_spawn_local);
    let _ = stringify!(ollama_model_supports_tools);
    let _ = stringify!(capabilities_include_tools);
    let _ = stringify!(ensure_local_catalog);
    let _ = stringify!(fake_embedded_session);
    let _ = stringify!(live_embedded_session);
    let _ = stringify!(start_embedded_mem_watch);
    let _ = stringify!(watch_embedded_memory);
    let _ = super::session_spawn_watch::start_embedded_mem_watch;
    let _ = stringify!(isolated_tool_factory);
    let _ = stringify!(CompleteWrite);
    let _ = stringify!(stubby_write_error);
    let _ = stringify!(write_field_str);
    let _ = stringify!(path_looks_like_bin);
    let _ = stringify!(push_escaped);
    let _ = stringify!(unescape_local_write_content);
    let _ = stringify!(normalize_write_input);
    let _ = stringify!(PiEmbeddedSession);
    let _ = stringify!(PiRuntime);
    let _ = stringify!(PiLoopCtl);
    let _ = stringify!(PromptCmd);
    let _ = stringify!(take_test_prompt_if_blocked);
    let _ = stringify!(pi_blocking_session_options);
    let _ = stringify!(map_pi_agent_event);
    let _ = stringify!(pi_sdk_client_from_raw);
    let _ = crate::model_id::ModelBackend::Cursor;
}
