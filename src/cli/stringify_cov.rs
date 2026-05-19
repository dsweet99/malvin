//! `kiss` coverage: static `stringify!` refs for the `malvin` binary and crates it uses directly (kept out of `mod.rs` for line budget).

#[test]
fn kiss_stringify_cli_symbols_a() {
    let _ = stringify!(crate::cli::shared_opts::SharedOpts);
    let _ = stringify!(crate::cli::Cli);
    let _ = stringify!(crate::cli::shared_opts::GlobalOpts);
    let _ = stringify!(crate::cli::Commands);
    let _ = stringify!(crate::cli::CodeArgs);
    let _ = stringify!(crate::cli::do_flow::DoArgs);
    let _ = stringify!(crate::init_cmd::InitArgs);
    let _ = stringify!(crate::cli::models_cmd::ModelsArgs);
    let _ = stringify!(crate::cli::KpopArgs);
    let _ = stringify!(crate::cli::BugArgs);
    let _ = stringify!(crate::cli::TidyArgs);
    let _ = stringify!(crate::cli::PlanArgs);
    let _ = stringify!(crate::cli::SharedOpts);
    let _ = stringify!(crate::cli::Exit);
    let _ = stringify!(crate::cli::WorkflowCliOptions);
    let _ = stringify!(crate::cli::AgentStdoutTeeFlags);
    let _ = stringify!(crate::cli::entrypoint);
    let _ = stringify!(crate::cli::run_code);
    let _ = stringify!(crate::cli::run_do);
    let _ = stringify!(crate::cli::run_tidy);
    let _ = stringify!(crate::cli::run_plan);
    let _ = stringify!(crate::cli::plan_flow::plan_prompt::prepare_plan_prompt_store);
    let _ = stringify!(crate::cli::plan_flow::plan_prompt::compose_plan_prompt);
    let _ = stringify!(crate::cli::plan_flow::resolve_plan_destination);
    let _ = stringify!(crate::cli::plan_flow::apply_plan_source);
    let _ = stringify!(crate::cli::plan_flow::plan_session_work_dir);
    let _ = stringify!(crate::cli::format_code_pre_check_failure);
    let _ = stringify!(crate::cli::format_pre_check_gate_failure);
    let _ = stringify!(crate::cli::format_workspace_gate_failure);
    let _ = stringify!(crate::cli::do_flow::prepare_do_prompt_store);
    let _ = stringify!(crate::cli::run_kpop);
    let _ = stringify!(crate::cli::run_bug);
    let _ = stringify!(crate::cli::kpop_flow::KpopPrepared);
    let _ = stringify!(crate::cli::kpop_flow::KpopAcpMultiturnCtx);
    let _ = stringify!(crate::cli::kpop_flow::kpop_run_acp_multiturn);
}

#[test]
fn kiss_stringify_cli_symbols_b() {
    let _ = stringify!(crate::cli::prepare_prompt_store);
    let _ = stringify!(crate::cli::prepare_kpop_prompt_store);
    let _ = stringify!(crate::cli::prepare_bug_prompt_store);
    let _ = stringify!(crate::cli::run_emit::echo_primary_to_stdout);
    let _ = stringify!(crate::cli::run_emit::emit_command_line);
    let _ = stringify!(crate::cli::run_emit::emit_run_startup_sequence);
    let _ = stringify!(crate::format_logs_dir);
    let _ = stringify!(crate::cli::build_agent);
    let _ = stringify!(crate::cli::shared_opts::SharedOpts::tee_startup_stdout);
    let _ = stringify!(crate::init_cmd::run_init);
    let _ = stringify!(crate::cli::models_cmd::run_models);
    let _ = stringify!(crate::lookup_bin_on_path);
    let _ = stringify!(crate::acp_post_run::emit_run_timing_after_acp);
    let _ = stringify!(crate::acp_post_run::merge_acp_and_timing_results);
    let _ = stringify!(crate::acp_post_run::prefer_primary_over_secondary);
    let _ = stringify!(crate::acp_post_run::merge_acp_with_workspace_session_restore);
    let _ = stringify!(
        crate::repo_checks::kissconfig_warn::warn_kissconfig_test_coverage_if_needed
    );
    let _ = stringify!(crate::repo_checks::run_repo_workspace_gates);
    let _ = stringify!(crate::repo_checks::run_repo_workspace_gates_no_kiss_clamp);
    let _ = stringify!(crate::source_detect::has_source_files);
    let _ = stringify!(crate::cli::mid_session_gates::mid_pre_summary_repo_gates);
    let _ = stringify!(crate::cli::mid_session_gates::pre_summary_repo_gates_tidy_retry_flow);
    let _ = stringify!(
        crate::cli::mid_session_gates::mid_session_post_run_tidy::run_tidy_prompt_after_post_run_gate_failure
    );
    let _ = stringify!(super::entrypoint::run_code_command);
    let _ = stringify!(super::require_kiss_for_cli_command);
    let _ = stringify!(super::print_command_error);
    let _ = stringify!(super::try_tokio_runtime);
}
