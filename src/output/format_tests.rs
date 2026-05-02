use super::acp_tee::{AcpTeeDirection, print_stdout_acp_tee_line};
use super::{
    LEARNING_PLACEHOLDER, LOG_TAG_INNER_WIDTH, MALVIN_WHO, format_acp_directional_tag_prefix,
    format_line, format_line_with_timestamp, format_line_with_timestamp_ansi, format_log_tag_inner,
    init_stdout_style, is_command_prelude_line, print_outgoing_prompt_log, print_stderr_line,
    print_stdout_line, print_stdout_text,
};

#[test]
fn formats_expected_mini_header() {
    let inner = format_log_tag_inner("kpop");
    assert_eq!(
        format_line_with_timestamp("20260413.121314.015", "kpop", "hello"),
        format!("20260413.121314.015:[{inner}]: hello")
    );
}

#[test]
fn log_tag_inner_is_fixed_width() {
    assert_eq!(
        format_log_tag_inner("plan").chars().count(),
        LOG_TAG_INNER_WIDTH
    );
    let long = "a".repeat(100);
    assert_eq!(
        format_log_tag_inner(&long).chars().count(),
        LOG_TAG_INNER_WIDTH
    );
}

#[test]
fn ansi_timestamp_line_keeps_payload_plain() {
    let plain = format_line_with_timestamp("20260413.121314.015", "kpop", "hello");
    assert!(!plain.contains('\x1b'));
    let ansi = format_line_with_timestamp_ansi("20260413.121314.015", "kpop", "hello");
    assert!(ansi.contains('\x1b'));
    assert!(ansi.ends_with(" hello"));
}

#[test]
fn detects_prefixed_and_unprefixed_command_prelude() {
    let inner = format_log_tag_inner(MALVIN_WHO);
    assert!(is_command_prelude_line("Command: malvin code @plan.md"));
    assert!(is_command_prelude_line(&format!(
        "20260413.121314.015:[{inner}]: Command: malvin code @plan.md"
    )));
    assert!(!is_command_prelude_line(
        "20260413.121314.015:[kpop]: not a command line"
    ));
}

#[test]
fn exported_constants_match_public_contract() {
    assert_eq!(MALVIN_WHO, "malvin");
    assert_eq!(LEARNING_PLACEHOLDER, "[learning...]");
}

#[test]
fn smoke_print_and_format_paths_cover_helpers() {
    assert_eq!(format_acp_directional_tag_prefix('>', "x"), ">x");
    let _ = format_line("who", "body");
    init_stdout_style(true);
    print_stdout_line("u", "one");
    print_stdout_acp_tee_line(AcpTeeDirection::FromAgent, "<w", "two");
    print_stderr_line("e", "err");
    print_stdout_text("t", "a\nb");
    print_outgoing_prompt_log("main");
    let mut it = super::logical_lines("x\ny");
    assert_eq!(it.next(), Some("x"));
    assert_eq!(it.next(), Some("y"));
}

#[test]
fn kiss_stringify_output_symbols() {
    let _ = stringify!(super::format_log_tag_inner);
    let _ = stringify!(super::format_acp_directional_tag_prefix);
    let _ = stringify!(super::format_line_with_timestamp);
    let _ = stringify!(super::timestamp_now_string);
    let _ = stringify!(super::format_line);
    let _ = stringify!(super::format_line_with_timestamp_ansi);
    let _ = stringify!(super::init_stdout_style);
    let _ = stringify!(super::stdout_use_color);
    let _ = stringify!(super::print_stdout_line);
    let _ = stringify!(super::print_stderr_line);
    let _ = stringify!(super::print_stdout_text);
    let _ = stringify!(super::print_outgoing_prompt_log);
    let _ = stringify!(super::is_command_prelude_line);
    let _ = stringify!(super::logical_lines);
    let _ = stringify!(super::print_stdout_acp_tee_line);
    let _ = stringify!(super::print_stdout_acp_tee_line_with_timestamp);
}
