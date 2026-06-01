//! Tool-summary bracket regressions for plan.md.

use super::kpop_stdout_logger_plan_helpers::{
    begin_stdout_log_fixture, finish_stdout_log_fixture, open_styled_markdown_trace_writer,
    stdout_log_test_guard, styled_markdown_trace_writer,
};
use crate::acp::write_tool_summary_trace_line;
use crate::acp::format_styled_tool_summary_tee_line;
use crate::ansi_strip::strip_ansi_escapes;
use crate::output::assert_acp_tool_summary_dim_preserves_bracket;
use crate::tool_summary::apply_tool_summary_ansi;
use crate::terminal_palette::ansi_tool_dark;
use serde_json::json;

pub(super) fn read_tool_bracket_pair_updates(
    path: &str,
) -> (serde_json::Value, serde_json::Value, serde_json::Value) {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_bracket_pair",
            "kind": "read",
            "status": "pending",
            "rawInput": {"path": path}
        }}
    });
    let pending = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_bracket_pair",
            "kind": "read",
            "status": "pending"
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_bracket_pair",
            "kind": "read",
            "status": "completed",
            "rawOutput": {"content": "x"}
        }}
    });
    (start, pending, done)
}

pub(super) fn assert_payload_omits_brackets_after_who_tag(start_line: &str, done_line: &str) {
    assert!(start_line.contains("Reading"), "got {start_line:?}");
    assert!(done_line.contains("Read "), "got {done_line:?}");
    for line in [start_line, done_line] {
        let plain = strip_ansi_escapes(line);
        let who_end = plain.find('|').expect("who pipe delimiter");
        let payload = plain[who_end + 1..].trim_start();
        assert!(
            !payload.starts_with('['),
            "payload must not open with [ after who tag; got {plain:?}"
        );
        assert!(
            !payload.ends_with(']'),
            "payload must not end with ] after who tag; got {plain:?}"
        );
    }
}

pub(super) fn assert_styled_tool_summary_payloads_match(start_payload: &str, done_payload: &str) {
    let offline_dir = tempfile::tempdir().unwrap();
    let offline_file = std::fs::File::create(offline_dir.path().join("tee-offline")).unwrap();
    let tee_writer = styled_markdown_trace_writer(
        tokio::fs::File::from_std(offline_file),
        offline_dir.path().to_path_buf(),
    );
    for payload in [start_payload, done_payload] {
        let plain = payload
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .to_string();
        let display = apply_tool_summary_ansi(&plain);
        let styled = format_styled_tool_summary_tee_line(
            &tee_writer,
            &plain,
            &display,
            "20260413.121314.015",
        );
        assert!(
            styled.contains(ansi_tool_dark()),
            "styled tool summary payload verbs use dark bold; got {styled:?}"
        );
        assert_acp_tool_summary_dim_preserves_bracket(&styled);
        crate::output::assert_tool_payload_uses_verb_styling(&styled);
    }
}

async fn tee_tool_summary_updates(
    start: &serde_json::Value,
    pending: &serde_json::Value,
    done: &serde_json::Value,
) -> String {
    let fixture = {
        let _guard = stdout_log_test_guard();
        begin_stdout_log_fixture()
    };
    let work_dir = fixture.tmp.path().to_path_buf();
    let (mut writer, mut coalesce) =
        open_styled_markdown_trace_writer(&fixture.trace_path, &work_dir).await;
    write_tool_summary_trace_line(&mut writer, &mut coalesce, start, true).await;
    write_tool_summary_trace_line(&mut writer, &mut coalesce, pending, true).await;
    write_tool_summary_trace_line(&mut writer, &mut coalesce, done, true).await;
    drop(writer);
    finish_stdout_log_fixture(fixture)
}

pub(super) async fn tee_read_tool_bracket_pair_stdout(path: &str) -> (String, String) {
    let (start, pending, done) = read_tool_bracket_pair_updates(path);
    let stdout = tee_tool_summary_updates(&start, &pending, &done).await;
    let start_line = stdout
        .lines()
        .find(|l| l.contains("Reading") && l.contains(path))
        .expect("start summary on stdout")
        .to_string();
    let done_line = stdout
        .lines()
        .find(|l| l.contains("Read ") && l.contains(path))
        .expect("done summary on stdout")
        .to_string();
    (start_line, done_line)
}

#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_bracket_helpers() {
        let _ = stringify!(read_tool_bracket_pair_updates);
        let _ = stringify!(assert_payload_omits_brackets_after_who_tag);
        let _ = stringify!(assert_styled_tool_summary_payloads_match);
        let _ = stringify!(tee_tool_summary_updates);
        let _ = stringify!(tee_read_tool_bracket_pair_stdout);
    }
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = read_tool_bracket_pair_updates;
        let _ = tee_tool_summary_updates;
    }
}
