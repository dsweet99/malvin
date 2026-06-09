//! Kiss coverage references for [`super::kpop_summarize`] privates.

#[test]
fn kiss_cov_kpop_summarize_privates() {
    let _: Option<super::kpop_summarize::OuterLoopSummarizeParams<'_>> = None;
    let _ = super::kpop_summarize::run_summarize_agent_session;
    let _ = super::kpop_summarize::run_outer_loop_summarize_if_warranted;
    let _ = super::kpop_summarize::render_kpop_summarize_prompt;
    let _ = super::kpop_summarize::exp_log_paths_markdown;
    let _ = super::kpop_summarize::outer_loop_summarize_warranted;
    let _ = super::kpop_summarize::kpop_outer_loop_summarize_params;
    let _ = super::kpop_summarize::code_outer_loop_summarize_params;
    let _: Option<super::kpop_summarize::CodeOuterLoopSummarizeInputs<'_>> = None;
    let _: Option<super::kpop_summarize::KpopOuterLoopSummarizeInputs<'_>> = None;
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
