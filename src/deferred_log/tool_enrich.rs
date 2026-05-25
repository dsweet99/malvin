use serde_json::Value;

use crate::tool_summary::parse_tool_update;
use crate::tool_summary::{ToolSummaryTracker, TOOL_PHASE_DONE};

use super::types::{EnrichKey, ToolDrainMeta};

pub(crate) fn tool_drain_enrich_fields(
    parsed: &Value,
    tracker: &ToolSummaryTracker,
    plain: &str,
) -> (Option<EnrichKey>, Option<ToolDrainMeta>) {
    let Some(update) = parse_tool_update(parsed) else {
        return (None, None);
    };
    if update.phase != TOOL_PHASE_DONE {
        return (None, None);
    }
    let has_wire_path = update.input_path.is_some()
        || tracker
            .record(&update.id)
            .and_then(|r| r.input_path.as_ref())
            .is_some();
    if has_wire_path {
        return (None, None);
    }
    if !matches!(update.kind.as_str(), "read" | "edit") {
        return (None, None);
    }
    let elapsed = tracker
        .record(&update.id)
        .map(|r| r.started.elapsed())
        .unwrap_or_default();
    let fallback = plain
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(plain)
        .to_string();
    (
        Some(EnrichKey {
            tool_call_id: update.id.clone(),
            kind: update.kind.clone(),
        }),
        Some(ToolDrainMeta {
            tool_call_id: update.id,
            kind: update.kind,
            elapsed,
            raw_output: update.raw_output,
            fallback_plain: fallback,
        }),
    )
}
