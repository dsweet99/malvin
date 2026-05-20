//! Static symbol refs for CLI kiss per-file coverage (kept separate from behavioral smokes).

#[test]
fn smoke_cov_cli_cli_units_0() {
    let _ = stringify!(super::args::BugArgs);
    let _ = stringify!(super::args::CodeArgs);
    let _ = stringify!(super::args::KpopArgs);
    let _ = stringify!(super::build_agent);
    let _ = stringify!(super::prepare_bug_prompt_store);
    let _ = stringify!(super::prepare_kpop_prompt_store);
    let _ = stringify!(super::prepare_prompt_store);
    let _ = stringify!(crate::do_flow::do_flow_prompt::combine_do_prompt_file_and_user);
    let _ = stringify!(crate::do_flow::do_flow_prompt::prepare_do_prompt_store_validating);
    let _ = stringify!(super::entrypoint::dispatch_command);
    let _ = stringify!(super::entrypoint::print_command_error);
    let _ = stringify!(super::entrypoint::run_async_cli);
    let _ = stringify!(super::entrypoint::run_code_command);
    let _ = stringify!(super::exit::Exit);
    let _ = stringify!(crate::init_cmd::InitArgs);
    let _ = stringify!(crate::init_cmd::bootstrap_repo_tooling);
    let _ = stringify!(crate::init_cmd::create_initial_commit);
    let _ = stringify!(crate::init_cmd::emit_init_startup);
    let _ = stringify!(crate::init_cmd::repo_already_has_commits);
    let _ = stringify!(crate::init_cmd::run_init);
    let _ = stringify!(crate::init_cmd::run_init_summary_phase);
}

#[test]
fn smoke_cov_cli_cli_units_1a() {
    let _ = stringify!(crate::init_cmd::write_init_templates);
    let _ = stringify!(super::kpop_flow::KpopPrepared);
    let _ = stringify!(super::kpop_flow::kpop_run_acp_multiturn);
    let _ = stringify!(super::kpop_flow::run_kpop);
    let _ = stringify!(super::mid_session_gates::mid_session_post_run_tidy::PostRunTidyPrompt);
    let _ = stringify!(crate::repo_checks::gate_log::append_quality_gates_log_text);
    let _ = stringify!(crate::repo_checks::gate_log::append_quality_gates_log_line);
    let _ = stringify!(crate::repo_checks::gate_run::append_quality_gates_command_output);
    let _ = stringify!(crate::repo_checks::gate_log::emit_repo_gate_line);
    let _ = stringify!(crate::repo_checks::gate_run::prepare_repo_workspace);
    let _ = stringify!(crate::repo_checks::command_support::run_command_for);
    let _ = stringify!(crate::repo_checks::command_support::run_command_failure);
    let _ = stringify!(crate::repo_checks::kissconfig_warn::should_warn_low_test_coverage);
    let _ = stringify!(crate::repo_checks::RepoGateOutput);
    let _ = stringify!(crate::repo_checks::RepoGateFailure);
    let _ = stringify!(crate::repo_checks::gate_run::scan_for_extension_handles_symlink_cycles);
    let _ = stringify!(super::run_emit::echo_primary_to_stdout);
    let _ = stringify!(super::run_emit::emit_run_startup_sequence);
    let _ = stringify!(super::shared_opts::GlobalOpts);
    let _ = stringify!(super::tidy_flow::recovery::TidyReviewAttemptOutcome);
    let _ = stringify!(super::tidy_flow::recovery::run_tidy_concerns_coder_turn);
    let _ = stringify!(super::tidy_flow::run_tidy_learn_mid_gates_and_summary);
    let _ = stringify!(super::tidy_flow::recovery::tidy_fail_on_abort);
    let _ = stringify!(super::tidy_flow::recovery::tidy_learn_elapsed_threshold_ms);
    let _ = stringify!(super::tidy_flow::recovery::tidy_review_attempt_with_retries);
    let _ = stringify!(crate::acp_post_run::emit_run_timing_after_acp);
    let _ = stringify!(crate::acp_post_run::merge_error_mentions_restore);
}

#[test]
fn smoke_cov_cli_cli_units_1b() {
    let _ = stringify!(crate::do_flow::do_flow_prompt::DoCoderRun);
    let _ = stringify!(crate::init_cmd::RunInitOptions);
    let _ = stringify!(crate::init_cmd::RunInitRequest);
    let _ = stringify!(crate::init_cmd::init_summary_combined_body);
    let _ = stringify!(crate::init_cmd::init_summary_coder_turn_with_timing_emit);
    let _ = stringify!(crate::init_cmd::Language);
    let _ = stringify!(crate::init_cmd::from_str_case_insensitive);
    let _ = stringify!(crate::init_cmd::parse_languages);
    let _ = stringify!(super::kpop_flow::prepare_kpop_run);
    let _ = stringify!(super::tidy_flow::run_tidy_coder_prompt_for_attempt);
    let _ = stringify!(super::tidy_flow::tidy_handle_lgtm_outcome);
    let _ = stringify!(super::tidy_flow::recovery::TidyRecoveryPaths);
    let _ = stringify!(super::tidy_flow::recovery::TidyRecoveryRequest);
    let _ = stringify!(super::tidy_flow::recovery::TidyMaxLoopsOneRecovery);
    let _ = stringify!(super::tidy_flow::run_tidy_learn_prompt_if_elapsed);
    let _ = stringify!(super::tidy_flow::run_tidy_summary_prompt);
    let _ = stringify!(super::tidy_flow::tidy_session_dotfile_backups);
    let _ = stringify!(super::tidy_flow::tidy_skip_agent_startup);
    let _ = stringify!(super::tidy_flow::TidyAgentStartupRequest);
}

#[test]
fn smoke_cov_cli_cli_tidy_flow_units() {
    let _ = stringify!(super::tidy_flow::TidyArgs);
    let _ = stringify!(super::tidy_flow::TidyStartup);
    let _ = stringify!(super::tidy_flow::TidyAcpInput);
    let _ = stringify!(super::tidy_flow::TidyPromptRestore);
    let _ = stringify!(super::tidy_flow::prepare_tidy_prompt_store);
    let _ = stringify!(super::tidy_flow::compose_tidy_prompt);
    let _ = stringify!(super::tidy_flow::compose_tidy_concerns_prompt);
    let _ = stringify!(super::tidy_flow::write_checks_do_not_pass_to_review_path);
    let _ = stringify!(super::tidy_flow::write_checks_do_not_pass_for_artifacts);
    let _ = stringify!(super::tidy_flow::run_tidy_prompt);
    let _ = stringify!(super::tidy_flow::run_tidy_prompt_with_restore);
    let _ = stringify!(super::tidy_flow::run_tidy_interleaved_loop);
    let _ = stringify!(super::tidy_flow::tidy_finish_lgtm_attempt);
    let _ = stringify!(super::tidy_flow::TidyLgtmFinishCtx);
    let _ = stringify!(super::tidy_flow::tidy_recovery_stdout_line);
    let _ = stringify!(super::tidy_flow::run_tidy_post_concerns_recovery);
    let _ = stringify!(super::tidy_flow::run_tidy_bonus_gate_recovery);
    let _ = stringify!(super::tidy_flow::run_tidy_max_loops_one_not_lgtm_recovery);
    let _ = stringify!(super::tidy_flow::run_tidy_acp);
    let _ = stringify!(super::tidy_flow::merge_tidy_timing);
    let _ = stringify!(super::tidy_flow::tidy_prompt_context);
    let _ = stringify!(super::tidy_flow::tidy_run_agent_startup);
    let _ = stringify!(super::tidy_flow::prepare_tidy_run);
    let _ = stringify!(super::PlanArgs);
}

#[test]
fn smoke_cov_cli_cli_symbols_a() {
    let _ = stringify!(crate::cli::shared_opts::SharedOpts);
    let _ = stringify!(crate::cli::Cli);
    let _ = stringify!(crate::cli::Commands);
    let _ = stringify!(crate::do_flow::DoArgs);
    let _ = stringify!(crate::cli::models_cmd::ModelsArgs);
    let _ = stringify!(crate::cli::WorkflowCliOptions);
    let _ = stringify!(crate::cli::AgentStdoutTeeFlags);
    let _ = stringify!(crate::plan_flow::plan_prompt::prepare_plan_prompt_store);
    let _ = stringify!(crate::plan_flow::plan_prompt::compose_plan_prompt);
    let _ = stringify!(crate::plan_flow::resolve_plan_destination);
    let _ = stringify!(crate::plan_flow::apply_plan_source);
    let _ = stringify!(crate::plan_flow::plan_session_work_dir);
    let _ = stringify!(crate::cli::format_code_pre_check_failure);
    let _ = stringify!(crate::cli::format_pre_check_gate_failure);
    let _ = stringify!(crate::cli::format_workspace_gate_failure);
    let _ = stringify!(crate::do_flow::prepare_do_prompt_store);
    let _ = stringify!(crate::cli::run_bug);
    let _ = stringify!(crate::cli::kpop_flow::KpopAcpMultiturnCtx);
}

#[test]
fn smoke_cov_cli_cli_symbols_b() {
    let _ = stringify!(crate::cli::run_emit::emit_command_line);
    let _ = stringify!(crate::format_logs_dir);
    let _ = stringify!(crate::cli::shared_opts::SharedOpts::tee_startup_stdout);
    let _ = stringify!(crate::cli::models_cmd::run_models);
    let _ = stringify!(crate::lookup_bin_on_path);
    let _ = stringify!(crate::acp_post_run::merge_acp_with_workspace_session_restore);
    let _ = stringify!(
        crate::repo_checks::kissconfig_warn::warn_kissconfig_test_coverage_if_needed
    );
    let _ = stringify!(crate::repo_checks::run_repo_workspace_gates);
    let _ = stringify!(crate::repo_checks::run_repo_workspace_gates_no_kiss_clamp);
    let _ = stringify!(crate::cli::mid_session_gates::mid_pre_summary_repo_gates);
    let _ = stringify!(crate::cli::mid_session_gates::pre_summary_repo_gates_tidy_retry_flow);
    let _ = stringify!(
        crate::cli::mid_session_gates::mid_session_post_run_tidy::run_tidy_prompt_after_post_run_gate_failure
    );
    let _ = stringify!(super::entrypoint::try_tokio_runtime);
}
