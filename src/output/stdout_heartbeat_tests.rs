use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::output::stdout_heartbeat::{
    emit_heartbeat_line, heartbeat_due, heartbeat_log_line_for_defer_sink,
    maybe_emit_stdout_heartbeat, poll_wall_clock_heartbeat_if_due, reset_stdout_heartbeat_for_test,
    test_set_last_heartbeat_elapsed, try_emit_heartbeat_if_due, wall_clock_poller_loop,
    write_heartbeat_log_line, HEARTBEAT_TEST_LOCK,
};
use crate::output::{
    MALVIN_WHO, format_log_tag_inner, init_stdout_style, is_log_timestamp_token,
    print_stdout_line, set_stdout_log_path, STDOUT_LOG_TEST_LOCK,
};

fn heartbeat_test_guards() -> (
    std::sync::MutexGuard<'static, ()>,
    std::sync::MutexGuard<'static, ()>,
) {
    let guard = HEARTBEAT_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let stdout_guard = STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    (guard, stdout_guard)
}

fn prompt_defer_sink() -> Arc<std::sync::Mutex<crate::deferred_log::DeferredLogSink>> {
    Arc::new(std::sync::Mutex::new(
        crate::deferred_log::DeferredLogSink::for_prompt("hb".to_string(), PathBuf::new())
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

fn register_active_defer_session(
    shared: &Arc<std::sync::Mutex<crate::deferred_log::DeferredLogSink>>,
) {
    crate::deferred_log::register_active_sink(Arc::clone(shared));
    crate::deferred_log::install_stdout_hooks();
}

fn log_inline_heartbeat_through_active_sink(
    shared: &Arc<std::sync::Mutex<crate::deferred_log::DeferredLogSink>>,
) {
    crate::deferred_log::log_with_heartbeat(
        &mut lock_defer_sink(shared),
        crate::deferred_log::build_tagged_stdout_entry("x".to_string(), "y".to_string()),
    );
}

fn due_active_defer_heartbeat_log() -> String {
    due_heartbeat_emit_test(|| {
        let shared = prompt_defer_sink();
        register_active_defer_session(&shared);
        log_inline_heartbeat_through_active_sink(&shared);
        flush_registered_defer_sink(shared);
    })
}

fn flush_registered_defer_sink(
    shared: Arc<std::sync::Mutex<crate::deferred_log::DeferredLogSink>>,
) -> String {
    crate::deferred_log::unregister_active_sink();
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .force_flush();
    String::new()
}

#[test]
fn heartbeat_helpers_smoke() {
    let now = Instant::now();
    assert!(!heartbeat_due(now, now));
    let _ = try_emit_heartbeat_if_due;
    let _ = poll_wall_clock_heartbeat_if_due;
    let _ = wall_clock_poller_loop;
}

#[test]
fn heartbeat_log_line_uses_logger_timestamp_only() {
    let (_guard, _stdout_guard) = heartbeat_test_guards();
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    emit_heartbeat_line();
    set_stdout_log_path(None);
    let text = std::fs::read_to_string(&path).expect("read");
    let inner = format_log_tag_inner(MALVIN_WHO);
    let line = text.lines().next().expect("heartbeat line");
    assert!(is_log_timestamp_token(line.split_whitespace().next().unwrap_or("")));
    let payload = line
        .split_once(&format!("[{inner}] "))
        .map_or("", |(_, rest)| rest);
    assert_eq!(payload, "heartbeat");
}

#[test]
fn heartbeat_emits_once_when_interval_not_elapsed() {
    let (_guard, _stdout_guard) = heartbeat_test_guards();
    reset_stdout_heartbeat_for_test();
    test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    maybe_emit_stdout_heartbeat();
    maybe_emit_stdout_heartbeat();
    set_stdout_log_path(None);
    let text = std::fs::read_to_string(path).expect("read");
    assert_eq!(text.matches('[').count(), 1, "expected one heartbeat: {text:?}");
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
fn heartbeat_log_line_if_due_covers_arm_and_due_paths() {
    reset_stdout_heartbeat_for_test();
    assert!(heartbeat_log_line_for_defer_sink(Instant::now(), false).is_none());
    assert!(heartbeat_log_line_for_defer_sink(Instant::now(), true).is_none());
    assert!(heartbeat_log_line_for_defer_sink(Instant::now(), false).is_none());
    test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    assert!(heartbeat_log_line_for_defer_sink(Instant::now(), false).is_some());
}

#[test]
fn write_heartbeat_log_line_covers_deferred_and_immediate() {
    let (_guard, _stdout_guard) = heartbeat_test_guards();
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    write_heartbeat_log_line("20260524.000000.000 [malvin........] heartbeat");
    let shared = prompt_defer_sink();
    register_active_defer_session(&shared);
    test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    write_heartbeat_log_line("20260524.000000.001 [malvin........] heartbeat");
    assert_eq!(
        lock_defer_sink(&shared).queue_len(),
        1,
        "active defer session must enqueue wall-clock heartbeat on defer sink"
    );
    log_inline_heartbeat_through_active_sink(&shared);
    flush_registered_defer_sink(shared);
    set_stdout_log_path(None);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    assert!(text.contains("heartbeat"));
}

fn due_heartbeat_emit_test<F: FnOnce()>(run: F) -> String {
    let (_guard, _stdout_guard) = heartbeat_test_guards();
    reset_stdout_heartbeat_for_test();
    test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    run();
    set_stdout_log_path(None);
    std::fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn try_emit_heartbeat_if_due_hits_deferred_and_immediate_paths() {
    let immediate = due_heartbeat_emit_test(|| {
        try_emit_heartbeat_if_due(Instant::now(), false);
    });
    assert!(immediate.contains("heartbeat"));
    let deferred = due_active_defer_heartbeat_log();
    assert!(deferred.contains("heartbeat"));
}

#[test]
fn emit_heartbeat_line_immediate_when_no_active_sink() {
    let (_guard, _stdout_guard) = heartbeat_test_guards();
    reset_stdout_heartbeat_for_test();
    test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    emit_heartbeat_line();
    set_stdout_log_path(None);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    assert!(text.contains("heartbeat"));
}

#[test]
fn deferred_heartbeat_routes_through_active_sink() {
    let text = due_active_defer_heartbeat_log();
    assert!(text.contains("heartbeat"));
}

#[test]
fn heartbeat_logs_during_stdout_silence_when_interval_elapsed() {
    let (_guard, _stdout_guard) = heartbeat_test_guards();
    reset_stdout_heartbeat_for_test();
    test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    init_stdout_style(true);
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    poll_wall_clock_heartbeat_if_due();
    set_stdout_log_path(None);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let inner = format_log_tag_inner(MALVIN_WHO);
    assert!(text.contains(&format!("[{inner}] heartbeat")));
}
