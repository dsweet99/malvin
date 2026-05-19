#[test]
fn kiss_stringify_tidy_flow_units() {
    let _ = stringify!(crate::cli::tidy_flow::TidyArgs);
    let _ = stringify!(crate::cli::tidy_flow::TidyAcpInput);
    let _ = stringify!(crate::cli::tidy_flow::prepare_tidy_prompt_store);
    let _ = stringify!(crate::cli::tidy_flow::TidyStartup);
    let _ = stringify!(crate::cli::tidy_flow::compose_tidy_prompt);
    let _ = stringify!(crate::cli::tidy_flow::compose_tidy_concerns_prompt);
    let _ = stringify!(crate::cli::tidy_flow::write_checks_do_not_pass_to_review_path);
    let _ = stringify!(crate::cli::tidy_flow::run_tidy_interleaved_loop);
    let _ = stringify!(crate::orchestrator::ensure_artifact_review_after_review_write);
    let _ = stringify!(crate::cli::tidy_flow::run_tidy_prompt);
    let _ = stringify!(crate::cli::tidy_flow::run_tidy_acp);
    let _ = stringify!(crate::cli::tidy_flow::tidy_prompt_context);
    let _ = stringify!(crate::cli::tidy_flow::prepare_tidy_run);
    let _ = stringify!(crate::cli::tidy_flow::merge_tidy_timing);
    let _ = stringify!(crate::cli::tidy_flow::run_tidy);
    let _ = stringify!(crate::orchestrator::finish_review_write_attempt);
    let _ = stringify!(crate::orchestrator::fail_on_abort_for_artifacts);
}
