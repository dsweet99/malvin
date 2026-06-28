use super::{join_strata, PromptStratum, WorkflowRenderContext};

#[test]
fn join_strata_skips_empty_and_trims_trailing_whitespace() {
    assert_eq!(join_strata(["a\n", "", "  b  "]), "a\n\n  b");
}

#[test]
fn workflow_render_context_round_trip() {
    let mut ctx = WorkflowRenderContext::default();
    ctx.insert("plan_path", "/tmp/plan.md");
    assert_eq!(ctx.as_map().get("plan_path").map(String::as_str), Some("/tmp/plan.md"));
    let map = ctx.into_map();
    assert_eq!(map.len(), 1);
}

#[test]
fn prompt_stratum_variants_exist() {
    let _ = (
        PromptStratum::EmbeddedTemplate,
        PromptStratum::PlaceholderContext,
        PromptStratum::WorkflowHeader,
        PromptStratum::UserRequest,
        PromptStratum::GateLoopBlock,
        PromptStratum::MiniConstraints,
    );
}
