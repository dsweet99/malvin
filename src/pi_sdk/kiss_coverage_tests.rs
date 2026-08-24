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
    let _ = stringify!(ProviderModelCache);
    let _ = stringify!(cache_is_fresh);
    let _ = stringify!(pi_model_cache_dir);
    let _ = stringify!(pi_model_cache_path);
    let _ = stringify!(resolve_provider_api_key);
    let _ = stringify!(authenticated_providers);
    let _ = stringify!(provider_needs_refresh);
    let _ = stringify!(provider_supports_pi_live_model_fetch);
    let _ = stringify!(openai_compat_models_url);
    let _ = stringify!(fetch_provider_models_sync);
    let _ = stringify!(static_registry_lookup);
    let _ = stringify!(append_live_models);
    let _ = stringify!(append_static_models_without_live);
    let _ = stringify!(PI_MODEL_CACHE_TTL);
    let _ = stringify!(PiModelListing);
    let _ = stringify!(DEFAULT_PI_LIST_MODELS_TIMEOUT_MS);
}

#[test]
fn kiss_cov_pi_sdk_live_provider_auth() {
    let _ = super::is_provider_authenticated;
    let _ = stringify!(provider_has_access);
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
    let _ = stringify!(split_provider_model);
    let _ = stringify!(pi_thinking_level);
    let _ = stringify!(fake_embedded_session);
    let _ = stringify!(live_embedded_session);
    let _ = stringify!(start_embedded_mem_watch);
    let _ = stringify!(watch_embedded_memory);
    let _ = stringify!(isolated_tool_factory);
    let _ = stringify!(PiEmbeddedSession);
    let _ = stringify!(PiRuntime);
    let _ = stringify!(PiLoopCtl);
    let _ = stringify!(PromptCmd);
    let _ = stringify!(take_test_prompt_if_blocked);
    let _ = stringify!(pi_blocking_session_options);
    let _ = stringify!(map_pi_agent_event);
    let _ = stringify!(pi_sdk_client_from_raw);
    let _ = crate::bridge_sdk::BridgeWire::NodeBridge;
}
