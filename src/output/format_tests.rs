use super::acp_tee::{AcpTeeDirection, print_stdout_acp_tee_line};
use super::{
    LOG_TAG_INNER_WIDTH, MALVIN_WHO, WHO_M, WHO_O, WHO_T, WHO_U,
    format_acp_directional_tag_prefix,
    format_line, format_line_with_timestamp, format_line_with_timestamp_ansi, format_log_tag_inner,
    format_who_tag_delim, format_who_tag_prefix, init_stdout_style, is_command_prelude_line,
    print_outgoing_prompt_log, print_stderr_line, print_stdout_line, print_stdout_raw_line,
    print_stdout_text, set_stdout_log_path,
};

#[test]
fn formats_expected_mini_header() {
    let delim = format_who_tag_delim(WHO_M);
    assert_eq!(
        format_line_with_timestamp("20260413.121314.015", WHO_M, "hello"),
        format!("20260413.121314.015 {delim}hello")
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
    let plain = format_line_with_timestamp("20260413.121314.015", WHO_M, "hello");
    assert!(!plain.contains('\x1b'));
    let ansi = format_line_with_timestamp_ansi("20260413.121314.015", WHO_M, "hello");
    assert!(ansi.contains('\x1b'));
    assert!(ansi.ends_with("hello"));
}

#[test]
fn detects_prefixed_and_unprefixed_command_prelude() {
    let prefix = format_who_tag_prefix(MALVIN_WHO);
    assert!(is_command_prelude_line("Command: malvin code @plan.md"));
    assert!(is_command_prelude_line(&format!(
        "{prefix}Command: malvin code @plan.md"
    )));
    assert!(is_command_prelude_line(&format_line_with_timestamp(
        "20260413.121314.015",
        MALVIN_WHO,
        "Command: malvin code @plan.md"
    )));
    assert!(!is_command_prelude_line(
        "20260413.121314.015 m|not a command line"
    ));
}

#[test]
fn command_prelude_detection_ignores_unrelated_bracket_command_substrings() {
    assert!(
        !is_command_prelude_line("agent note | Command: not a malvin prelude"),
        "only fixed-width tagged preludes should match, not arbitrary '| Command:' text"
    );
}

#[test]
fn command_prelude_rejects_short_who_tags_and_non_timestamp_prefixes() {
    assert!(!is_command_prelude_line("mm| Command: malvin code"));
    assert!(!is_command_prelude_line(
        "agent-ts o| Command: malvin code"
    ));
    assert!(!is_command_prelude_line(
        "20260413 o| Command: malvin code"
    ));
    assert!(!is_command_prelude_line(""));
    assert!(!is_command_prelude_line("not a command"));
}

#[test]
fn command_prelude_rejects_dot_only_timestamp_token() {
    use super::is_log_timestamp_token;

    assert!(!is_log_timestamp_token("."));
    let prefix = format_who_tag_prefix(MALVIN_WHO);
    assert!(
        !is_command_prelude_line(&format!(". {prefix}Command: not-a-real-prelude")),
        "a lone '.' must not qualify as a log timestamp prefix"
    );
}

#[test]
fn who_tag_payload_and_timestamp_token_helpers() {
    use super::{
        is_log_timestamp_token, payload_after_fixed_width_bracket_tag,
        payload_after_fixed_width_who_tag,
    };

    assert!(!is_log_timestamp_token(""));
    assert!(!is_log_timestamp_token("nodot"));
    assert!(!is_log_timestamp_token("."));
    assert!(!is_log_timestamp_token("20260413.121314"));
    assert!(is_log_timestamp_token("20260413.121314.015"));
    assert_eq!(payload_after_fixed_width_who_tag("no-tag"), None);
    let prefix = format_who_tag_prefix(MALVIN_WHO);
    assert_eq!(
        payload_after_fixed_width_who_tag(&format!("{prefix}Command: x")),
        Some("Command: x")
    );
    assert_eq!(
        payload_after_fixed_width_bracket_tag(&format!("{prefix}Command: x")),
        Some("Command: x")
    );
    assert_eq!(
        payload_after_fixed_width_who_tag(&format!("{}bad", format_who_tag_delim(MALVIN_WHO))),
        Some("bad")
    );
    assert_eq!(
        payload_after_fixed_width_bracket_tag(&format!("{}bad", format_who_tag_delim(MALVIN_WHO))),
        Some("bad")
    );
    assert_eq!(payload_after_fixed_width_who_tag("no-pipe-tag"), None);
}

#[test]
fn exported_constants_match_public_contract() {
    assert_eq!(MALVIN_WHO, WHO_O);
    assert_eq!(super::WARNING_WHO, "w");
    assert_eq!(super::ERROR_WHO, "e");
    assert_eq!(format_acp_directional_tag_prefix('>', "kpop"), WHO_U);
    assert_eq!(format_acp_directional_tag_prefix('<', "kpop"), WHO_M);
}

#[test]
fn ansi_who_tag_uses_palette_for_warning_error_and_default() {
    use crate::terminal_palette::{ansi_tool_amber, ansi_tool_coral, ansi_tool_navy};

    let ts = "20260413.121314.015";
    let warn = super::format_line_with_timestamp_ansi(ts, super::WARNING_WHO, "");
    let err = super::format_line_with_timestamp_ansi(ts, super::ERROR_WHO, "");
    let default = super::format_line_with_timestamp_ansi(ts, WHO_M, "");
    assert!(warn.contains(ansi_tool_amber()));
    assert!(err.contains(ansi_tool_coral()));
    assert!(!warn.contains(ansi_tool_coral()));
    assert!(default.contains(ansi_tool_navy()));
}

#[test]
fn smoke_print_and_format_paths_cover_helpers() {
    assert_eq!(format_acp_directional_tag_prefix('>', "x"), WHO_U);
    assert_eq!(format_acp_directional_tag_prefix('<', "x"), WHO_M);
    assert!(!crate::time_format::timestamp_now_string().is_empty());
    let (max_payload, _) = super::terminal_wrap::stdout_line_wrap_meta(WHO_O, "line");
    let wrapped = super::terminal_wrap::wrap_words_bounded(max_payload, "hello world");
    assert!(!wrapped.is_empty());
    let _ = format_line("who", "body");
    init_stdout_style(true);
    print_stdout_line(WHO_U, "one");
    print_stdout_acp_tee_line(AcpTeeDirection::FromAgent, WHO_M, "two");
    print_stderr_line("e", "err");
    print_stdout_text(WHO_T, "a\nb");
    print_outgoing_prompt_log("bug_fix", "bug_fix.md");
    let mut it = super::logical_lines("x\ny");
    assert_eq!(it.next(), Some("x"));
    assert_eq!(it.next(), Some("y"));
}

#[test]
fn stdout_log_timestamps_disk_but_not_live_display() {
    let (display, log_line) = crate::output::stdout_tagged_display_and_log_line(WHO_U, "m", None);
    assert_ne!(display, log_line);
    assert!(
        !super::is_log_timestamp_token(display.split_whitespace().next().unwrap_or("")),
        "live display must omit wall-clock prefix: {display:?}"
    );
    assert!(
        super::is_log_timestamp_token(log_line.split_whitespace().next().unwrap_or("")),
        "stdout.log line must be timestamped: {log_line:?}"
    );
    assert!(log_line.contains("|m"));
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
    assert!(!super::timestamp_now_string().is_empty());
    assert!(!crate::time_format::timestamp_now_string().is_empty());
}
