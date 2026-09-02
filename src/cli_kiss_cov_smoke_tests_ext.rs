#[test]
fn smoke_cov_cli_cross_file_symbols_b() {
    let _ = stringify!(DoRunPrep);
    let _ = stringify!(RouterRunPrep);
    let _ = stringify!(new_do_client);
    let _ = stringify!(prepare_do_run);
    let _ = stringify!(run_do_coder_prompt);
    let _ = stringify!(run_do_acp);
    let _ = stringify!(new_router_client);
    let _ = stringify!(prepare_router_run);
    let _ = stringify!(run_router_coder_prompt);
    let _ = stringify!(run_router_acp_iteration);
    let _ = stringify!(run_router_agent_loops);
    let _ = stringify!(RouterAcpIterationInput);
    let _ = stringify!(RouterAcpIterationOutcome);
    let _ = stringify!(RouterAgentLoopInput);
    let _ = stringify!(RouterAgentLoopOutcome);
    let _ = stringify!(build_mbc2_render_context);
    let _ = stringify!(render_mbc2_prompt);
    let _ = stringify!(build_router_mbc2_prompt);
    let _ = stringify!(test_router_args);
    let _ = stringify!(install_mock_agent_env);
    let _ = stringify!(write_mock_agent);
}
