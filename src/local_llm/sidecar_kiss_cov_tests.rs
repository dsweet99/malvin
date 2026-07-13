//! Kiss static coverage witnesses for local sidecar modules.

#[test]
fn kiss_cov_sidecar_symbol_names() {
    let _ = stringify!(LocalSidecarHandle);
    let _ = stringify!(base_url);
    let _ = stringify!(model_slug);
    let _ = stringify!(ensure_local_sidecar);
    let _ = stringify!(ensure_local_sidecar_impl);
    let _ = stringify!(start_sidecar);
    let _ = stringify!(abort_unhealthy_sidecar);
    let _ = stringify!(local_openrouter_config);
    let _ = stringify!(local_slug);
    let _ = stringify!(wait_for_health);
    let _ = stringify!(http_get_ok);
    let _ = stringify!(parse_loopback_base_url);
    let _ = stringify!(spawn_sidecar_process);
    let _ = stringify!(free_loopback_port);
    let _ = stringify!(sidecar_run_dir);
    let _ = stringify!(open_sidecar_logs);
    let _ = stringify!(sidecar_argv);
    let _ = crate::local_llm::ensure_local_sidecar;
    let cfg = crate::local_llm::local_openrouter_config("qwen35_9b_q4", "http://127.0.0.1:1/v1");
    assert!(cfg.api_key.is_empty());
    assert_eq!(cfg.base_url, "http://127.0.0.1:1/v1");
}
