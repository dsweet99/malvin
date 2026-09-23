use crate::output::stdout_render::{
    StdoutRenderPrelude, emit_stdout_rendered_immediate, print_stdout_rendered_line,
    route_stdout_rendered_line, write_heartbeat_log_line,
};
use crate::output::{
    MALVIN_WHO, STDOUT_LOG_TEST_LOCK, enable_stdout_capture, is_log_timestamp_token,
    set_stdout_log_path, stdout_heartbeat_display_and_log_line, stdout_tagged_display_and_log_line,
    take_captured_stdout,
};

fn tagged_pair(payload: &str) -> (String, String) {
    stdout_tagged_display_and_log_line(MALVIN_WHO, payload, Some("20260524.000000.000"))
}

fn heartbeat_pair(suffix: &str) -> (String, String) {
    stdout_heartbeat_display_and_log_line(
        MALVIN_WHO,
        &format!("20260524.000000 {suffix}"),
        Some("20260524.000000.000"),
    )
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

fn immediate_emit_prints_display_not_log_on_terminal() {
    let display = "malvin.| hb-probe";
    let log = "20260524.000000.000 malvin.|hb-probe";
    let (terminal, disk) = with_render_capture(|| emit_stdout_rendered_immediate(display, log));
    assert_eq!(terminal.trim(), display);
    assert_eq!(disk.lines().next().expect("log line"), log);
    assert!(!terminal.starts_with("20"));
}

fn heartbeat_route_prints_display_on_terminal() {
    let (display, log) = heartbeat_pair("heartbeat");
    let (terminal, disk) = with_render_capture(|| write_heartbeat_log_line(&display, &log));
    assert_eq!(terminal.trim(), display);
    assert!(disk.contains("heartbeat"));
    assert!(!terminal.starts_with("20"));
}

fn flush_raw_line_with_ts_writes_log_without_defer() {
    let (display, log) = crate::output::stdout_log_pair::stdout_raw_display_and_log_line(
        "raw-flush-probe",
        Some("20260524.000000.000"),
    );
    let (terminal, disk) = with_render_capture(|| {
        route_stdout_rendered_line(&display, &log, StdoutRenderPrelude::FlushOnly);
    });
    assert!(disk.contains("raw-flush-probe"));
    assert_eq!(terminal.trim(), "raw-flush-probe");
}

fn flush_only_writes_timestamped_log_not_display_prefix() {
    let display = "malvin.| flush-probe";
    let log = "20260524.000000.000 malvin.|flush-probe";
    let (terminal, disk) = with_render_capture(|| {
        route_stdout_rendered_line(display, log, StdoutRenderPrelude::FlushOnly);
    });
    assert_eq!(disk.lines().next().expect("log line"), log);
    assert_eq!(terminal.trim(), display);
    let ts = log.split_whitespace().next().expect("timestamp");
    assert!(is_log_timestamp_token(ts));
    assert!(!terminal.starts_with("20"));
}

fn tagged_route_writes_immediate_log_when_no_defer() {
    let (display, log) = tagged_pair("tagged-route");
    let (terminal, disk) = with_render_capture(|| print_stdout_rendered_line(&display, &log));
    assert!(disk.contains("tagged-route"));
    assert!(is_log_timestamp_token(
        disk.lines()
            .next()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap(),
    ));
    assert!(!terminal.starts_with("20"));
}

fn heartbeat_route_writes_immediate_log_when_no_defer() {
    let (display, log) = heartbeat_pair("heartbeat");
    let (terminal, disk) = with_render_capture(|| write_heartbeat_log_line(&display, &log));
    assert!(disk.contains("heartbeat"));
    assert!(terminal.contains("heartbeat"));
    assert!(!terminal.starts_with("20"));
}

fn heartbeat_route_respects_stdout_color_gate() {
    use std::time::Instant;

    crate::output::init_stdout_style_for_test(false);
    let (terminal, _disk) =
        super::stdout_heartbeat_test_support::due_heartbeat_render_capture_test(|| {
            crate::output::stdout_heartbeat::try_emit_heartbeat_if_due(Instant::now(), false);
        });
    assert!(!terminal.starts_with("20"));
    if crate::output::stdout_use_color() {
        assert!(terminal.contains('\x1b'));
    }
}

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
fn kiss_bundled_output_stdout_render_tests() {
    immediate_emit_prints_display_not_log_on_terminal();
    heartbeat_route_prints_display_on_terminal();
    flush_raw_line_with_ts_writes_log_without_defer();
    flush_only_writes_timestamped_log_not_display_prefix();
    tagged_route_writes_immediate_log_when_no_defer();
    heartbeat_route_writes_immediate_log_when_no_defer();
    heartbeat_route_respects_stdout_color_gate();
    route_all_preludes_emit_when_defer_inactive();
}
