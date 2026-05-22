use super::acp_tee::{
    AcpTeeDirection, AcpTeeLineFmt, AcpTeeStdoutEvent, acp_tee_display_line, acp_tee_log_line,
    acp_tee_log_prefix, format_line_with_timestamp_acp_ansi, print_stdout_acp_tool_summary_tee,
    print_stdout_acp_tee_line, print_stdout_acp_tee_line_with_timestamp,
};
use super::acp_tee_format::{format_line_acp_ansi_payload, format_line_with_timestamp_acp_ansi_payload};

#[test]
fn kpop_h1_and_h5_timestamp_present_on_acp_tee_helpers() {
    let ts = "20260413.121314.015";
    let ctx = AcpTeeLineFmt {
        ts,
        direction: AcpTeeDirection::FromAgent,
        who: "<check_plan",
        line: "[thinking]",
        dim_payload: true,
    };
    let tool_log = super::format_line_with_timestamp(ts, "<check_plan", "Run echo hi · 1ms · ✓");
    let tee_log = acp_tee_log_line(&ctx);
    assert!(
        super::is_log_timestamp_token(tool_log.split_whitespace().next().unwrap_or("")),
        "tool summary log should start with timestamp token; got {tool_log:?}"
    );
    assert!(
        super::is_log_timestamp_token(tee_log.split_whitespace().next().unwrap_or("")),
        "acp_tee_log_line should include timestamp; got {tee_log:?}"
    );
    let prefix = acp_tee_log_prefix(&AcpTeeLineFmt {
        ts,
        direction: AcpTeeDirection::FromAgent,
        who: "<kpop",
        line: "",
        dim_payload: false,
    });
    assert!(
        prefix.starts_with("20260413"),
        "markdown/log prefix should include timestamp; got {prefix:?}"
    );
}

#[test]
fn acp_display_and_log_helpers_include_timestamp_on_stdout_formatted_lines() {
    let ctx = AcpTeeLineFmt {
        ts: "20260413.121314.015",
        direction: AcpTeeDirection::ToAgent,
        who: "malvin",
        line: "hello",
        dim_payload: false,
    };
    let plain_acp = format_line_acp_ansi_payload(&ctx);
    assert!(!plain_acp.contains("20260413"), "raw non-timestamp helper remains non-ts");
    assert!(acp_tee_display_line(&ctx).contains("hello"));
    assert!(acp_tee_display_line(&ctx).starts_with("20260413"));
    assert!(acp_tee_log_line(&ctx).starts_with("20260413"));
    assert!(acp_tee_log_prefix(&ctx).starts_with("20260413"));
    print_stdout_acp_tee_line(AcpTeeDirection::FromAgent, "<w", "probe");
    print_stdout_acp_tee_line_with_timestamp(&AcpTeeStdoutEvent {
        direction: AcpTeeDirection::ToAgent,
        who: "w",
        line: "plain",
        ts: "t",
        emit_stdout_markdown: false,
        dim_payload: false,
    });
    print_stdout_acp_tool_summary_tee(
        &AcpTeeStdoutEvent {
            direction: AcpTeeDirection::FromAgent,
            who: "k",
            line: "Run cargo test · 1.0s · ✓",
            ts: "ts",
            emit_stdout_markdown: false,
            dim_payload: false,
        },
        "Run cargo test · 1.0s · ✓",
    );
}

#[test]
fn ansi_acp_tee_directions_use_distinct_bracket_colors() {
    let _: Option<super::acp_tee::AcpTeeStdoutEvent> = None;
    let _ = super::acp_tee::print_stdout_acp_tee_line;
    let _ = super::acp_tee::print_stdout_acp_tee_line_with_timestamp;
    let _ = super::acp_tee::print_stdout_acp_tee_line_with_timestamp_dim_plain;
    let _: Option<super::acp_tee_markdown::TermimadStdoutGate> = None;
    let to_line = format_line_with_timestamp_acp_ansi(
        "20260413.121314.015",
        AcpTeeDirection::ToAgent,
        ">stem",
        "out",
    );
    let from_line = format_line_with_timestamp_acp_ansi(
        "20260413.121314.015",
        AcpTeeDirection::FromAgent,
        "<stem",
        "in",
    );
    assert!(to_line.contains('\x1b'));
    assert!(from_line.contains('\x1b'));
    assert_ne!(to_line, from_line);
    assert!(to_line.ends_with(" out"));
    assert!(from_line.ends_with(" in"));
}

#[test]
fn ansi_acp_tee_can_dim_payload_text() {
    let line = format_line_with_timestamp_acp_ansi_payload(&AcpTeeLineFmt {
        ts: "20260413.121314.015",
        direction: AcpTeeDirection::FromAgent,
        who: "<stem",
        line: "[thinking]",
        dim_payload: true,
    });
    assert!(line.contains("\x1b[90m[thinking]\x1b[0m"));
}
