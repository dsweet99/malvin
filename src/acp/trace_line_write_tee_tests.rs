use super::*;
use crate::acp::trace_line_write::TraceTeeStdoutCtx;
use crate::acp::trace_plain_tee::print_tee_unprefixed_wrapped_line;
use crate::tool_summary::apply_tool_summary_ansi;

fn trace_writer() -> PromptTraceWriter {
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
        who: "<tee".to_string(),
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

fn with_stdout_log<F: FnOnce()>(color: bool, f: F) -> String {
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

fn rendered_tool_summary_tee_display(
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
fn thought_stdout_payload_indents_three_spaces() {
    let writer = trace_writer();
    let out = trace_stdout_tee_payload(
        "internal reasoning",
        Some(SessionUpdateChunkKind::Thought),
        &writer,
    );
    assert_eq!(out, "   internal reasoning");
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
    let dim = trace_tee_stdout_event(&writer, "thought", "ts", true);
    let bright = trace_tee_stdout_event(&writer, "plain", "ts", false);
    assert!(dim.dim_payload);
    assert!(!bright.dim_payload);
    assert_eq!(dim.who, "<tee");
}

#[test]
fn format_styled_tool_summary_tee_line_applies_dim_after_bracket() {
    let writer = trace_writer();
    let plain = "Run echo hi · 1ms · ✓";
    let display = apply_tool_summary_ansi(plain);
    let line = format_styled_tool_summary_tee_line(&writer, plain, &display, "20260413.121314.015");
    crate::output::assert_acp_tool_summary_dim_preserves_bracket(&line);
    crate::output::assert_tool_payload_uses_verb_styling(&line);
}

#[test]
fn styled_tool_payload_uses_verb_styling_without_brackets() {
    let writer = trace_writer();
    for plain in [
        "Run echo hi · 1ms · ✓",
        "Reading ./src/foo.rs…",
        "Read ./src/foo.rs · 1ms",
    ] {
        let display = apply_tool_summary_ansi(plain);
        let line = format_styled_tool_summary_tee_line(
            &writer,
            plain,
            &display,
            "20260413.121314.015",
        );
        crate::output::assert_tool_payload_uses_verb_styling(&line);
    }
}

#[test]
fn trace_tee_stdout_line_styled_tool_summary_dims_payload() {
    let plain = "Run echo hi · 1ms · ✓";
    let display = apply_tool_summary_ansi(plain);
    let ts = "20260413.121314.015";
    let log = with_stdout_log(true, || {
        let mut writer = trace_writer();
        trace_tee_stdout_line(
            &mut writer,
            plain,
            Some(&display),
            &TraceTeeStdoutCtx {
                tee_stdout: true,
                kind: None,
                ts,
            },
        );
        let rendered = rendered_tool_summary_tee_display(&writer, plain, &display, ts);
        crate::output::assert_acp_tool_summary_dim_preserves_bracket(&rendered);
    });
    assert!(log.contains("echo hi"), "got {log:?}");
}

#[test]
fn trace_tee_stdout_line_plain_lines_writes_timestamped_log() {
    let log = with_stdout_log(false, || {
        let mut writer = trace_writer();
        writer.plain_lines = true;
        trace_tee_stdout_line(
            &mut writer,
            "plain tee line",
            None,
            &TraceTeeStdoutCtx {
                tee_stdout: true,
                kind: None,
                ts: "20260413.121314.015",
            },
        );
    });
    assert!(log.contains("plain tee line"), "got {log:?}");
    assert!(crate::output::is_log_timestamp_token(
        log.split_whitespace().next().unwrap_or("")
    ));
}
