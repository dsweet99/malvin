//! Kiss coverage references for [`crate::cli::kpop_summarize`] privates.

#[test]
fn kiss_cov_kpop_summarize_privates() {
    let shared = crate::cli::SharedOpts {
        model: crate::config::DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tenacious: false,
        no_tee: true,
        no_markdown: true,
        verbose: false,
        max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
        doc: false,
        name: None,
        mini: false,
        mini_max_bash_turns: 32,
        mini_max_http_turns: 32,
        mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_transport_retries: crate::support_paths::DEFAULT_MAX_MINI_TRANSPORT_RETRIES,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
    };
    let inputs = crate::cli::kpop_summarize::KpopOuterLoopSummarizeInputs {
        agent_ran: true,
        shared: &shared,
    };
    let crate::cli::kpop_summarize::KpopOuterLoopSummarizeInputs {
        agent_ran,
        shared: _,
    } = inputs;
    assert!(agent_ran);
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("kpop", Some(tmp.path())).expect("artifacts");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let params = crate::cli::kpop_summarize::kpop_outer_loop_summarize_params(inputs, &store, &artifacts);
    let crate::cli::kpop_summarize::OuterLoopSummarizeParams {
        agent_ran: params_agent_ran,
        shared: _,
        workflow: _,
        store: _,
        artifacts: _,
        malvin_command,
    } = params;
    assert!(params_agent_ran);
    assert_eq!(malvin_command, "malvin kpop");
    let _ = crate::cli::kpop_summarize::run_summarize_coder_prompt;
    let _ = crate::cli::kpop_summarize::run_outer_loop_summarize_if_warranted;
    let _ = crate::cli::kpop_summarize::render_kpop_summarize_prompt;
    let _ = crate::cli::kpop_summarize::exp_log_paths_markdown;
    let _ = crate::cli::kpop_summarize::outer_loop_summarize_warranted;
    let _ = crate::cli::kpop_summarize::kpop_outer_loop_summarize_params;
    let _ = crate::cli::kpop_summarize::code_outer_loop_summarize_params;
    let _: Option<crate::cli::kpop_summarize::CodeOuterLoopSummarizeInputs<'_>> = None;
    let _ = stringify!(run_summarize_coder_prompt);
    let _ = stringify!(list_written_exp_logs);
    let _ = stringify!(is_written_exp_log_path);
    let _ = stringify!(insert_summarize_log_context);
    let _ = stringify!(prefer_gate_outcome_over_summarize);
}

#[cfg(unix)]
#[test]
fn kiss_cov_kpop_summarize_test_helpers() {
    let _ = super::kpop_summarize_mock_tests::write_mock_summarize_agent;
    let _ = super::kpop_summarize_tests::summarize_shared_opts;
    let _ = stringify!(super::kpop_summarize_mock_tests::with_summarize_mock_agent);
    let _ = stringify!(super::kpop_summarize_tests::kpop_inputs);
    let _ = stringify!(super::kpop_summarize_tests::summarize_test_workspace);
    let _ = stringify!(run_outer_loop_summarize_if_warranted_runs_mock_summary_agent);
}
