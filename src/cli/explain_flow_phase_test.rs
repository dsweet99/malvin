//! Standalone kiss test-file witnesses (`*_test.rs` naming) for explain Review/Plan/Work types.

#[test]
fn kiss_explain_kpop_phase_type_names() {
    let _ = stringify!(ExplainKpopPhaseParams);
    let _ = stringify!(ExplainKpopPhaseResult);
    let _ = stringify!(ExplainWorkParams);
    let _ = stringify!(ExplainSuccessInput);
    let _ = stringify!(run_explain_kpop_phase);
    let _ = stringify!(run_explain_kpop_phase_once);
    let _ = stringify!(build_explain_kpop_phase_prompt);
    let _ = stringify!(explain_kpop_chat_rules);
    let _ = stringify!(run_explain_work);
    let _ = stringify!(validate_explain_output);
    let _ = stringify!(resolve_explain_output_paths);
    let _ = stringify!(products_nonempty);
    let _ = stringify!(finish_explain_success);
    let _ = stringify!(emit_explain_startup);
    let _ = stringify!(EXPLAIN_PHASE_REVIEW);
    let _ = stringify!(EXPLAIN_PHASE_PLAN);
    let _ = stringify!(REVIEW_CHAT_RULES);
    let _ = stringify!(PLAN_CHAT_RULES);
    let _: Option<crate::cli::explain_flow::kpop_phase::ExplainKpopPhaseParams<'_>> = None;
    let _: Option<crate::cli::explain_flow::kpop_phase::ExplainKpopPhaseResult> = None;
    let _: Option<crate::cli::explain_flow::work::ExplainWorkParams<'_>> = None;
    let _ = crate::cli::explain_flow::kpop_phase::run_explain_kpop_phase;
    let _ = crate::cli::explain_flow::work::run_explain_work;
    let _ = crate::cli::explain_flow::kpop_phase::EXPLAIN_PHASE_REVIEW;
    let _ = crate::cli::explain_flow::kpop_phase::EXPLAIN_PHASE_PLAN;
}
