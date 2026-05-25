use crate::cursor_store::ToolCallArgs;
use crate::deferred_log::types::ToolDrainMeta;
use crate::tool_summary::ToolSummaryTracker;
use crate::tool_summary::human_done_line;
use crate::tool_summary::{tool_summary_stdout_display, ParsedToolUpdate, TOOL_PHASE_DONE};

pub fn enriched_tool_plain(
    meta: &ToolDrainMeta,
    args: Option<&ToolCallArgs>,
    work_dir: &std::path::Path,
    emit_stdout_markdown: bool,
) -> (String, String) {
    let Some(args) = args.filter(|a| a.path.is_some()) else {
        return styled_tool_payload(&meta.fallback_plain, emit_stdout_markdown);
    };
    let parsed = synthetic_tool_done(meta, args);
    let mut tracker = ToolSummaryTracker::default();
    tracker.set_work_dir(work_dir.to_path_buf());
    let plain = human_done_line(&parsed, &tracker, meta.kind.as_str(), meta.elapsed)
        .unwrap_or_else(|| meta.fallback_plain.clone());
    styled_tool_payload(&plain, emit_stdout_markdown)
}

pub(crate) fn styled_tool_payload(plain: &str, emit_stdout_markdown: bool) -> (String, String) {
    let plain = if emit_stdout_markdown {
        format!("[{plain}]")
    } else {
        plain.to_string()
    };
    let display = tool_summary_stdout_display(&plain);
    (plain, display)
}

pub(crate) fn synthetic_tool_done(meta: &ToolDrainMeta, args: &ToolCallArgs) -> ParsedToolUpdate {
    ParsedToolUpdate {
        phase: TOOL_PHASE_DONE,
        id: meta.tool_call_id.clone(),
        kind: meta.kind.clone(),
        title: String::new(),
        status: Some("completed".to_string()),
        command: None,
        input_path: args.path.clone(),
        input_line_range: args.line_range,
        search_query: None,
        raw_output: meta.raw_output.clone(),
    }
}
