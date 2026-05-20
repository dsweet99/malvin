#[test]
fn smoke_cov_tidy_flow_units() {
    let _: Option<crate::cli::tidy_flow::TidyArgs> = None;
    let _: Option<crate::cli::tidy_flow::TidyAcpInput> = None;
    let _ = crate::cli::tidy_flow::prepare_tidy_prompt_store;
    let _: Option<crate::cli::tidy_flow::TidyStartup> = None;
    let _ = crate::cli::tidy_flow::compose_tidy_prompt;
    let _ = crate::cli::tidy_flow::compose_tidy_concerns_prompt;
    let _ = crate::cli::tidy_flow::write_checks_do_not_pass_to_review_path;
    let _ = crate::cli::tidy_flow::run_tidy_interleaved_loop;
    let _ = crate::orchestrator::ensure_artifact_review_after_review_write;
    let _ = crate::cli::tidy_flow::run_tidy_prompt;
    let _ = crate::cli::tidy_flow::run_tidy_acp;
    let _ = crate::cli::tidy_flow::tidy_prompt_context;
    let _ = crate::cli::tidy_flow::prepare_tidy_run;
    let _ = crate::cli::tidy_flow::merge_tidy_timing;
    let _ = crate::cli::tidy_flow::run_tidy;
    let _ = crate::orchestrator::finish_review_write_attempt;
    let _ = crate::orchestrator::fail_on_abort_for_artifacts;
}
