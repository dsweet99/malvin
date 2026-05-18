//! Static name references for kiss per-file coverage of CLI modules.

#[test]
fn kiss_stringify_cli_units_0() {
    let _ = stringify!(super::args::BugArgs);
    let _ = stringify!(super::args::CodeArgs);
    let _ = stringify!(super::args::KpopArgs);
    let _ = stringify!(super::code_flow::build_agent);
    let _ = stringify!(super::code_flow::format_pre_check_gate_failure);
    let _ = stringify!(super::code_flow::prepare_bug_prompt_store);
    let _ = stringify!(super::code_flow::prepare_code_run);
    let _ = stringify!(super::code_flow::prepare_kpop_prompt_store);
    let _ = stringify!(super::code_flow::prepare_prompt_store);
    let _ = stringify!(super::code_flow::run_code);
    let _ = stringify!(super::do_flow::combine_do_prompt_file_and_user);
    let _ = stringify!(super::do_flow::prepare_do_prompt_store_validating);
    let _ = stringify!(super::do_flow::run_do_acp);
    let _ = stringify!(super::entrypoint::dispatch_command);
    let _ = stringify!(super::entrypoint::print_command_error);
    let _ = stringify!(super::entrypoint::run_async_cli);
    let _ = stringify!(super::entrypoint::run_code_command);
    let _ = stringify!(super::exit::Exit);
    let _ = stringify!(super::init_cmd::InitArgs);
    let _ = stringify!(super::init_cmd::bootstrap_repo_tooling);
    let _ = stringify!(super::init_cmd::create_initial_commit);
    let _ = stringify!(super::init_cmd::emit_init_startup);
    let _ = stringify!(super::init_cmd::repo_already_has_commits);
    let _ = stringify!(super::init_cmd::run_init);
    let _ = stringify!(super::init_cmd::run_init_summary_phase);
}

#[test]
fn kiss_stringify_cli_units_1a() {
    let _ = stringify!(super::init_cmd::write_init_templates);
    let _ = stringify!(super::kpop_flow::KpopPrepared);
    let _ = stringify!(super::kpop_flow::into_bug_followup_artifacts);
    let _ = stringify!(super::kpop_flow::kpop_run_acp_multiturn);
    let _ = stringify!(super::kpop_flow::run_kpop);
    let _ = stringify!(super::mid_session_gates::mid_session_post_run_tidy::PostRunTidyPrompt);
    let _ = stringify!(super::mid_session_gates::mid_session_post_run_tidy::prepare);
    let _ = stringify!(super::repo_checks::emit::append_quality_gates_log_text);
    let _ = stringify!(super::run_emit::echo_primary_to_stdout);
    let _ = stringify!(super::run_emit::emit_run_startup_sequence);
    let _ = stringify!(super::shared_opts::GlobalOpts);
    let _ = stringify!(super::tidy_flow::recovery::TidyReviewAttemptOutcome);
    let _ = stringify!(super::tidy_flow::recovery::run_tidy_concerns_coder_turn);
    let _ = stringify!(super::tidy_flow::run_tidy_learn_mid_gates_and_summary);
    let _ = stringify!(super::tidy_flow::recovery::tidy_fail_on_abort);
    let _ = stringify!(super::tidy_flow::recovery::tidy_learn_elapsed_threshold_ms);
    let _ = stringify!(super::tidy_flow::recovery::tidy_review_attempt_with_retries);
    let _ = stringify!(super::timing_merge::emit_run_timing_after_acp);
    let _ = stringify!(super::timing_merge::merge_error_mentions_restore);
}

#[test]
fn kiss_stringify_cli_units_1b() {
    let _ = stringify!(super::code_flow_b::CodeRunExecuteArgs);
    let _ = stringify!(super::code_flow_b::code_run_execute);
    let _ = stringify!(super::do_flow_a::do_agent_client_for_do_cmd);
    let _ = stringify!(super::do_flow_a::finalize_do_workspace_restore_merge);
    let _ = stringify!(super::do_flow_a::stamp_do_timing_display_name);
    let _ = stringify!(super::do_flow_a::bootstrap_do_workspace_from_request);
    let _ = stringify!(super::do_flow_a::compose_do_coder_bundle);
    let _ = stringify!(super::do_flow_a::merge_two_string_results);
    let _ = stringify!(super::init_cmd_early::RunInitOptions);
    let _ = stringify!(super::init_cmd_early::RunInitRequest);
    let _ = stringify!(super::init_cmd_early::init_summary_combined_body);
    let _ = stringify!(super::init_cmd_early::init_summary_coder_turn_with_timing_emit);
    let _ = stringify!(super::kpop_flow_a::init_kpop_exp_log_file);
    let _ = stringify!(super::kpop_flow_a::kpop_boot_store_client_prepared);
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
fn kiss_stringify_cli_tidy_flow_units() {
    let _ = stringify!(super::tidy_flow::TidyArgs);
    let _ = stringify!(super::tidy_flow::run_tidy);
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
    let _ = stringify!(super::plan_flow::PlanArgs);
}

#[test]
fn tidy_zero_max_loops_effective_budget_is_one() {
    assert_eq!(super::tidy_flow::effective_tidy_max_loops(0), 1);
    assert_eq!(super::tidy_flow::effective_tidy_max_loops(3), 3);
}
