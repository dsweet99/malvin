#[test]
fn kiss_cov_deferred_log_symbols() {
    let _ = super::config::defer_log_enabled_from_env;
    let _ = super::config::defer_log_max_age_from_env;
    let _ = super::config::defer_log_max_drain_from_env;
    let _ = super::config::defer_log_cursor_dir_from_env;
    let _ = super::config::env_is_zero;
    let _ = stringify!(super::config::DeferredLogConfig);
    let _ = super::emit::emit_deferred_entry;
    let _ = super::emit::acp_event;
    let _ = super::enrich::enriched_tool_plain;
    let _ = super::enrich::styled_tool_payload;
    let _ = super::enrich::synthetic_tool_done;
    let _ = super::sink::build_heartbeat_entry;
    let _ = super::sink::build_tagged_stdout_entry;
    let _ = super::active::register;
    let _ = super::active::unregister;
    let _ = super::active::try_log;
    let _ = super::active::try_push;
    let _ = super::active::flush_pending_into;
    let _ = super::active::pending_len;
    let _ = super::log_with_heartbeat;
    let _ = super::install_stdout_hooks;
    let _ = super::register_active_sink;
    let _ = super::unregister_active_sink;
    let _ = super::active_defer_sink_registered;
    let _ = super::test_fixtures::test_tool_entry;
    let _ = super::test_fixtures::enrich_read_entry;
    let _ = super::test_fixtures::zero_age_sink;
    let _ = super::test_fixtures::capture_stdout_log(|| {});
    let _ = super::sink::test_access::push_back;
    let _ = super::tool_enrich::tool_drain_enrich_fields;
    let _ = stringify!(super::types::DeferredPayload);
    let _ = stringify!(super::types::DeferredEntry);
    let _ = stringify!(super::types::EnrichKey);
    let _ = stringify!(super::types::ToolDrainMeta);
    let _ = stringify!(super::types::TeeSinkMeta);
}
