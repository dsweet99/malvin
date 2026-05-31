use crate::output::stdout_log_pair::{
    acp_tee_log_line, acp_tee_payload_prefix, acp_tee_payload_prefix_width,
    format_line_acp_ansi_payload, stderr_tagged_display_and_log_line, stdout_acp_display_and_log,
    tagged_display_and_log_line_for_color, tagged_display_line_with_timestamp_ansi, tagged_log_line,
    AcpTeeDirection, AcpTeeLineFmt,
};
use crate::output::{
    format_heartbeat_stdout_ansi, format_line_stdout, format_who_tag_delim, format_who_tag_prefix,
    is_log_timestamp_token, stdout_tagged_display_and_log_line, ERROR_WHO, MALVIN_WHO, WARNING_WHO,
    WHO_B, WHO_H, WHO_M, WHO_O, WHO_T, WHO_U,
};

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

#[cfg(test)]
pub(crate) fn assert_tool_payload_uses_verb_styling(line: &str) {
    use crate::terminal_palette::{ansi_tool_dark, ANSI_BOLD, ANSI_DIM, ANSI_RESET};

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
    use crate::terminal_palette::{ansi_tool_dark, ANSI_DIM};

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
        prefix.contains(ansi_tool_dark()),
        "who prefix stays dark; got {line:?}"
    );
    assert!(
        !prefix.contains(ANSI_DIM),
        "who prefix must not be dimmed; got {line:?}"
    );
}

fn log_line_uses_delim_without_trailing_space(log: &str, who: &str, payload: &str) -> bool {
    let delim = format_who_tag_delim(who);
    log.ends_with(&format!("{delim}{payload}"))
}

const FUZZ_WHO_TAGS: &[&str] = &[WHO_O, WHO_M, WHO_B, WHO_T, WHO_U, WHO_H];

#[test]
fn tagged_log_line_omits_space_after_who_pipe_for_all_tags() {
    const TAGS: &[&str] = &[WHO_O, WHO_M, WHO_B, WHO_T, WHO_U, WHO_H, ERROR_WHO, WARNING_WHO];
    let ts = "20260524.000000.000";
    for who in TAGS {
        let payload = "probe payload";
        let log = tagged_log_line(ts, who, payload);
        assert!(
            log_line_uses_delim_without_trailing_space(&log, who, payload),
            "who={who}: {log:?}"
        );
    }
}

#[test]
fn display_log_metamorphic_pipe_space_only_for_thought_and_tool() {
    const TAGS: &[&str] = &[WHO_O, WHO_M, WHO_B, WHO_T];
    let ts = "20260524.000000.000";
    for who in TAGS {
        for payload in ["Run cargo test", "internal reasoning", ""] {
            let (display, log) =
                tagged_display_and_log_line_for_color(who, payload, Some(ts), false);
            assert_eq!(log, tagged_log_line(ts, who, payload));
            assert_eq!(display, format!("{}{payload}", format_who_tag_prefix(who)));
            assert!(log_line_uses_delim_without_trailing_space(&log, who, payload));
        }
    }
}

#[test]
fn log_rejects_decorative_space_after_who_pipe() {
    let ts = "20260524.000000.000";
    let who = WHO_O;
    let payload = "Logs: /tmp/run";
    let log = tagged_log_line(ts, who, payload);
    let delim = format_who_tag_delim(who);
    let bad = format!("{ts} {delim} {payload}");
    assert_ne!(log, bad, "log must not insert decorative space after who pipe");
    assert!(log_line_uses_delim_without_trailing_space(&log, who, payload));
}

#[test]
fn acp_tee_log_line_omits_space_after_who_pipe() {
    let payload = "Run echo hi · 1ms · ✓";
    let ctx = AcpTeeLineFmt {
        ts: "20260524.000000.000",
        direction: AcpTeeDirection::FromAgent,
        who: WHO_T,
        line: payload,
        dim_payload: true,
    };
    let log = acp_tee_log_line(&ctx);
    assert!(log_line_uses_delim_without_trailing_space(&log, WHO_T, payload), "{log:?}");
}

#[test]
fn tagged_log_line_no_pipe_space_fuzz() {
    use rand::{Rng, SeedableRng};

    let seed = std::env::var("LOG_PIPE_FUZZ_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(rand::random);
    eprintln!("tagged_log_line_no_pipe_space_fuzz seed: {seed}");
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let ts = "20260524.000000.000";
    for _ in 0..200 {
        let who = FUZZ_WHO_TAGS[rng.gen_range(0..FUZZ_WHO_TAGS.len())];
        let lead_spaces: String = (0..rng.gen_range(0..3)).map(|_| ' ').collect();
        let payload = format!("{lead_spaces}payload{}", rng.gen_range(0..u32::MAX));
        let log = tagged_log_line(ts, who, &payload);
        assert!(
            log_line_uses_delim_without_trailing_space(&log, who, &payload),
            "seed={seed} log={log:?}"
        );
    }
}
