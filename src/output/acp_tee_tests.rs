use super::acp_tee::{
    AcpTeeDirection, AcpTeeLineFmt, format_line_with_timestamp_acp_ansi,
    format_line_with_timestamp_acp_ansi_payload,
};
use super::{TermimadStdoutGate, termimad_inline_payload_for_stdout};

#[test]
fn ansi_acp_tee_directions_use_distinct_bracket_colors() {
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
fn kiss_stringify_acp_tee() {
    let _ = stringify!(AcpTeeDirection);
    let _ = stringify!(AcpTeeDirection::ToAgent);
    let _ = stringify!(AcpTeeDirection::FromAgent);
    let _ = stringify!(AcpTeeLineFmt);
    let _ = stringify!(super::TermimadStdoutGate);
    let _ = stringify!(super::acp_tee::AcpTeeStdoutEvent);
    let _ = stringify!(format_line_with_timestamp_acp_ansi);
    let _ = stringify!(format_line_with_timestamp_acp_ansi_payload);
    let _ = stringify!(super::acp_tee::print_stdout_acp_tee_line);
    let _ = stringify!(super::acp_tee::print_stdout_acp_tee_line_with_timestamp);
    let _ = stringify!(super::acp_tee::print_stdout_acp_tee_line_with_timestamp_dim_payload);
    let _ = stringify!(super::termimad_inline_payload_for_stdout);
}

#[test]
fn termimad_inline_bold_when_emit_and_tty_forced() {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: false,
        allow_inline_styling: true,
    };
    let s = termimad_inline_payload_for_stdout("**m**", &gate).expect("render");
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
    let rendered = termimad_inline_payload_for_stdout("plain", &gate).expect("render");
    assert_eq!(rendered, "plain");
}

#[test]
fn termimad_inline_none_when_emit_false_even_if_tty() {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: false,
        dim_payload: false,
        allow_inline_styling: true,
    };
    assert!(termimad_inline_payload_for_stdout("**m**", &gate).is_none());
}

#[test]
fn termimad_inline_none_when_dim_even_if_emit() {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: true,
        allow_inline_styling: true,
    };
    assert!(termimad_inline_payload_for_stdout("**m**", &gate).is_none());
}

#[test]
fn termimad_inline_none_when_styling_disabled() {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: false,
        allow_inline_styling: false,
    };
    assert!(termimad_inline_payload_for_stdout("**m**", &gate).is_none());
}
