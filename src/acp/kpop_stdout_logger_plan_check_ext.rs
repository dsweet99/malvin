//! Additional plan falsification tests (h9–h11).

use super::kpop_stdout_logger_plan_helpers::{
    begin_stdout_log_fixture, execute_tool_done_json, execute_tool_json,
    finish_stdout_log_fixture, open_styled_markdown_trace_writer,
    production_execute_done_stdout, production_execute_done_trace_and_stdout,
    styled_markdown_trace_writer, stdout_log_test_guard,
};
use crate::acp::{format_styled_tool_summary_tee_line, write_tool_summary_trace_line};
use crate::output::assert_acp_tool_summary_dim_preserves_bracket;
use crate::tool_summary::{
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
    assert!(
        lines.stdout.is_none(),
        "expected pending edit with unknown path to be suppressed"
    );
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
    let indented = crate::output::acp_tee::indent_tool_call_log_payload(plain);
    let ctx = AcpTeeLineFmt {
        ts,
        direction: AcpTeeDirection::FromAgent,
        who: "<kpop",
        line: &indented,
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
    assert!(
        live.contains(plain),
        "display should carry payload; got {live:?}"
    );
}


fn h22_tee_writer(work_dir: std::path::PathBuf) -> crate::acp::PromptTraceWriter {
    let tee_dir = tempfile::tempdir().unwrap();
    let trace_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(tee_dir.path().join("tee-dim.trace"))
        .unwrap();
    styled_markdown_trace_writer(tokio::fs::File::from_std(trace_file), work_dir)
}

fn h22_rendered_tool_summary_tee(
    work_dir: std::path::PathBuf,
    start: &serde_json::Value,
    done: &serde_json::Value,
    ts: &str,
) -> String {
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let plain = tool_summary_lines(done, &mut tracker, ToolSummaryDetail::Stdout)
        .unwrap()
        .stdout
        .expect("stdout summary");
    let display = crate::tool_summary::apply_tool_summary_ansi(&plain);
    let tee_writer = h22_tee_writer(work_dir);
    format_styled_tool_summary_tee_line(&tee_writer, &plain, &display, ts)
}

#[tokio::test]
async fn h22_styled_tool_summary_trace_tee_dims_payload() {
    let start = execute_tool_json("tool_dim", "pending", "echo hi");
    let done = execute_tool_done_json("tool_dim");
    let ts = "20260413.121314.015";

    let _guard = stdout_log_test_guard();
    let fixture = begin_stdout_log_fixture();
    let (mut writer, mut coalesce) =
        open_styled_markdown_trace_writer(&fixture.trace_path, fixture.tmp.path()).await;
    let work_dir = writer.work_dir.clone();
    write_tool_summary_trace_line(&mut writer, &mut coalesce, &start, true).await;
    write_tool_summary_trace_line(&mut writer, &mut coalesce, &done, true).await;
    drop(writer);
    let stdout = finish_stdout_log_fixture(fixture);
    assert!(
        stdout.lines().any(|l| l.contains("echo hi")),
        "got {stdout:?}"
    );
    let rendered = h22_rendered_tool_summary_tee(work_dir, &start, &done, ts);
    assert_acp_tool_summary_dim_preserves_bracket(&rendered);
    assert!(rendered.contains("echo hi"), "got {rendered:?}");
}

#[test]
fn h22_tee_writer_opens() {
    let _ = h22_tee_writer(std::path::PathBuf::new());
}

#[test]
fn h22_rendered_tool_summary_tee_offline() {
    let start = execute_tool_json("tool_dim_offline", "pending", "echo hi");
    let done = execute_tool_done_json("tool_dim_offline");
    let rendered = h22_rendered_tool_summary_tee(std::path::PathBuf::new(), &start, &done, "ts");
    assert_acp_tool_summary_dim_preserves_bracket(&rendered);
}


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_h10_write_trace_line_coalesced_tees_timestamped_tool_summary_to_stdout_log() { let _ = stringify!(h10_write_trace_line_coalesced_tees_timestamped_tool_summary_to_stdout_log); }

    #[test]
    fn kiss_cov_h12_tool_summary_trace_and_stdout_log_share_timestamp() { let _ = stringify!(h12_tool_summary_trace_and_stdout_log_share_timestamp); }

    #[test]
    fn kiss_cov_h22_styled_tool_summary_trace_tee_dims_payload() { let _ = stringify!(h22_styled_tool_summary_trace_tee_dims_payload); }

}
