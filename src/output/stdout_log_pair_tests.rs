use crate::output::stdout_log_pair::{
    acp_tee_payload_prefix, acp_tee_payload_prefix_width,
    format_line_acp_ansi_payload, stderr_tagged_display_and_log_line, stdout_acp_display_and_log,
    tagged_display_and_log_line_for_color, tagged_display_line_with_timestamp_ansi, tagged_log_line,
    AcpTeeDirection, AcpTeeLineFmt,
};
use crate::output::{format_heartbeat_stdout_ansi, format_line_stdout, is_log_timestamp_token, stdout_tagged_display_and_log_line, MALVIN_WHO};

#[test]
fn heartbeat_stdout_ansi_keeps_who_color_through_payload() {
    let payload = "HB: 20260524.000000";
    let line = format_heartbeat_stdout_ansi(MALVIN_WHO, payload);
    assert!(line.contains(payload));
    assert!(line.contains('\x1b'));
}

#[test]
fn tagged_log_line_includes_timestamp_and_payload() {
    let line = tagged_log_line("20260524.000000.000", MALVIN_WHO, "probe");
    assert!(line.contains("probe"));
    assert!(is_log_timestamp_token(line.split_whitespace().next().unwrap()));
}

#[test]
fn tagged_display_and_log_line_splits_terminal_from_log() {
    let (display, log) =
        tagged_display_and_log_line_for_color(MALVIN_WHO, "hb", Some("20260524.000000.000"), false);
    assert!(!display.starts_with("20"));
    assert!(log.starts_with("20260524"));
    assert_eq!(display, format_line_stdout(MALVIN_WHO, "hb"));
}

#[test]
fn stderr_tagged_pair_uses_stderr_format() {
    let (display, log) = stderr_tagged_display_and_log_line(MALVIN_WHO, "err", Some("20260524.000000.000"));
    assert!(display.contains("err"));
    assert!(log.contains("err"));
}

#[test]
fn acp_tee_display_and_log_split_prefix_from_payload() {
    let ctx = AcpTeeLineFmt {
        ts: "20260524.000000.000",
        direction: AcpTeeDirection::FromAgent,
        who: MALVIN_WHO,
        line: "payload",
        dim_payload: false,
    };
    let (display, log) = stdout_acp_display_and_log(&ctx, &ctx);
    assert!(display.contains("payload"));
    assert!(log.contains("payload"));
    assert!(acp_tee_payload_prefix_width(&acp_tee_payload_prefix(&ctx)) > 0);
}

#[test]
fn tagged_display_resolves_timestamp_when_none() {
    let (_display, log) = stdout_tagged_display_and_log_line(MALVIN_WHO, "now-ts", None);
    let ts = log.split_whitespace().next().expect("timestamp");
    assert!(is_log_timestamp_token(ts));
}

#[test]
fn tagged_display_line_with_timestamp_ansi_includes_payload() {
    let line = tagged_display_line_with_timestamp_ansi("20260524.000000.000", MALVIN_WHO, "ansi");
    assert!(line.contains("ansi"));
    assert!(line.contains("20260524.000000.000"));
}

#[test]
fn tagged_display_and_log_line_color_branch() {
    let (display, log) =
        tagged_display_and_log_line_for_color(MALVIN_WHO, "color", Some("20260524.000000.000"), true);
    assert!(display.contains("color"));
    assert!(log.contains("color"));
}

#[test]
fn acp_bracket_color_covers_both_directions() {
    let to_ctx = AcpTeeLineFmt {
        ts: "20260524.000000.000",
        direction: AcpTeeDirection::ToAgent,
        who: MALVIN_WHO,
        line: "payload",
        dim_payload: false,
    };
    let from_ctx = AcpTeeLineFmt {
        direction: AcpTeeDirection::FromAgent,
        ..to_ctx
    };
    assert_ne!(
        format_line_acp_ansi_payload(&to_ctx),
        format_line_acp_ansi_payload(&from_ctx)
    );
}

#[test]
fn acp_bracket_payload_supports_dim_mode() {
    let ctx = AcpTeeLineFmt {
        ts: "20260524.000000.000",
        direction: AcpTeeDirection::FromAgent,
        who: MALVIN_WHO,
        line: "dim-payload",
        dim_payload: true,
    };
    let payload = format_line_acp_ansi_payload(&ctx);
    assert!(payload.contains("dim-payload"));
}
