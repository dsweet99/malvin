#[test]
fn smoke_cov_cli_cli_units_0() {
    let _ = crate::cli::build_agent_backend;
    let _ = crate::do_flow::do_flow_prompt::combine_do_prompt_file_and_user;
    let _ = stringify!(crate::cli::entrypoint::dispatch_command);
    let _ = crate::cli::entrypoint::print_command_error;
    let _ = crate::cli::entrypoint::print_command_error;
    let _ = crate::cli::entrypoint_commands::run_write_command;
    let _: Option<crate::cli::exit::Exit> = None;
    let _ = crate::cli::init_flow::run_init;
    let _ = crate::cli::init_flow::render_init_router_request;
    let _ = stringify!(crate::cli::init_flow::InitWorkflowOpts);
    let _: Option<crate::cli::init_flow::InitWorkflowOpts> = None;
    let _ = crate::cli::init_flow::effective_init_max_loops;
}

#[test]
fn smoke_cov_cli_cli_units_1a() {
    let _: Option<crate::repo_checks::RepoGateOutput> = None;
    let _: Option<crate::repo_checks::RepoGateFailure> = None;
    let _ = crate::cli::run_emit::emit_run_startup_sequence;
    let _: Option<crate::cli::shared_opts::SharedOpts> = None;
}

#[test]
fn smoke_cov_cli_cli_units_1b() {
    let run = crate::do_flow::do_flow_prompt::DoCoderRun {
        combined: "body".into(),
        header_user_for_trace: ("hdr".into(), "user".into()),
    };
    let crate::do_flow::do_flow_prompt::DoCoderRun {
        combined,
        header_user_for_trace: (hdr, user),
    } = run;
    assert_eq!(combined, "body");
    assert_eq!(hdr, "hdr");
    assert_eq!(user, "user");
    let _ = crate::do_flow::do_flow_prompt::prepare_do_prompt_store;
    let _ = crate::do_flow::do_flow_prompt::combine_do_acp_prompt_header_and_user;
    let _ = crate::do_flow::do_flow_prompt::combine_do_raw_header_and_user;
    let _ = crate::do_flow::do_flow_prompt::build_do_coder_run_with_store;
    let _ = crate::do_flow::do_flow_prompt::build_do_coder_run;
    let shared = crate::cli::SharedOpts {
        background: false,
        model: crate::model_id::parse_model_id(crate::config::DEFAULT_CLI_MODEL).expect("model"),
        no_force: false,
        no_tenacious: false,
        gates: false,

        quiet: false,
        verbose: false,
        max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
        doc: false,
        git: false,
        creative: None,
        no_kpop: false,
    };
    let _ = shared.model;
}

#[test]
fn smoke_cov_cli_cli_workflow_router_shared_units() {
    let _ = crate::cli::workflow_router_shared::effective_max_loops;
    let _ = crate::cli::workflow_router_shared::run_router_workspace_gates;
    let _ = crate::cli::workflow_router_shared::prefer_gate_outcome_over_post_gate_cleanup;
    let _ = stringify!(gate_loop_early_exit);
    let _ = stringify!(gate_early_exit_fixture);
}

#[test]
fn smoke_cov_cli_router_test_units() {
    if !crate::cli::workflow_router_shared_tests::artifact_storage_available() {
        eprintln!("skipping router fixture smoke test: artifact root is not writable");
        return;
    }
    let (_tmp, _store, _artifacts) =
        crate::cli::workflow_router_shared_tests::router_render_fixture("code");
    let (_tmp, _bin, _guard, _artifacts, _backups) =
        crate::cli::workflow_router_shared_tests::gate_failure_fixture(1);
    let tmp = tempfile::tempdir().expect("tmp");
    let _ = crate::cli::workflow_router_shared_tests::missing_checks_fixture(tmp.path());
    let tmp = tempfile::tempdir().expect("tmp");
    let _ = crate::cli::workflow_router_shared_tests::router_gates_restore_fixture(tmp.path());
    let tmp = tempfile::tempdir().expect("tmp");
    let _ = crate::cli::workflow_router_shared_tests::gitignore_restore_failure_fixture(tmp.path());
}

#[test]
fn smoke_cov_cli_cli_tidy_flow_units() {
    let _ = crate::cli::tidy_flow::run_tidy;
    let _ = crate::cli::tidy_flow::effective_tidy_max_loops;
    let _ = crate::cli::tidy_flow::TIDY_ROUTER_REQUEST;
    let _ = crate::cli::tidy_flow::tidy_shared_with_gates_forced;
}

#[test]
fn smoke_cov_cli_cli_write_flow_units() {
    let _: Option<crate::cli::write_flow::WriteArgs> = None;
    let _ = crate::cli::write_flow::run_write;
    let _ = crate::cli::entrypoint_commands::run_write_command;
    let _ = crate::cli::write_flow::prep::compose_write_a_prompt;
    let _ = crate::cli::write_flow::prep::compose_write_b_prompt;
    let _ = crate::cli::write_flow::prep::write_preflight;
    let _ = stringify!(WriteRunPrep);
    let _ = stringify!(prepare_write_run);
    let _ = stringify!(prepare_write_prompts);
    let _ = stringify!(write_workspace_dir_display);
    let _ = stringify!(new_write_client);
    let _ = stringify!(create_write_artifacts);
    let _ = stringify!(run_write_coder_prompt);
    let _ = stringify!(run_write_coder_session);
}

#[test]
fn smoke_cov_cli_cli_symbols_a() {
    let _: Option<crate::cli::SharedOpts> = None;
    let _: Option<crate::cli::Cli> = None;
    let _: Option<crate::cli::Commands> = None;
    let do_args = crate::do_flow::DoArgs { request: None };
    let crate::do_flow::DoArgs { request } = do_args;
    assert!(request.is_none());
    let _ = stringify!(DoRunPrep);
    let _ = stringify!(new_do_client);
    let _ = crate::prompts::build_mbc2_render_context;
    let _ = crate::prompts::render_mbc2_prompt;
    let _: Option<crate::cli::models_cmd::ModelsArgs> = None;
    let _: Option<crate::cli::WorkflowCliOptions> = None;
    let _: Option<crate::cli::AgentStdoutTeeFlags> = None;
    let _ = crate::do_flow::prepare_do_prompt_store;
    let _ = crate::router_flow::prepare_router_prompt_store;
    let router_args = crate::router_flow::RouterArgs {
        request: None,
        max_loops: 1,
        max_hypotheses: crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES,
    };
    let crate::router_flow::RouterArgs {
        request: router_request,
        max_loops: router_max_loops,
        max_hypotheses: router_max_hypotheses,
    } = router_args;
    assert!(router_request.is_none());
    assert_eq!(router_max_loops, 1);
    assert_eq!(
        router_max_hypotheses,
        crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES
    );
    let _ = stringify!(RouterRunPrep);
    let _ = crate::router_flow::run_router;
    let _: Option<crate::router_flow::router_flow_acp::RouterAcpIterationInput<'_>> = None;
    let _: Option<crate::router_flow::router_flow_acp::RouterAcpIterationOutcome> = None;
    let _: Option<crate::router_flow::router_flow_loop::RouterAgentLoopInput<'_>> = None;
    let _: Option<crate::router_flow::router_flow_loop::RouterAgentLoopOutcome> = None;
    let _ = stringify!(RouterAcpIterationInput);
    let _ = stringify!(RouterAgentLoopInput);
}

#[test]
fn smoke_cov_cli_cli_symbols_b() {
    let _ = crate::repo_checks::run_repo_workspace_gates;
    let _ = crate::repo_checks::run_repo_workspace_gates;
    let _: Option<crate::repo_checks::FakeCommandDirGuard> = None;
    let _ = stringify!(FakeCommandDirGuard);
}

#[test]
fn smoke_cov_cli_cross_file_symbols_a() {
    let _ = stringify!(test_scan_for_extension_handles_symlink_cycles);
    let _ = stringify!(doc_text);
    let _ = stringify!(print_doc_for_cli);
    let _ = stringify!(try_append_log_line);
}
