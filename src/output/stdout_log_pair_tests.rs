use crate::output::stdout_log_pair::{
    AcpTeeDirection, AcpTeeLineFmt, acp_tee_payload_prefix, acp_tee_payload_prefix_width,
    format_line_acp_ansi_payload, stderr_tagged_display_and_log_line, stdout_acp_display_and_log,
    tagged_display_and_log_line_for_color, tagged_display_line_with_timestamp_ansi,
    tagged_log_line,
};
use crate::output::{
    MALVIN_WHO, WHO_M, WHO_O, WHO_T, format_heartbeat_stdout_ansi, format_line_stdout,
    is_log_timestamp_token, stdout_tagged_display_and_log_line,
};

fn heartbeat_stdout_ansi_keeps_who_color_through_payload() {
    let payload = "HB: 20260524.000000";
    let line = format_heartbeat_stdout_ansi(MALVIN_WHO, payload);
    assert!(line.contains(payload));
    assert!(line.contains('\x1b'));
}

fn tagged_log_line_includes_timestamp_and_payload() {
    let line = tagged_log_line("20260524.000000.000", MALVIN_WHO, "probe");
    assert!(line.contains("probe"));
    assert!(is_log_timestamp_token(
        line.split_whitespace().next().unwrap()
    ));
}

fn tagged_display_and_log_line_splits_terminal_from_log() {
    let (display, log) =
        tagged_display_and_log_line_for_color(MALVIN_WHO, "hb", Some("20260524.000000.000"), false);
    assert!(!display.starts_with("20"));
    assert!(log.starts_with("20260524"));
    assert_eq!(display, format_line_stdout(MALVIN_WHO, "hb"));
}

fn stderr_tagged_pair_uses_stderr_format() {
    let (display, log) =
        stderr_tagged_display_and_log_line(MALVIN_WHO, "err", Some("20260524.000000.000"));
    assert!(display.contains("err"));
    assert!(log.contains("err"));
}

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

fn tagged_display_resolves_timestamp_when_none() {
    let (_display, log) = stdout_tagged_display_and_log_line(MALVIN_WHO, "now-ts", None);
    let ts = log.split_whitespace().next().expect("timestamp");
    assert!(is_log_timestamp_token(ts));
}

fn tagged_display_line_with_timestamp_ansi_includes_payload() {
    let line = tagged_display_line_with_timestamp_ansi("20260524.000000.000", MALVIN_WHO, "ansi");
    assert!(line.contains("ansi"));
    assert!(line.contains("20260524.000000.000"));
}

fn tagged_display_and_log_line_color_branch() {
    let (display, log) = tagged_display_and_log_line_for_color(
        MALVIN_WHO,
        "color",
        Some("20260524.000000.000"),
        true,
    );
    assert!(display.contains("color"));
    assert!(log.contains("color"));
}

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
        format_line_acp_ansi_payload(&from_ctx),
        "direction still affects payload styling"
    );
}

fn acp_agent_who_prefix_matches_stdout_navy() {
    use crate::output::{format_line_stdout_ansi, who_tag_ansi};
    use crate::terminal_palette::ansi_tool_navy;

    for who in [WHO_M, WHO_O] {
        let stdout_prefix = format_line_stdout_ansi(who, "");
        let acp = format_line_acp_ansi_payload(&AcpTeeLineFmt {
            ts: "20260524.000000.000",
            direction: AcpTeeDirection::FromAgent,
            who,
            line: "",
            dim_payload: false,
        });
        assert!(
            stdout_prefix.contains(ansi_tool_navy()),
            "stdout who={who} must use navy; got {stdout_prefix:?}"
        );
        assert!(
            acp.contains(who_tag_ansi(who)),
            "ACP who={who} prefix must match who_tag_ansi; got {acp:?}"
        );
        assert!(
            !acp.contains('\n'),
            "prefix-only ACP line must be single-line; got {acp:?}"
        );
    }
}

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

#[cfg(test)]
pub(crate) fn assert_tool_payload_uses_verb_styling(line: &str) {
    use crate::terminal_palette::{ANSI_BOLD, ANSI_DIM, ANSI_RESET, ansi_tool_dark};

    let dim_sep = format!("{ANSI_RESET}{ANSI_DIM}");
    let dim_start = line
        .find(&dim_sep)
        .unwrap_or_else(|| panic!("expected dim tool payload; got {line:?}"));
    let payload = &line[dim_start + dim_sep.len()..];
    let dark_verb = format!("{ANSI_BOLD}{}", ansi_tool_dark());
    assert!(
        payload.contains(&dark_verb),
        "payload verb must use dark bold styling; got {payload:?} in {line:?}"
    );
    let dark_open = format!("{}[", ansi_tool_dark());
    assert!(
        !payload.starts_with(&dark_open),
        "payload must not start with styled open bracket; got {payload:?} in {line:?}"
    );
}

#[cfg(test)]
pub(crate) fn assert_acp_tool_summary_dim_preserves_bracket(line: &str) {
    use crate::output::who_tag_ansi;
    use crate::terminal_palette::ANSI_DIM;

    let bracket_end = line.find('|').expect("who pipe delimiter");
    assert!(
        line.contains(ANSI_DIM),
        "tee dims tool payload; got {line:?}"
    );
    assert!(
        line.find(ANSI_DIM).unwrap() > bracket_end,
        "dim must apply after who pipe; got {line:?}"
    );
    let prefix = &line[..=bracket_end];
    assert!(
        prefix.contains(who_tag_ansi(WHO_T)),
        "who prefix uses canonical who-tag color; got {line:?}"
    );
    assert!(
        !prefix.contains(ANSI_DIM),
        "who prefix must not be dimmed; got {line:?}"
    );
}

fn kiss_cov_assert_tool_payload_helpers() {
    use crate::output::who_tag_ansi;
    use crate::terminal_palette::{ANSI_BOLD, ANSI_DIM, ANSI_RESET, ansi_tool_dark};
    let who = who_tag_ansi(WHO_T);
    let dark = ansi_tool_dark();
    let line =
        format!("{who}|{ANSI_RESET}{ANSI_DIM}{ANSI_BOLD}{dark}Run{ANSI_RESET}{ANSI_DIM} something");
    assert_tool_payload_uses_verb_styling(&line);
    assert_acp_tool_summary_dim_preserves_bracket(&line);
}

#[test]
fn kiss_bundled_output_stdout_log_pair_tests() {
    heartbeat_stdout_ansi_keeps_who_color_through_payload();
    tagged_log_line_includes_timestamp_and_payload();
    tagged_display_and_log_line_splits_terminal_from_log();
    stderr_tagged_pair_uses_stderr_format();
    acp_tee_display_and_log_split_prefix_from_payload();
    tagged_display_resolves_timestamp_when_none();
    tagged_display_line_with_timestamp_ansi_includes_payload();
    tagged_display_and_log_line_color_branch();
    acp_bracket_color_covers_both_directions();
    acp_agent_who_prefix_matches_stdout_navy();
    acp_bracket_payload_supports_dim_mode();
    kiss_cov_assert_tool_payload_helpers();
}
