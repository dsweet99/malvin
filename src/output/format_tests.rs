use super::acp_tee::{AcpTeeDirection, print_stdout_acp_tee_line};
use super::{
    LEARNING_PLACEHOLDER, LOG_TAG_INNER_WIDTH, MALVIN_WHO, format_acp_directional_tag_prefix,
    format_line, format_line_with_timestamp, format_line_with_timestamp_ansi, format_log_tag_inner,
    init_stdout_style, is_command_prelude_line, print_outgoing_prompt_log, print_stderr_line,
    print_stdout_line, print_stdout_raw_line, print_stdout_text, set_stdout_log_path,
};

#[test]
fn formats_expected_mini_header() {
    let inner = format_log_tag_inner("kpop");
    assert_eq!(
        format_line_with_timestamp("20260413.121314.015", "kpop", "hello"),
        format!("20260413.121314.015 [{inner}] hello")
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
        "[{inner}] Command: malvin code @plan.md"
    )));
    assert!(is_command_prelude_line(&format_line_with_timestamp(
        "20260413.121314.015",
        MALVIN_WHO,
        "Command: malvin code @plan.md"
    )));
    assert!(!is_command_prelude_line(
        "20260413.121314.015 [kpop] not a command line"
    ));
}

#[test]
fn command_prelude_detection_ignores_unrelated_bracket_command_substrings() {
    assert!(
        !is_command_prelude_line("agent note ] Command: not a malvin prelude"),
        "only fixed-width tagged preludes should match, not arbitrary '] Command:' text"
    );
}

#[test]
fn command_prelude_rejects_short_bracket_tags_and_non_timestamp_prefixes() {
    assert!(!is_command_prelude_line("[kpop] Command: malvin code"));
    assert!(!is_command_prelude_line(
        "agent-ts [malvin         ] Command: malvin code"
    ));
    assert!(!is_command_prelude_line(
        "20260413 [malvin         ] Command: malvin code"
    ));
    assert!(!is_command_prelude_line(""));
    assert!(!is_command_prelude_line("not a command"));
}

#[test]
fn command_prelude_rejects_dot_only_timestamp_token() {
    use super::is_log_timestamp_token;

    assert!(!is_log_timestamp_token("."));
    let inner = format_log_tag_inner(MALVIN_WHO);
    assert!(
        !is_command_prelude_line(&format!(". [{inner}] Command: not-a-real-prelude")),
        "a lone '.' must not qualify as a log timestamp prefix"
    );
}

#[test]
fn bracket_tag_payload_and_timestamp_token_helpers() {
    use super::{is_log_timestamp_token, payload_after_fixed_width_bracket_tag};

    assert!(!is_log_timestamp_token(""));
    assert!(!is_log_timestamp_token("nodot"));
    assert!(!is_log_timestamp_token("."));
    assert!(!is_log_timestamp_token("20260413.121314"));
    assert!(is_log_timestamp_token("20260413.121314.015"));
    assert_eq!(payload_after_fixed_width_bracket_tag("no-bracket"), None);
    let inner = format_log_tag_inner(MALVIN_WHO);
    assert_eq!(
        payload_after_fixed_width_bracket_tag(&format!("[{inner}] Command: x")),
        Some("Command: x")
    );
    assert_eq!(
        payload_after_fixed_width_bracket_tag(&format!("[{inner}]bad")),
        None
    );
}

#[test]
fn exported_constants_match_public_contract() {
    assert_eq!(MALVIN_WHO, "malvin");
    assert_eq!(super::WARNING_WHO, "warning");
    assert_eq!(super::ERROR_WHO, "error");
    assert_eq!(LEARNING_PLACEHOLDER, "[learning...]");
}

#[test]
fn ansi_who_tag_uses_yellow_for_warning_and_red_for_error() {
    let ts = "20260413.121314.015";
    let warn = super::format_line_with_timestamp_ansi(ts, super::WARNING_WHO, "");
    let err = super::format_line_with_timestamp_ansi(ts, super::ERROR_WHO, "");
    assert!(warn.contains("\x1b[33m"));
    assert!(err.contains("\x1b[31m"));
}

#[test]
fn outgoing_prompt_log_who_tag_uses_stem_bracket_keeps_md() {
    let _guard = super::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    init_stdout_style(true);
    print_outgoing_prompt_log("check_plan", "check_plan.md");
    set_stdout_log_path(None);
    let text = std::fs::read_to_string(path).expect("read stdout log");
    let who = format_acp_directional_tag_prefix('>', "check_plan");
    let inner = format_log_tag_inner(&who);
    assert!(
        text.contains(&format!("[{inner}] [check_plan.md...]")),
        "expected stem who tag and .md bracket payload: {text:?}"
    );
    assert!(
        !text.contains(">check_plan.md"),
        "who tag must not include .md suffix: {text:?}"
    );
}

#[test]
fn smoke_print_and_format_paths_cover_helpers() {
    assert_eq!(format_acp_directional_tag_prefix('>', "x"), ">x");
    assert!(!crate::time_format::timestamp_now_string().is_empty());
    let (max_payload, _) = super::terminal_wrap::stdout_line_wrap_meta("malvin", "line");
    let wrapped = super::terminal_wrap::wrap_words_bounded(max_payload, "hello world");
    assert!(!wrapped.is_empty());
    let _ = format_line("who", "body");
    init_stdout_style(true);
    print_stdout_line("u", "one");
    print_stdout_acp_tee_line(AcpTeeDirection::FromAgent, "<w", "two");
    print_stderr_line("e", "err");
    print_stdout_text("t", "a\nb");
    print_outgoing_prompt_log("check_plan", "check_plan.md");
    let mut it = super::logical_lines("x\ny");
    assert_eq!(it.next(), Some("x"));
    assert_eq!(it.next(), Some("y"));
}

#[test]
fn stdout_log_mirrors_stdout_without_timestamps() {
    let (display, tagged_line) = crate::output::stdout_display::stdout_display_and_log("u", "m");
    assert_eq!(display, tagged_line);
    let first_token = tagged_line
        .split_whitespace()
        .next()
        .expect("tagged stdout line should not be empty");
    assert!(
        !super::is_log_timestamp_token(first_token),
        "stdout log should mirror stdout without timestamp prefixes: {tagged_line:?}"
    );
}

#[test]
fn append_stdout_log_line_writes_when_path_set() {
    let _guard = super::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("append.log");
    set_stdout_log_path(Some(path.clone()));
    print_stdout_raw_line("append probe");
    set_stdout_log_path(None);
    let text = std::fs::read_to_string(path).expect("read");
    assert!(text.contains("append probe"));
}

#[test]
fn timestamp_now_string_cross_module_smoke() {
    assert!(!crate::time_format::timestamp_now_string().is_empty());
}

#[test]
fn output_timestamp_wrapper_nonempty() {
    let _ = crate::stdout_log_path::set_stdout_log_path;
    let _ = super::stdout_use_color;
    let _ = super::append_stdout_log_line;
    let _ = super::print_stdout_rendered_line;
    let _ = crate::output::stdout_display::stdout_display_and_log;
    let _ = crate::output::stdout_heartbeat::emit_heartbeat_line;
    let _ = crate::output::stdout_heartbeat::spawn_wall_clock_poller_if_needed;
    let _ = super::stderr_log::print_log_warning;
    let _ = super::stderr_log::print_log_error;
    let _ = super::print_stdout_acp_tee_line_with_timestamp;
    assert!(!super::timestamp_now_string().is_empty());
    assert!(!crate::time_format::timestamp_now_string().is_empty());
}
