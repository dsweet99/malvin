use super::*;
use crate::acp::trace_line_write::TraceTeeStdoutCtx;
use crate::acp::trace_plain_tee::print_tee_unprefixed_wrapped_line;

pub(super) fn trace_writer() -> PromptTraceWriter {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace.log");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .unwrap();
    PromptTraceWriter {
        file: tokio::fs::File::from_std(file),
        who: crate::output::WHO_M.to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
        iterable_closed_warned: false,
        upgrade_plan_warned: false,
        work_dir: dir.path().to_path_buf(),
        run_timing: None,
        session_id: String::new(),
        deferred_sink: None,
    }
}

pub(super) fn with_stdout_log<F: FnOnce()>(color: bool, f: F) -> String {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(path.clone()));
    crate::output::init_stdout_style(color);
    f();
    crate::output::set_stdout_log_path(None);
    std::fs::read_to_string(path).unwrap()
}

pub(super) fn rendered_tool_summary_tee_display(
    writer: &PromptTraceWriter,
    plain: &str,
    display: &str,
    ts: &str,
) -> String {
    format_styled_tool_summary_tee_line(writer, &format!("[{plain}]"), display, ts)
}

#[test]
fn format_trace_display_line_brackets_thoughts() {
    assert_eq!(
        format_trace_display_line("reasoning", Some(SessionUpdateChunkKind::Thought)),
        "[reasoning]"
    );
    assert_eq!(format_trace_display_line("Run x", None), "Run x");
}

#[test]
fn thought_stdout_payload_indents_one_space() {
    let writer = trace_writer();
    let out = trace_stdout_tee_payload(
        "internal reasoning",
        Some(SessionUpdateChunkKind::Thought),
        &writer,
    );
    assert_eq!(out, "internal reasoning");
}

#[test]
fn tool_call_log_payload_indents_one_space() {
    assert_eq!(
        crate::output::acp_tee::indent_tool_call_log_payload("Run echo hi · 1ms · ✓"),
        " Run echo hi · 1ms · ✓"
    );
}

#[test]
fn trace_tee_stdout_line_noop_when_tee_disabled() {
    let mut writer = trace_writer();
    trace_tee_stdout_line(
        &mut writer,
        "line",
        Some("display"),
        &TraceTeeStdoutCtx {
            tee_stdout: false,
            kind: None,
            ts: "20260413.121314.015",
        },
    );
}

#[test]
fn print_tee_unprefixed_wrapped_line_writes_timestamped_raw_log() {
    let log = with_stdout_log(false, || {
        print_tee_unprefixed_wrapped_line("direct plain tee", "20260413.121314.015");
    });
    assert!(log.contains("direct plain tee"), "got {log:?}");
    assert!(crate::output::is_log_timestamp_token(
        log.split_whitespace().next().unwrap_or("")
    ));
}

#[test]
fn trace_tee_stdout_line_raw_output_writes_unprefixed_log() {
    let log = with_stdout_log(false, || {
        let mut writer = trace_writer();
        writer.raw_output = true;
        trace_tee_stdout_line(
            &mut writer,
            "raw line",
            Some("display"),
            &TraceTeeStdoutCtx {
                tee_stdout: true,
                kind: None,
                ts: "20260413.121314.015",
            },
        );
    });
    assert!(log.contains("raw line"), "got {log:?}");
}

#[test]
fn tool_summary_tee_event_sets_dim_payload() {
    let writer = trace_writer();
    let ev = trace_tee_tool_summary_stdout_event(&writer, "Run echo hi · 1ms · ✓", "ts");
    assert!(ev.dim_payload);
}

#[test]
fn trace_tee_stdout_event_respects_dim_payload_flag() {
    let writer = trace_writer();
    let dim = trace_tee_stdout_event(
        &writer,
        super::TeeStdoutEmit {
            line: "thought",
            ts: "ts",
            dim_payload: true,
            who: crate::output::WHO_B,
        },
    );
    let bright = trace_tee_stdout_event(
        &writer,
        super::TeeStdoutEmit {
            line: "plain",
            ts: "ts",
            dim_payload: false,
            who: crate::output::WHO_M,
        },
    );
    assert!(dim.dim_payload);
    assert!(!bright.dim_payload);
    assert_eq!(dim.who, crate::output::WHO_B);
}

#[test]
fn from_agent_who_maps_thought_message_and_tool_roles() {
    let thought_ctx = TraceTeeStdoutCtx {
        ts: "ts",
        kind: Some(crate::acp::SessionUpdateChunkKind::Thought),
        tee_stdout: true,
    };
    assert_eq!(from_agent_who(&thought_ctx, false), crate::output::WHO_B);
    let message_ctx = TraceTeeStdoutCtx {
        kind: None,
        ..thought_ctx
    };
    assert_eq!(from_agent_who(&message_ctx, false), crate::output::WHO_M);
    assert_eq!(from_agent_who(&message_ctx, true), crate::output::WHO_T);
}

#[path = "trace_line_write_tee_tests_b.rs"]
mod trace_line_write_tee_tests_b;
