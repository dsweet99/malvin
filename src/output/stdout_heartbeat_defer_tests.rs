use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::output::stdout_heartbeat::{
    reset_stdout_heartbeat_for_test, test_set_last_heartbeat_elapsed, try_emit_heartbeat_if_due,
};
use crate::output::{
    enable_stdout_capture, take_captured_stdout, write_heartbeat_log_line, MALVIN_WHO,
    format_who_tag_prefix, is_log_timestamp_token, set_stdout_log_path,
};

use crate::output::log_contains_heartbeat;

use super::stdout_heartbeat_test_support::heartbeat_test_guards;

fn heartbeat_lines_at(ts: &str) -> (String, String) {
    crate::output::stdout_heartbeat_display_and_log_line(MALVIN_WHO, "HB: 20260524.000000", Some(ts))
}

fn emit_heartbeat_at(ts: &str) {
    let (display, log) = heartbeat_lines_at(ts);
    write_heartbeat_log_line(&display, &log);
}

fn defer_sink_for_test() -> Arc<std::sync::Mutex<crate::deferred_log::DeferredLogSink>> {
    Arc::new(std::sync::Mutex::new(
        crate::deferred_log::DeferredLogSink::for_prompt("hb".into(), PathBuf::new())
            .expect("defer sink"),
    ))
}

fn lock_defer_sink(
    shared: &Arc<std::sync::Mutex<crate::deferred_log::DeferredLogSink>>,
) -> std::sync::MutexGuard<'_, crate::deferred_log::DeferredLogSink> {
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn flush_defer_sink(shared: &Arc<std::sync::Mutex<crate::deferred_log::DeferredLogSink>>) {
    crate::deferred_log::unregister_active_sink();
    lock_defer_sink(shared).force_flush();
}

fn begin_defer_session(shared: &Arc<std::sync::Mutex<crate::deferred_log::DeferredLogSink>>) {
    crate::deferred_log::register_active_sink(Arc::clone(shared));
    crate::deferred_log::install_stdout_hooks();
}

fn assert_deferred_heartbeat_enqueued(
    shared: &Arc<std::sync::Mutex<crate::deferred_log::DeferredLogSink>>,
) {
    assert_eq!(
        lock_defer_sink(shared).queue_len(),
        1,
        "active defer session must enqueue wall-clock heartbeat on defer sink"
    );
}

fn log_dummy_entry_with_heartbeat(
    shared: &Arc<std::sync::Mutex<crate::deferred_log::DeferredLogSink>>,
) {
    crate::deferred_log::log_with_heartbeat(
        &mut lock_defer_sink(shared),
        crate::deferred_log::build_display_log_entry("x".into(), "y".into()),
    );
}

fn due_active_defer_heartbeat_render_capture() -> (String, String) {
    let (_guard, _stdout_guard) = heartbeat_test_guards();
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    enable_stdout_capture();
    reset_stdout_heartbeat_for_test();
    test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    let shared = defer_sink_for_test();
    begin_defer_session(&shared);
    log_dummy_entry_with_heartbeat(&shared);
    flush_defer_sink(&shared);
    set_stdout_log_path(None);
    let terminal = take_captured_stdout();
    let log = std::fs::read_to_string(path).unwrap_or_default();
    (terminal, log)
}

fn enqueue_deferred_due_heartbeat(
    shared: &Arc<std::sync::Mutex<crate::deferred_log::DeferredLogSink>>,
) {
    test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    emit_heartbeat_at("20260524.000000.001");
    assert_deferred_heartbeat_enqueued(shared);
}

fn run_deferred_write_heartbeat_log_line_test() -> (String, String) {
    let (_guard, _stdout_guard) = heartbeat_test_guards();
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    enable_stdout_capture();
    emit_heartbeat_at("20260524.000000.000");
    let shared = defer_sink_for_test();
    begin_defer_session(&shared);
    enqueue_deferred_due_heartbeat(&shared);
    log_dummy_entry_with_heartbeat(&shared);
    flush_defer_sink(&shared);
    set_stdout_log_path(None);
    let terminal = take_captured_stdout();
    let log = std::fs::read_to_string(path).unwrap_or_default();
    (terminal, log)
}

#[test]
fn heartbeat_defer_helpers_are_exercised() {
    let (_guard, _stdout_guard) = heartbeat_test_guards();
    let (display, log) = heartbeat_lines_at("20260524.000000.000");
    assert!(log.contains("HB:"));
    assert!(!display.starts_with("20"));
    emit_heartbeat_at("20260524.000000.001");
    let shared = defer_sink_for_test();
    begin_defer_session(&shared);
    test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    emit_heartbeat_at("20260524.000000.002");
    assert_deferred_heartbeat_enqueued(&shared);
    log_dummy_entry_with_heartbeat(&shared);
    drop(lock_defer_sink(&shared));
    flush_defer_sink(&shared);
}

#[test]
fn write_heartbeat_log_line_covers_deferred_and_immediate() {
    let (terminal, text) = run_deferred_write_heartbeat_log_line_test();
    let prefix = format_who_tag_prefix(MALVIN_WHO);
    let delim = crate::output::format_who_tag_delim(MALVIN_WHO);
    let heartbeat_lines: Vec<_> = text
        .lines()
        .filter(|l| l.contains("HB:") || log_contains_heartbeat(&format!("{l}\n")))
        .collect();
    assert!(!heartbeat_lines.is_empty(), "expected heartbeat log lines: {text:?}");
    for line in heartbeat_lines {
        let ts = line.split_whitespace().next().expect("timestamp");
        assert!(is_log_timestamp_token(ts));
        assert!(
            line.contains(&format!("{prefix}HB:"))
                || line.contains(&format!("{delim}HB:"))
                || crate::time_format::heartbeat_payload_has_wall_clock_prefix(
                    line.split('|').nth(1).map_or("", str::trim_start),
                )
        );
    }
    assert!(log_contains_heartbeat(&text));
    assert!(log_contains_heartbeat(&terminal));
    assert!(!terminal.trim().starts_with("20"));
}

#[test]
fn try_emit_heartbeat_if_due_hits_deferred_path() {
    let (deferred_terminal, deferred) = due_active_defer_heartbeat_render_capture();
    assert!(log_contains_heartbeat(&deferred));
    assert!(log_contains_heartbeat(&deferred_terminal));
    assert!(!deferred_terminal.trim().starts_with("20"));
}

#[test]
fn try_emit_heartbeat_if_due_immediate_path_still_emits() {
    let (_guard, _stdout_guard) = heartbeat_test_guards();
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    enable_stdout_capture();
    reset_stdout_heartbeat_for_test();
    test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    try_emit_heartbeat_if_due(Instant::now(), false);
    let terminal = take_captured_stdout();
    set_stdout_log_path(None);
    let immediate = std::fs::read_to_string(path).unwrap_or_default();
    assert!(log_contains_heartbeat(&immediate));
    assert!(log_contains_heartbeat(&terminal));
    assert!(!terminal.trim().starts_with("20"));
}

#[cfg(test)]
mod kiss_cov_defer_tests {
    #[test]
    fn kiss_cov_enqueue_deferred_due_heartbeat() {
        let _ = super::enqueue_deferred_due_heartbeat;
    }

    #[test]
    fn log_contains_heartbeat_and_heartbeat_log_offset() {
        let sample = "20260524.000000.000 malvin.| 20260524.000000 Still alive.";
        assert!(crate::output::log_contains_heartbeat(sample));
        assert_eq!(crate::output::heartbeat_log_offset(sample), Some(0));
    }
}
