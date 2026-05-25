use std::time::Instant;

use crate::output::stdout_heartbeat::{
    maybe_emit_stdout_heartbeat, poll_wall_clock_heartbeat_if_due, reset_stdout_heartbeat_for_test,
    try_emit_heartbeat_if_due,
};
use crate::output::{
    MALVIN_WHO, format_log_tag_inner, init_stdout_style, is_log_timestamp_token, print_stdout_line,
    set_stdout_log_path,
};

use super::stdout_heartbeat_test_support::{
    due_heartbeat_render_capture_test, heartbeat_test_guards,
};

#[test]
fn heartbeat_log_line_uses_logger_timestamp_only() {
    let (terminal, text) = due_heartbeat_render_capture_test(|| {
        try_emit_heartbeat_if_due(Instant::now(), false);
    });
    let inner = format_log_tag_inner(MALVIN_WHO);
    let line = text.lines().next().expect("heartbeat line");
    assert!(is_log_timestamp_token(line.split_whitespace().next().unwrap_or("")));
    let payload = line
        .split_once(&format!("[{inner}] "))
        .map_or("", |(_, rest)| rest);
    assert!(payload.starts_with("HB: "));
    assert!(terminal.contains(&format!("[{inner}] {payload}")));
    assert!(!terminal.trim().starts_with("20"));
    assert!(!terminal.is_empty());
}

#[test]
fn heartbeat_emits_once_when_interval_not_elapsed() {
    let (terminal, text) = due_heartbeat_render_capture_test(|| {
        maybe_emit_stdout_heartbeat();
        maybe_emit_stdout_heartbeat();
    });
    assert_eq!(text.matches('[').count(), 1, "expected one heartbeat: {text:?}");
    assert_eq!(terminal.matches('[').count(), 1, "expected one heartbeat: {terminal:?}");
    assert!(!terminal.trim().starts_with("20"));
}

#[test]
fn due_heartbeat_terminal_uses_color_without_wall_clock_prefix() {
    init_stdout_style(true);
    let (terminal, text) = due_heartbeat_render_capture_test(|| {
        try_emit_heartbeat_if_due(Instant::now(), false);
    });
    assert!(text.contains("HB:"));
    assert!(terminal.contains("HB:"));
    assert!(!terminal.trim().starts_with("20"));
    if crate::output::stdout_use_color() {
        assert!(terminal.contains('\x1b'));
    }
}

#[test]
fn first_tagged_stdout_line_is_not_preceded_by_immediate_heartbeat() {
    let (_guard, _stdout_guard) = heartbeat_test_guards();
    reset_stdout_heartbeat_for_test();
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    print_stdout_line("u", "payload");
    set_stdout_log_path(None);
    let text = std::fs::read_to_string(path).expect("read");
    assert!(!text.contains(&format!("[{MALVIN_WHO}]")));
    assert!(text.contains("] payload"));
}

#[test]
fn try_emit_heartbeat_if_due_immediate_when_no_active_sink() {
    let (terminal, text) = due_heartbeat_render_capture_test(|| {
        try_emit_heartbeat_if_due(Instant::now(), false);
    });
    assert!(text.contains("HB:"));
    assert!(terminal.contains("HB:"));
    assert!(!terminal.trim().starts_with("20"));
}

#[test]
fn heartbeat_logs_during_stdout_silence_when_interval_elapsed() {
    init_stdout_style(true);
    crate::output::stdout_heartbeat::spawn_wall_clock_poller_if_needed();
    let (terminal, text) = due_heartbeat_render_capture_test(|| {
        poll_wall_clock_heartbeat_if_due();
    });
    assert!(text.contains("HB:"));
    assert!(terminal.contains("HB:"));
    assert!(!terminal.trim().starts_with("20"));
}
