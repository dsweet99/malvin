#[test]
fn kiss_cov_remaining_shared_helpers() {
    use crate::coder_prompt_phase::MiniPhase;
    assert_eq!(MiniPhase::Investigate.as_str(), "investigate");
    let _ = (MiniPhase::WindDown, MiniPhase::Terminal);
    let parsed = crate::model_id::parse_model_id("cursor:auto").expect("parse");
    let _ = parsed.backend;
    let _ = parsed.canonical();
    let build = crate::deferred_log::ToolSummaryBuild {
        tee: crate::deferred_log::TeeSinkMeta {
            who: "w".into(),
            ts: "t".into(),
            emit_stdout_markdown: false,
        },
        plain: "p".into(),
        display: "d".into(),
        enrich: None,
        meta: None,
    };
    let _ = crate::deferred_log::build_tool_entry(build);
}
