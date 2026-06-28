//! Hidden ACP observability contract fixture for integration tests.

use std::path::{Path, PathBuf};

use serde_json::json;

use super::trace_line_write::WriteTraceLineCoalescedOpts;
use super::{PromptTraceWriter, TraceChunkCoalescer};

pub(crate) async fn open_contract_trace_writer(trace_path: &Path) -> PromptTraceWriter {
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(trace_path)
        .await
        .expect("open trace");
    PromptTraceWriter {
        file,
        who: crate::output::WHO_M.to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        iterable_closed_warned: false,
        upgrade_plan_warned: false,
        work_dir: PathBuf::new(),
        run_timing: None,
        session_id: String::new(),
        deferred_sink: None,
    }
}

pub(crate) async fn tee_coalesced_tool_execute(
    writer: &mut PromptTraceWriter,
    coalesce: &mut TraceChunkCoalescer,
) {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "contract_tool",
            "kind": "execute",
            "status": "pending",
            "rawInput": {"command": "echo contract"}
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "contract_tool",
            "kind": "execute",
            "status": "completed",
            "rawOutput": {"exitCode": 0, "stdout": "", "stderr": ""}
        }}
    });
    for update in [&start, &done] {
        let raw = update.to_string();
        super::trace_line_write::write_trace_line_coalesced(
            writer,
            coalesce,
            WriteTraceLineCoalescedOpts {
                parsed: Some(update),
                raw_line: &raw,
                tee_stdout: true,
            },
        )
        .await;
    }
}

/// Runs an ACP tool-call tee fixture; returns `(trace.jsonl text, stdout.log text)`.
#[doc(hidden)]
pub async fn contract_acp_tee_tool_fixture(
    trace_path: &Path,
    stdout_path: &Path,
) -> (String, String) {
    crate::output::set_stdout_log_path(Some(stdout_path.to_path_buf()));
    crate::output::init_stdout_style(false);
    let mut writer = open_contract_trace_writer(trace_path).await;
    let mut coalesce = TraceChunkCoalescer::default();
    tee_coalesced_tool_execute(&mut writer, &mut coalesce).await;
    drop(writer);
    crate::output::set_stdout_log_path(None);
    let trace = std::fs::read_to_string(trace_path).unwrap_or_default();
    let stdout = std::fs::read_to_string(stdout_path).unwrap_or_default();
    (trace, stdout)
}
