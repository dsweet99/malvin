use super::{AggregatedInitialPromptBuilder, WorkflowRenderContext, join_strata};

#[test]
fn join_strata_skips_empty_and_trims_trailing_whitespace() {
    assert_eq!(join_strata(["a\n", "", "  b  "]), "a\n\n  b");
}

#[test]
fn workflow_render_context_round_trip() {
    let mut ctx = WorkflowRenderContext::default();
    ctx.insert("plan_path", "/tmp/plan.md");
    assert_eq!(
        ctx.as_map().get("plan_path").map(String::as_str),
        Some("/tmp/plan.md")
    );
    assert_eq!(ctx.as_map().len(), 1);
}

#[test]
fn aggregated_builder_skips_empty_and_joins_labels() {
    let mut b = AggregatedInitialPromptBuilder::new();
    b.push_nonempty("a.md", String::new());
    b.push_nonempty("b.md", "B".into());
    b.push_nonempty("c.md", "C".into());
    let out = b.finish("who");
    assert_eq!(out.log_who, "who");
    assert_eq!(out.stdout_label, "b.md+c.md");
    assert!(out.body.contains('B'));
    assert!(out.body.contains('C'));
    assert!(!out.body.contains("a.md"));
}
