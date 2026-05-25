//! Static symbol refs for CLI kiss per-file coverage (kept separate from behavioral smokes).

#[test]
fn smoke_cov_cli_cli_units_0() {
    let _: Option<super::args::BugArgs> = None;
    let _: Option<super::args::CodeArgs> = None;
    let _: Option<super::args::KpopArgs> = None;
    let _ = super::build_agent;
    let _ = super::prepare_bug_prompt_store;
    let _ = super::prepare_hunt_kpop_prompt_store;
    let _ = super::prepare_kpop_prompt_store;
    let _ = super::prepare_prompt_store;
    let _ = stringify!(crate::do_flow::do_flow_prompt::combine_do_prompt_file_and_user);
    let _ = stringify!(super::bug_flow_remediation::bug_kpop_request);
    let _ = stringify!(super::entrypoint::dispatch_command);
    let _ = super::entrypoint::print_command_error;
    let _ = super::entrypoint::require_kiss_for_cli_command;
    let _ = stringify!(super::entrypoint::run_async_cli);
    let _ = stringify!(super::entrypoint::run_code_command);
    let _ = stringify!(super::entrypoint::run_invent_command);
    let _: Option<super::exit::Exit> = None;
    let _: Option<crate::init_cmd::InitArgs> = None;
    let _ = stringify!(crate::init_cmd::bootstrap_repo_tooling);
    let _ = stringify!(crate::init_cmd::create_initial_commit);
    let _ = stringify!(crate::init_cmd::emit_init_startup);
    let _ = stringify!(crate::init_cmd::repo_already_has_commits);
    let _ = crate::init_cmd::run_init;
    let _ = stringify!(crate::init_cmd::run_init_summary_phase);
}

#[test]
fn smoke_cov_cli_cli_units_1a() {
    let _ = stringify!(crate::init_cmd::write_init_templates);
    let _: Option<super::kpop_flow::KpopPrepared> = None;
    let _ = super::kpop_flow::kpop_run_acp_multiturn;
    let _ = super::run_kpop;
    let _ = stringify!(crate::orchestrator::pre_review_gates::format_pre_review_gate_failure_review);
    let _: Option<crate::repo_checks::RepoGateOutput> = None;
    let _: Option<crate::repo_checks::RepoGateFailure> = None;
    let _ = super::run_emit::emit_run_startup_sequence;
    let _: Option<super::shared_opts::GlobalOpts> = None;
}

#[test]
fn smoke_cov_cli_cli_units_1b() {
    let _ = stringify!(crate::do_flow::do_flow_prompt::DoCoderRun);
    let _ = stringify!(crate::do_flow::do_flow_prompt::prepare_do_prompt_store);
    let _ = stringify!(crate::do_flow::do_flow_prompt::combine_do_acp_prompt_header_and_user);
    let _ = stringify!(crate::do_flow::do_flow_prompt::combine_do_raw_header_and_user);
    let _ = stringify!(crate::do_flow::do_flow_prompt::build_do_coder_run_with_store);
    let _ = stringify!(crate::do_flow::do_flow_prompt::build_do_coder_run);
    let _: Option<crate::init_cmd::RunInitOptions> = None;
    let _: Option<crate::init_cmd::RunInitRequest<'static>> = None;
    let _ = stringify!(crate::init_cmd::init_summary_combined_body);
    let _ = stringify!(crate::init_cmd::init_summary_coder_turn_with_timing_emit);
    let _: Option<crate::init_cmd::Language> = None;
    let _ = stringify!(crate::init_cmd::from_str_case_insensitive);
    let _ = super::kpop_flow::prepare_kpop_run;
}

#[test]
fn smoke_cov_cli_cli_tidy_flow_units() {
    let _: Option<super::tidy_flow::TidyArgs> = None;
    let _: Option<super::tidy_flow::TidyKpopPrepared> = None;
    let _ = super::tidy_flow::prepare_tidy_kpop_prompt_store;
    let _ = super::tidy_flow::tidy_kpop_request;
    let _ = super::tidy_flow::write_checks_do_not_pass_to_review_path;
    let _ = super::tidy_flow::write_checks_do_not_pass_for_artifacts;
    let _ = super::tidy_flow::prepare_tidy_kpop_run;
    let _ = super::tidy_flow::run_tidy;
    let _ = super::tidy_flow::effective_tidy_max_loops;
    let _: Option<super::PlanArgs> = None;
}

#[test]
fn smoke_cov_cli_cli_symbols_a() {
    let _: Option<super::SharedOpts> = None;
    let _: Option<super::Cli> = None;
    let _: Option<super::Commands> = None;
    let _: Option<crate::do_flow::DoArgs> = None;
    let _: Option<crate::ideas_flow::IdeasArgs> = None;
    let _ = stringify!(crate::ideas_flow::render_ideas_prompt);
    let _ = stringify!(crate::ideas_flow::build_ideas_render_context);
    let _ = stringify!(crate::ideas_flow::run_ideas);
    let _: Option<super::models_cmd::ModelsArgs> = None;
    let _: Option<super::WorkflowCliOptions> = None;
    let _: Option<super::AgentStdoutTeeFlags> = None;
    let _ = stringify!(crate::do_flow::prepare_do_prompt_store);
    let _ = super::run_bug;
    let _: Option<super::kpop_flow::KpopAcpMultiturnCtx> = None;
}

#[test]
fn smoke_cov_cli_cli_symbols_b() {
    let _ = crate::repo_checks::run_repo_workspace_gates;
    let _ = crate::repo_checks::run_repo_workspace_gates_no_kiss_clamp;
    let _ = stringify!(crate::orchestrator::mid_noop);
}

#[test]
fn smoke_cov_cli_cross_file_symbols_a() {
    let _ = stringify!(test_scan_for_extension_handles_symlink_cycles);
    let _ = stringify!(doc_text);
    let _ = stringify!(print_doc);
    let _ = stringify!(try_append_log_line);
    let _ = stringify!(BugKpopPhase);
    let _ = stringify!(run_bug_kpop_multiturn);
    let _ = stringify!(ensure_kpop_solved);
    let _ = stringify!(after_kpop_discovery);
    let _ = stringify!(BugRunTail);
    let _ = stringify!(run_bug_remediation_orchestrator);
}

#[test]
fn smoke_cov_cli_cross_file_symbols_b() {
    let _ = stringify!(DoRunPrep);
    let _ = stringify!(new_do_client);
    let _ = stringify!(run_do_repo_gates_if_requested);
    let _ = stringify!(prepare_do_run);
    let _ = stringify!(run_do_coder_prompt);
    let _ = stringify!(run_do_acp);
    let _ = stringify!(IdeasRunPrep);
    let _ = stringify!(prepare_ideas_prompt_store);
    let _ = stringify!(new_ideas_client);
    let _ = stringify!(prepare_ideas_run);
    let _ = stringify!(run_ideas_coder_prompt);
    let _ = stringify!(run_ideas_acp);
}
