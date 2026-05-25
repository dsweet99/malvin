use crate::output::stdout_render::{
emit_stdout_rendered_immediate, flush_stdout_rendered_line, print_stdout_rendered_line,
route_stdout_rendered_line, write_heartbeat_log_line, StdoutRenderPrelude,
};
use crate::output::{
enable_stdout_capture, is_log_timestamp_token, set_stdout_log_path,
stdout_tagged_display_and_log_line, take_captured_stdout, try_defer_heartbeat,
try_defer_tagged_stdout, MALVIN_WHO, STDOUT_LOG_TEST_LOCK,
};
use std::path::PathBuf;
use std::sync::Arc;

fn tagged_pair(payload: &str) -> (String, String) {
    stdout_tagged_display_and_log_line(MALVIN_WHO, payload, Some("20260524.000000.000"))
}

fn with_render_capture<F: FnOnce()>(run: F) -> (String, String) {
    let _guard = STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    enable_stdout_capture();
    run();
    let terminal = take_captured_stdout();
    set_stdout_log_path(None);
    let log = std::fs::read_to_string(path).unwrap_or_default();
    (terminal, log)
}

fn with_log<F: FnOnce()>(run: F) -> String {
    let _guard = STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    run();
    set_stdout_log_path(None);
    std::fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn immediate_emit_prints_display_not_log_on_terminal() {
    let display = "[malvin.........] hb-probe";
    let log = "20260524.000000.000 [malvin.........] hb-probe";
    let (terminal, disk) = with_render_capture(|| emit_stdout_rendered_immediate(display, log));
    assert_eq!(terminal.trim(), display);
    assert_eq!(disk.lines().next().expect("log line"), log);
    assert!(!terminal.starts_with("20"));
}

#[test]
fn heartbeat_route_prints_display_on_terminal() {
    let (display, log) = tagged_pair("heartbeat");
    let (terminal, disk) = with_render_capture(|| write_heartbeat_log_line(&display, &log));
    assert_eq!(terminal.trim(), display);
    assert!(disk.contains("heartbeat"));
    assert!(!terminal.starts_with("20"));
}

#[test]
fn flush_raw_line_with_ts_writes_log_without_defer() {
    let (terminal, disk) = with_render_capture(|| {
        crate::output::flush_stdout_raw_line_with_ts("raw-flush-probe", Some("20260524.000000.000"));
    });
    assert!(disk.contains("raw-flush-probe"));
    assert_eq!(terminal.trim(), "raw-flush-probe");
}

#[test]
fn flush_only_writes_timestamped_log_not_display_prefix() {
    let display = "[malvin.........] flush-probe";
    let log = "20260524.000000.000 [malvin.........] flush-probe";
    let (terminal, disk) = with_render_capture(|| flush_stdout_rendered_line(display, log));
    assert_eq!(disk.lines().next().expect("log line"), log);
    assert_eq!(terminal.trim(), display);
    let ts = log.split_whitespace().next().expect("timestamp");
    assert!(is_log_timestamp_token(ts));
    assert!(!terminal.starts_with("20"));
}

#[test]
fn tagged_route_writes_immediate_log_when_no_defer() {
    let (display, log) = tagged_pair("tagged-route");
    let (terminal, disk) = with_render_capture(|| print_stdout_rendered_line(&display, &log));
    assert!(disk.contains("tagged-route"));
    assert!(is_log_timestamp_token(
        disk.lines().next().unwrap().split_whitespace().next().unwrap(),
    ));
    assert!(!terminal.starts_with("20"));
}

#[test]
fn heartbeat_route_writes_immediate_log_when_no_defer() {
    let (display, log) = tagged_pair("heartbeat");
    let (terminal, disk) = with_render_capture(|| write_heartbeat_log_line(&display, &log));
    assert!(disk.contains("heartbeat"));
    assert!(terminal.contains("heartbeat"));
    assert!(!terminal.starts_with("20"));
}

#[test]
fn heartbeat_route_respects_stdout_color_gate() {
    use std::time::Instant;

    crate::output::init_stdout_style(true);
    let (terminal, _disk) =
        super::stdout_heartbeat_test_support::due_heartbeat_render_capture_test(|| {
            crate::output::stdout_heartbeat::try_emit_heartbeat_if_due(Instant::now(), false);
        });
    assert!(!terminal.starts_with("20"));
    if crate::output::stdout_use_color() {
        assert!(terminal.contains('\x1b'));
    }
}

#[test]
fn route_all_preludes_emit_when_defer_inactive() {
    let (display, log) = tagged_pair("prelude-probe");
    let (terminal, disk) = with_render_capture(|| {
        route_stdout_rendered_line(&display, &log, StdoutRenderPrelude::FlushOnly);
        route_stdout_rendered_line(&display, &log, StdoutRenderPrelude::HeartbeatOnly);
        route_stdout_rendered_line(&display, &log, StdoutRenderPrelude::TaggedWithHeartbeat);
    });
    assert_eq!(disk.lines().count(), 3);
    assert_eq!(terminal.lines().count(), 3);
    assert!(!terminal.starts_with("20"));
}

#[test]
fn defer_hooks_capture_tagged_and_heartbeat_routes() {
    let text = with_log(|| {
        let shared = Arc::new(std::sync::Mutex::new(
            crate::deferred_log::DeferredLogSink::for_prompt(
                "render_hook".to_string(),
                PathBuf::new(),
            )
            .expect("defer sink"),
        ));
        crate::deferred_log::register_active_sink(Arc::clone(&shared));
        crate::deferred_log::install_stdout_hooks();
        let (display, log) = tagged_pair("defer-capture");
        assert!(try_defer_tagged_stdout(&display, &log));
        assert!(try_defer_heartbeat(&display, &log));
    });
    assert!(text.is_empty(), "defer must suppress immediate log write");
    crate::deferred_log::unregister_active_sink();
}

#[test]
fn tagged_route_defers_when_session_active() {
    let (display, log) = tagged_pair("tag-defer");
    let shared = Arc::new(std::sync::Mutex::new(
        crate::deferred_log::DeferredLogSink::for_prompt(
            "render_tag".to_string(),
            PathBuf::new(),
        )
        .expect("defer sink"),
    ));
    let (terminal, disk) = with_render_capture(|| {
        crate::deferred_log::register_active_sink(Arc::clone(&shared));
        crate::deferred_log::install_stdout_hooks();
        print_stdout_rendered_line(&display, &log);
        crate::deferred_log::unregister_active_sink();
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .force_flush();
    });
    assert!(disk.contains("tag-defer"));
    assert_eq!(terminal.trim(), display);
    assert!(!terminal.starts_with("20"));
}

#[test]
fn heartbeat_route_defers_when_session_active() {
    let (display, log) = tagged_pair("heartbeat");
    let shared = Arc::new(std::sync::Mutex::new(
        crate::deferred_log::DeferredLogSink::for_prompt(
            "render_hb".to_string(),
            PathBuf::new(),
        )
        .expect("defer sink"),
    ));
    let (terminal, disk) = with_render_capture(|| {
        crate::deferred_log::register_active_sink(Arc::clone(&shared));
        crate::deferred_log::install_stdout_hooks();
        write_heartbeat_log_line(&display, &log);
        crate::deferred_log::unregister_active_sink();
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .force_flush();
    });
    assert!(disk.contains("heartbeat"));
    assert_eq!(terminal.trim(), display);
    assert!(!terminal.starts_with("20"));
}

#[test]
fn heartbeat_route_defers_then_flush_preserves_split() {
    let (display, log) = tagged_pair("heartbeat");
    let shared = Arc::new(std::sync::Mutex::new(
        crate::deferred_log::DeferredLogSink::for_prompt(
            "render_hb_flush".to_string(),
            PathBuf::new(),
        )
        .expect("defer sink"),
    ));
    let (terminal, disk) = with_render_capture(|| {
        crate::deferred_log::register_active_sink(Arc::clone(&shared));
        crate::deferred_log::install_stdout_hooks();
        write_heartbeat_log_line(&display, &log);
        crate::deferred_log::unregister_active_sink();
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .force_flush();
    });
    assert_eq!(terminal.trim(), display);
    assert_eq!(disk.lines().next().expect("log line"), log);
    assert!(!terminal.starts_with("20"));
}

#[test]
fn emit_without_log_path_skips_disk_append() {
    let _guard = STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(None);
    crate::output::enable_stdout_capture();
    emit_stdout_rendered_immediate("[probe] x", "20260524.000000.000 [probe] x");
    let terminal = crate::output::take_captured_stdout();
    assert_eq!(terminal.trim(), "[probe] x");
    crate::output::set_stdout_log_path(Some(path.clone()));
    emit_stdout_rendered_immediate("[probe] y", "20260524.000000.000 [probe] y");
    crate::output::set_stdout_log_path(None);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    assert!(text.contains("[probe] y"));
}
