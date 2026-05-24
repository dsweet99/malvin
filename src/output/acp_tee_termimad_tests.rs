use super::acp_tee::{
    AcpTeeDirection, AcpTeeLineFmt, AcpTeeStdoutEvent, print_stdout_acp_tee_line_with_timestamp,
};
use super::acp_tee_markdown::termimad_text_lines_for_stdout;
use super::{TermimadStdoutGate, termimad_inline_payload_for_stdout};
use crate::output::stdout_log_pair::{
    acp_tee_payload_prefix, acp_tee_payload_prefix_width, stdout_acp_prefix_rendered_line,
};

fn markdown_rendered_tee_pairs(ts: &str, who: &str, line: &str) -> Vec<(String, String)> {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: false,
        allow_inline_styling: true,
    };
    let prefix_ctx = AcpTeeLineFmt {
        ts,
        direction: AcpTeeDirection::FromAgent,
        who,
        line: "",
        dim_payload: false,
    };
    let prefix_len = acp_tee_payload_prefix_width(&acp_tee_payload_prefix(&prefix_ctx));
    let (max_payload, _) = super::terminal_wrap::line_wrap_for_prefix_len(
        prefix_len,
        line,
        super::terminal_wrap::stdout_allows_log_word_wrap(),
    );
    termimad_text_lines_for_stdout(line, gate, max_payload)
        .expect("markdown should render")
        .into_iter()
        .map(|rendered| stdout_acp_prefix_rendered_line(&prefix_ctx, &rendered))
        .collect()
}

#[test]
fn termimad_inline_bold_when_emit_and_inline_styling_gate_true() {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: false,
        allow_inline_styling: true,
    };
    let s = termimad_inline_payload_for_stdout("**m**", gate).expect("render");
    assert!(
        s.contains('\x1b'),
        "expected termimad ANSI styling in rendered payload: {s:?}"
    );
    assert!(
        !s.contains("**m**"),
        "expected markdown markers to be consumed: {s:?}"
    );
}

#[test]
fn termimad_inline_plain_when_no_markdown_syntax() {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: false,
        allow_inline_styling: true,
    };
    let rendered = termimad_inline_payload_for_stdout("plain", gate).expect("render");
    assert_eq!(rendered, "plain");
}

#[test]
fn termimad_inline_none_when_emit_false_even_if_tty() {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: false,
        dim_payload: false,
        allow_inline_styling: true,
    };
    assert!(termimad_inline_payload_for_stdout("**m**", gate).is_none());
}

#[test]
fn termimad_inline_wraps_dim_when_dim_with_emit() {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: true,
        allow_inline_styling: true,
    };
    let s = termimad_inline_payload_for_stdout("**m** tail", gate).expect("render");
    assert!(s.starts_with("\x1b[90m"), "expected outer dim wrap: {s:?}");
    assert!(
        s.ends_with("\x1b[0m"),
        "expected reset after dim wrap: {s:?}"
    );
    assert!(!s.contains("**m**"), "expected markdown consumed: {s:?}");
    assert!(
        s.contains("\x1b[0m\x1b[90m tail"),
        "expected dim to resume after inner reset: {s:?}"
    );
}

#[test]
fn termimad_inline_none_when_styling_disabled() {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: false,
        allow_inline_styling: false,
    };
    assert!(termimad_inline_payload_for_stdout("**m**", gate).is_none());
}

#[test]
fn termimad_text_lines_wrap_list_item_at_width() {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: false,
        allow_inline_styling: true,
    };
    let lines =
        termimad_text_lines_for_stdout("- **alpha** beta gamma delta", gate, 10).expect("render");
    assert!(
        lines.len() > 1,
        "expected wrapped list item lines, got {lines:?}"
    );
    assert!(
        lines.iter().all(|line| !line.is_empty()),
        "expected non-empty rendered lines, got {lines:?}"
    );
    assert!(
        lines.iter().all(|line| !line.contains("**alpha**")),
        "expected markdown markers to be consumed: {lines:?}"
    );
}

#[test]
fn termimad_text_lines_keep_dim_across_inner_resets() {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: true,
        allow_inline_styling: true,
    };
    let lines = termimad_text_lines_for_stdout("- **alpha** tail", gate, 40).expect("render");
    assert_eq!(lines.len(), 1, "unexpected wrap for short input: {lines:?}");
    let line = &lines[0];
    assert!(
        line.starts_with("\x1b[90m"),
        "expected outer dim wrap: {line:?}"
    );
    assert!(
        line.contains("\x1b[0m\x1b[90m tail"),
        "expected dim to resume after inner reset: {line:?}"
    );
}

#[test]
#[allow(unsafe_code)]
fn acp_tee_markdown_tee_path_omits_wall_clock_prefix_on_live_display() {
    use crate::ansi_strip::strip_ansi_escapes;

    let prev_no_color = std::env::var_os("NO_COLOR");
    unsafe {
        std::env::remove_var("NO_COLOR");
    }
    let _guard = super::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("stdout.log");
    super::set_stdout_log_path(Some(path.clone()));
    super::init_stdout_style(false);
    let ts = "20260413.121314.015";
    let line = "# Title";
    let expected_pairs = markdown_rendered_tee_pairs(ts, "<md", line);
    print_stdout_acp_tee_line_with_timestamp(&AcpTeeStdoutEvent {
        direction: AcpTeeDirection::FromAgent,
        who: "<md",
        line,
        ts,
        emit_stdout_markdown: true,
        dim_payload: false,
    });
    super::set_stdout_log_path(None);
    unsafe {
        match prev_no_color {
            Some(v) => std::env::set_var("NO_COLOR", v),
            None => std::env::remove_var("NO_COLOR"),
        }
    }
    let log_text = std::fs::read_to_string(path).unwrap();
    for (display, log) in expected_pairs {
        assert!(
            !strip_ansi_escapes(&display).starts_with(ts),
            "rendered markdown tee display must omit wall-clock prefix; got {display:?}"
        );
        let expected_log = strip_ansi_escapes(&log);
        assert!(
            log_text.lines().any(|file_line| file_line == expected_log),
            "stdout.log should contain rendered markdown log segment; expected={expected_log:?} file={log_text:?}"
        );
    }
}

#[test]
fn acp_tee_markdown_tee_path_writes_timestamped_stdout_log_lines() {
    let _guard = super::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("stdout.log");
    super::set_stdout_log_path(Some(path.clone()));
    super::init_stdout_style(true);
    print_stdout_acp_tee_line_with_timestamp(&AcpTeeStdoutEvent {
        direction: AcpTeeDirection::FromAgent,
        who: "<md",
        line: "- **alpha** beta gamma delta epsilon zeta eta theta",
        ts: "20260413.121314.015",
        emit_stdout_markdown: true,
        dim_payload: false,
    });
    super::set_stdout_log_path(None);
    let text = std::fs::read_to_string(path).unwrap();
    assert!(
        !text.is_empty(),
        "markdown tee path should write stdout.log; got {text:?}"
    );
    for line in text.lines() {
        assert!(
            super::is_log_timestamp_token(line.split_whitespace().next().unwrap_or("")),
            "markdown tee segments should be timestamped; got {line:?}"
        );
    }
}

#[test]
fn termimad_text_lines_only_for_structural_markdown_and_safe_widths() {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: false,
        allow_inline_styling: true,
    };
    assert!(termimad_text_lines_for_stdout("**bold**", gate, 40).is_none());
    assert!(termimad_text_lines_for_stdout("# heading", gate, 40).is_some());
    assert!(termimad_text_lines_for_stdout("- item", gate, 40).is_some());
    assert!(termimad_text_lines_for_stdout("1. ordered", gate, 40).is_some());
    assert!(termimad_text_lines_for_stdout("# x", gate, 2).is_none());
}
