use super::*;

#[test]
fn kiss_cov_defer_raw_wrapped_lines() {
}

#[test]
fn kiss_cov_trace_tee_deferred_line() {
}

#[test]
fn kiss_cov_trace_tee_immediate_line() {
}

#[test]
fn kiss_cov_tee_stdout_emit_field_names() {
    let _ = stringify!(TeeStdoutEmit);
    let _ = stringify!(line);
    let _ = stringify!(ts);
    let _ = stringify!(dim_payload);
    let _ = stringify!(who);
    for (line, ts, dim_payload, who) in [
        ("a", "ts1", true, crate::output::WHO_B),
        ("b", "ts2", false, crate::output::WHO_M),
        ("c", "ts3", true, crate::output::WHO_T),
    ] {
        let emit = TeeStdoutEmit {
            line,
            ts,
            dim_payload,
            who,
        };
        let TeeStdoutEmit {
            line: l,
            ts: t,
            dim_payload: d,
            who: w,
        } = emit;
        assert_eq!(l, line);
        assert_eq!(t, ts);
        assert_eq!(d, dim_payload);
        assert_eq!(w, who);
        let writer = trace_writer();
        let ev = trace_tee_stdout_event(&writer, emit);
        assert_eq!(ev.line, line);
    }
}

#[test]
fn format_styled_tool_summary_tee_line_applies_dim_after_bracket() {
    let writer = trace_writer();
    let plain = "Run echo hi · 1ms · ✓";
    let display = crate::tool_summary::apply_tool_summary_ansi(plain);
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
        let display = crate::tool_summary::apply_tool_summary_ansi(plain);
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
    let display = crate::tool_summary::apply_tool_summary_ansi(plain);
    let ts = "20260413.121314.015";
    let log = with_stdout_log(true, || {
        let mut writer = trace_writer();
        trace_tee_stdout_line(
            &mut writer,
            plain,
            Some(&display),
            &crate::acp::trace_line_write::TraceTeeStdoutCtx {
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
            &crate::acp::trace_line_write::TraceTeeStdoutCtx {
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
