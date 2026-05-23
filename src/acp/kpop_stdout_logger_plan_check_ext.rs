//! Additional plan falsification tests (h9–h11).

use super::kpop_stdout_logger_plan_helpers::{
    production_execute_done_stdout, production_execute_done_trace_and_stdout,
};
use crate::acp::tool_summary::{
    ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines, tool_summary_stdout_display,
};
use crate::ansi_strip::strip_ansi_escapes;
use crate::output::{
    AcpTeeDirection, AcpTeeLineFmt, AcpTeeStdoutEvent, acp_tee_display_line, acp_tee_log_line,
    is_log_timestamp_token, print_stdout_acp_tool_summary_tee, set_stdout_log_path,
};
use serde_json::json;

#[test]
fn h9_edit_without_path_suppresses_pending_stdout() {
    let pending = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_edit_file",
            "kind": "edit",
            "status": "pending",
            "title": "Edit"
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&pending, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    assert!(lines.stdout.is_none(), "expected pending edit with unknown path to be suppressed");
}

#[tokio::test]
async fn h10_write_trace_line_coalesced_tees_timestamped_tool_summary_to_stdout_log() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let stdout = production_execute_done_stdout().await;
    let first = stdout.lines().next().unwrap_or("");
    assert!(
        is_log_timestamp_token(first.split_whitespace().next().unwrap_or("")),
        "production tool-summary tee should be timestamped; got {stdout:?}"
    );
    assert!(
        first.contains("Run"),
        "production path should tee human execute summary; got {stdout:?}"
    );
}

#[tokio::test]
async fn h12_tool_summary_trace_and_stdout_log_share_timestamp() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (trace, stdout) = production_execute_done_trace_and_stdout().await;
    let stdout_line = stdout
        .lines()
        .find(|l| l.contains("Run "))
        .expect("human execute summary on stdout.log");
    let trace_line = trace
        .lines()
        .find(|l| l.contains("[tool]") && l.contains("done"))
        .expect("tool done line on trace");
    let stdout_ts = stdout_line.split_whitespace().next().unwrap_or("");
    let trace_ts = trace_line.split_whitespace().next().unwrap_or("");
    assert!(
        is_log_timestamp_token(stdout_ts),
        "stdout timestamp token; got {stdout_line:?}"
    );
    assert!(
        is_log_timestamp_token(trace_ts),
        "trace timestamp token; got {trace_line:?}"
    );
    assert_eq!(
        stdout_ts, trace_ts,
        "trace and stdout.log must share one timestamp per tool-summary tee event; stdout={stdout_ts:?} trace={trace_ts:?}"
    );
}

#[test]
fn h13_read_dotted_title_without_path_suppresses_pending_stdout() {
    let pending = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_read_version",
            "kind": "read",
            "status": "pending",
            "title": "Read v2.0"
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&pending, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    assert!(
        lines.stdout.is_none(),
        "pending read with dotted non-path title must stay suppressed (plan Q2); got {:?}",
        lines.stdout
    );
}

#[test]
fn h11_tool_summary_tee_log_matches_stripped_display_when_color_on() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    crate::output::init_stdout_style(true);

    let plain = "Run cargo test · 1.0s · ✓";
    let display = tool_summary_stdout_display(plain);
    let ts = "20260413.121314.015";
    print_stdout_acp_tool_summary_tee(
        &AcpTeeStdoutEvent {
            direction: AcpTeeDirection::FromAgent,
            who: "<kpop",
            line: plain,
            ts,
            emit_stdout_markdown: false,
            dim_payload: false,
        },
        &display,
    );
    set_stdout_log_path(None);

    let log_line = std::fs::read_to_string(path).unwrap();
    let log_line = log_line.trim_end();
    let ctx = AcpTeeLineFmt {
        ts,
        direction: AcpTeeDirection::FromAgent,
        who: "<kpop",
        line: plain,
        dim_payload: false,
    };
    assert_eq!(log_line, acp_tee_log_line(&ctx));
    let live = strip_ansi_escapes(&acp_tee_display_line(&AcpTeeLineFmt {
        ts,
        direction: AcpTeeDirection::FromAgent,
        who: "<kpop",
        line: &display,
        dim_payload: false,
    }));
    assert!(
        !is_log_timestamp_token(live.split_whitespace().next().unwrap_or("")),
        "live tee display must omit wall-clock prefix; got {live:?}"
    );
    assert!(live.contains(plain), "display should carry payload; got {live:?}");
}
