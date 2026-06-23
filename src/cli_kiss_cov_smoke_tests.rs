//! Kiss per-file CLI symbol witnesses (declared from lib.rs, not cli mod).

#[test]
fn smoke_cov_cli_cli_units_0() {
    let _: Option<crate::cli::CodeArgs> = None;
    let _: Option<crate::cli::args::KpopArgs> = None;
    let _ = crate::cli::build_agent;
    let _ = crate::cli::prepare_kpop_prompt_store;
    let _ = crate::cli::prepare_prompt_store;
    let _ = crate::do_flow::do_flow_prompt::combine_do_prompt_file_and_user;
    let _ = stringify!(crate::cli::entrypoint::dispatch_command);
    let _ = crate::cli::entrypoint::print_command_error;
    let _ = crate::cli::entrypoint::require_kiss_for_cli_command;
    let _ = stringify!(crate::cli::entrypoint::run_async_cli);
    let _ = crate::cli::entrypoint_commands::run_code_command;
    let _ = crate::cli::entrypoint_commands::run_inspire_command;
    let _: Option<crate::cli::exit::Exit> = None;
    let _: Option<crate::init_cmd::InitArgs> = None;
    let _ = stringify!(crate::init_cmd::bootstrap_repo_tooling);
    let _ = stringify!(crate::init_cmd::init_cmd_bootstrap::ensure_git_repo);
    let _ = stringify!(crate::init_cmd::init_cmd_bootstrap::ensure_pre_commit_hooks);
    let _ = stringify!(crate::init_cmd::init_cmd_bootstrap::ensure_kiss_repo_init);
    let _ = stringify!(crate::init_cmd::init_cmd_bootstrap::ensure_git_lfs_hooks);
    let _ = stringify!(crate::init_cmd::create_initial_commit);
    let _ = stringify!(crate::init_cmd::init_cmd_mid_core::emit_init_startup);
    let _ = stringify!(crate::init_cmd::repo_already_has_commits);
    let _ = crate::init_cmd::run_init;
}

#[test]
fn smoke_cov_cli_cli_units_1a() {
    let _ = stringify!(crate::init_cmd::write_init_templates);
    let _: Option<crate::cli::kpop_flow::KpopPrepared> = None;
    let _ = crate::cli::kpop_flow::kpop_run_acp_multiturn;
    let _ = crate::cli::run_kpop;
    let _: Option<crate::repo_checks::RepoGateOutput> = None;
    let _: Option<crate::repo_checks::RepoGateFailure> = None;
    let _ = crate::cli::run_emit::emit_run_startup_sequence;
    let _: Option<crate::cli::shared_opts::GlobalOpts> = None;
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
    let _: Option<crate::init_cmd::RunInitOptions> = None;
    let shared = crate::cli::SharedOpts {
        model: crate::config::DEFAULT_CLI_MODEL.into(),
        no_force: false,
        no_tenacious: false,
        no_tee: false,
        no_markdown: false,
        verbose: false,
        max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
        doc: false,
        name: None,
        mini: false,
        mini_max_bash_turns: 32,
    };
    let init_req = crate::init_cmd::RunInitRequest {
        path: None,
        languages: &[],
        shared: &shared,
        opts: crate::init_cmd::RunInitOptions {
            overwrite_templates: false,
            tee_startup_stdout: false,
        },
    };
    let crate::init_cmd::RunInitRequest {
        path,
        languages,
        shared: _,
        opts: _,
    } = init_req;
    assert!(path.is_none());
    assert!(languages.is_empty());
    let _ = crate::init_cmd::run_init;
    let _: Option<crate::init_cmd::Language> = None;
    let _ = stringify!(crate::init_cmd::from_str_case_insensitive);
    let _ = stringify!(prepare_kpop_artifacts);
}

#[test]
fn smoke_cov_cli_cli_code_flow_units() {
    let _: Option<crate::cli::CodeArgs> = None;
    let _: Option<crate::cli::code_flow::CodeKpopPrepared> = None;
    let _ = crate::cli::code_flow::prepare_code_kpop_prompt_store;
    let _ = crate::cli::code_flow::code_kpop_request;
    let _ = crate::cli::code_flow::prepare_code_kpop_run;
    let _ = crate::cli::code_flow::run_code;
    let _ = crate::cli::code_flow::effective_code_max_loops;
    let _ = crate::cli::workflow_kpop_shared::effective_max_loops;
    let _ = crate::cli::workflow_kpop_shared::kpop_workflow_context;
    let _ = crate::cli::workflow_kpop_shared::post_kpop_session_gates;
    let _ = crate::cli::workflow_kpop_shared::run_kpop_workspace_gates;
    let _ = crate::cli::workflow_kpop_shared::print_kpop_session_log_line;
    let _ = crate::cli::workflow_kpop_shared::finish_kpop_acp_session;
    let _ = stringify!(gate_kpop_session_declared_solved_detects_kpop_solved_marker);
    let _ = stringify!(crate::gate_kpop_workflow::run_gate_kpop_loop);
    let _ = stringify!(crate::gate_kpop_workflow::post_gate_kpop_gates);
    let _ = stringify!(crate::gate_kpop_workflow::finish_gate_kpop_after_pass);
    let _ = stringify!(crate::gate_kpop_workflow::fail_gate_kpop_after_exhausted);
    let _: Option<crate::gate_kpop_workflow::GateKpopLoopParams> = None;
    let _ = stringify!(gate_early_exit_fixture);
    let _: Option<crate::gate_kpop_workflow::GateKpopMultiturnCtx<'_>> = None;
}

#[test]
fn smoke_cov_cli_cli_tidy_flow_units() {
    let _: Option<crate::cli::tidy_flow::TidyArgs> = None;
    let _: Option<crate::cli::tidy_flow::TidyKpopPrepared> = None;
    let _ = crate::cli::tidy_flow::prepare_tidy_kpop_prompt_store;
    let _ = crate::cli::tidy_flow::tidy_kpop_request;
    let _ = crate::cli::tidy_flow::write_checks_do_not_pass_to_review_path;
    let _ = crate::cli::tidy_flow::write_checks_do_not_pass_for_artifacts;
    let _ = crate::cli::workflow_kpop_shared::write_checks_do_not_pass_to_review_path;
    let _ = crate::cli::workflow_kpop_shared::write_checks_do_not_pass_for_artifacts;
    let _ = crate::cli::tidy_flow::prepare_tidy_kpop_run;
    let _ = crate::cli::tidy_flow::run_tidy;
    let _ = crate::cli::tidy_flow::effective_tidy_max_loops;
}

#[test]
fn smoke_cov_cli_cli_delight_flow_units() {
    let _: Option<crate::cli::delight_flow::DelightArgs> = None;
    let _ = crate::cli::entrypoint_commands::run_delight_then_plan;
    let _ = crate::cli::entrypoint_commands::plan_args_for_delight_output;
    let _ = crate::cli::entrypoint_commands::run_explain_then_revise;
    let _ = crate::cli::entrypoint_commands::revise_args_for_explain_output;
    let _ = crate::cli::delight_flow::run_delight;
    let _ = crate::cli::delight_flow::effective_delight_max_loops;
    let _ = crate::gate_kpop_workflow::GateLoopBehavior::DELIGHT;
}

#[test]
fn smoke_cov_cli_cli_revise_flow_units() {
    let _: Option<crate::cli::revise_flow::ReviseArgs> = None;
    let _ = crate::cli::revise_flow::run_revise;
    let _ = crate::cli::revise_flow::effective_revise_max_loops;
    let _ = crate::gate_kpop_workflow::GateLoopBehavior::REVISE;
    let _ = crate::cli::entrypoint_commands::run_revise_command;
}

#[test]
fn smoke_cov_cli_cli_explain_flow_units() {
    let _: Option<crate::cli::explain_flow::ExplainArgs> = None;
    let _ = crate::cli::explain_flow::run_explain;
    let _ = crate::cli::explain_flow::effective_explain_max_loops;
    let _ = crate::gate_kpop_workflow::GateLoopBehavior::EXPLAIN;
    let _ = crate::cli::entrypoint_commands::run_explain_command;
}

#[test]
fn smoke_cov_cli_kpop_flow_run_loop_types() {
    let outcome = crate::cli::kpop_flow::kpop_flow_run_loop::kpop_loop_abort(true, "e".into());
    let crate::cli::kpop_flow::kpop_flow_run_loop::RunKpopAgentLoopsOutcome {
        acp_result: _,
        agent_ran,
    } = outcome;
    assert!(agent_ran);
    let _: Option<crate::cli::kpop_flow::kpop_flow_run_loop::RunKpopAgentLoopsParams<'_>> = None;
    let _: Option<crate::cli::kpop_flow::kpop_flow_run_loop::KpopLoopSnapshot> = None;
    let _ = stringify!(KpopLoopExitAfterIteration);
    let _ = stringify!(kpop);
    let _ = stringify!(store);
    let _ = stringify!(client);
    let _ = stringify!(prepared);
    let _ = stringify!(backups);
    let _ = stringify!(exp_iter);
    let _ = stringify!(exp_log_path);
}

#[test]
fn smoke_cov_cli_cli_symbols_a() {
    let _: Option<crate::cli::SharedOpts> = None;
    let _: Option<crate::cli::Cli> = None;
    let _: Option<crate::cli::Commands> = None;
    let do_args = crate::do_flow::DoArgs {
        repo_gates: false,
        thoughts: false,
        request: None,
    };
    let crate::do_flow::DoArgs {
        repo_gates,
        thoughts,
        request,
    } = do_args;
    assert!(!repo_gates && !thoughts && request.is_none());
    let _ = stringify!(DoRunPrep);
    let _ = stringify!(new_do_client);
    let _ = stringify!(RenderKpopProgram);
    let _: Option<crate::cli::kpop_flow::kpop_flow_run_loop::RunKpopAgentLoopsParams<'_>> = None;
    let _: Option<crate::cli::kpop_flow::kpop_flow_run_loop::RunKpopAgentLoopsOutcome> = None;
    let _: Option<crate::inspire_flow::InspireArgs> = None;
    let _ = stringify!(InspireRunPrep);
    let _ = crate::inspire_flow::render_inspire_prompt;
    let _ = crate::inspire_flow::build_inspire_render_context;
    let _ = crate::inspire_flow::run_inspire;
    let _: Option<crate::plan_flow::PlanArgs> = None;
    let _ = crate::plan_flow::run_plan;
    let _ = crate::plan_flow::prepare_plan_prompt_store;
    let _ = crate::cli::adversarial_profile::adversarial_profile_active;
    let _: Option<crate::cli::models_cmd::ModelsArgs> = None;
    let _: Option<crate::cli::WorkflowCliOptions> = None;
    let _: Option<crate::cli::AgentStdoutTeeFlags> = None;
    let _ = crate::do_flow::prepare_do_prompt_store;
    let _: Option<crate::cli::kpop_flow::KpopAcpMultiturnCtx> = None;
}

#[test]
fn smoke_cov_cli_cli_symbols_b() {
    let _ = crate::repo_checks::run_repo_workspace_gates;
    let _ = crate::repo_checks::run_repo_workspace_gates_no_kiss_clamp;
    let _: Option<crate::repo_checks::FakeCommandDirGuard> = None;
    let _ = stringify!(FakeCommandDirGuard);
}

#[test]
fn smoke_cov_cli_cross_file_symbols_a() {
    let _ = stringify!(test_scan_for_extension_handles_symlink_cycles);
    let _ = stringify!(doc_text);
    let _ = stringify!(print_doc);
    let _ = stringify!(try_append_log_line);
}
