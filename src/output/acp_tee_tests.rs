use super::acp_tee::{
    AcpTeeDirection, AcpTeeLineFmt, format_line_with_timestamp_acp_ansi,
    format_line_with_timestamp_acp_ansi_payload,
};
use super::acp_tee_markdown::termimad_text_lines_for_stdout;
use super::{TermimadStdoutGate, termimad_inline_payload_for_stdout};

#[test]
fn ansi_acp_tee_directions_use_distinct_bracket_colors() {
    let _: Option<super::acp_tee::AcpTeeStdoutEvent> = None;
    let _ = super::acp_tee::print_stdout_acp_tee_line;
    let _ = super::acp_tee::print_stdout_acp_tee_line_with_timestamp;
    let _ = super::acp_tee::print_stdout_acp_tee_line_with_timestamp_dim_plain;
    let _: Option<super::TermimadStdoutGate> = None;
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
