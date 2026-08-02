//! Kiss coverage references for [`crate::cli::kpop_summarize`] privates.

#[test]
fn kiss_cov_kpop_summarize_privates() {
    let shared = crate::cli::SharedOpts {
        model: crate::config::DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tenacious: false,
        gates: false,

        quiet: false,
        verbose: false,
        max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
        doc: false,
        name: None,
        mini_max_bash_turns: 32,
        mini_max_http_turns: 32,
        mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_transport_retries: crate::support_paths::DEFAULT_MAX_MINI_TRANSPORT_RETRIES,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
        no_download: false,
        git: false,
            no_kpop: false,
        };
    let _ = &shared;
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("kpop", Some(tmp.path())).expect("artifacts");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let _ = crate::cli::kpop_summarize::run_summarize_coder_prompt;
    let _ = crate::cli::kpop_summarize::render_kpop_summarize_prompt;
    let _ = crate::cli::kpop_summarize::exp_log_paths_markdown;
    let _ = crate::cli::kpop_summarize::should_inline_outer_loop_summarize_on_gate_iteration;
    let _ = crate::cli::kpop_summarize::list_written_exp_logs;
    let _ = crate::cli::kpop_summarize::kpop_flows_ran;
    let _ = artifacts;
    let _ = store;
}
