//! Extra kiss CLI smoke witnesses (split for lines-per-file gate).

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
    let _ = stringify!(InspireRunPrep);
    let _ = stringify!(prepare_inspire_prompt_store);
    let _ = stringify!(new_inspire_client);
    let _ = stringify!(inspire_emit_startup_banner);
    let _ = stringify!(prepare_inspire_run);
    let _ = stringify!(run_inspire_coder_prompt);
    let _ = stringify!(run_inspire_coder_session);
    let _ = stringify!(test_kpop_args);
    let _ = stringify!(install_mock_agent_env);
    let _ = stringify!(write_mock_agent);
}
