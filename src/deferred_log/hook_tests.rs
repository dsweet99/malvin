use super::{
    build_display_log_entry, defer_heartbeat_hook, defer_tagged_stdout_hook, install_stdout_hooks,
    log_with_heartbeat, register_active_sink, unregister_active_sink, DeferredLogSink, SharedDeferSink,
};
use std::path::PathBuf;
use std::sync::Arc;

fn aged_defer_shared(session: &str) -> SharedDeferSink {
    Arc::new(std::sync::Mutex::new(DeferredLogSink::new(
        session.to_string(),
        PathBuf::new(),
        super::config::DeferredLogConfig {
            max_age: std::time::Duration::from_secs(3600),
            max_drain_per_log: 64,
            cursor_dir: PathBuf::new(),
        },
    )))
}

fn arm_due_heartbeat() {
    crate::output::reset_stdout_heartbeat_for_test();
    crate::output::test_set_last_heartbeat_elapsed(std::time::Duration::from_secs(61));
}

fn begin_defer_heartbeat_session(session: &str) -> SharedDeferSink {
    let shared = aged_defer_shared(session);
    register_active_sink(Arc::clone(&shared));
    install_stdout_hooks();
    arm_due_heartbeat();
    shared
}

fn flush_heartbeat_terminal_count(
    shared: &SharedDeferSink,
    before_flush: impl FnOnce(),
) -> usize {
    let (terminal, _log) = crate::deferred_log::test_fixtures::capture_stdout_render(|| {
        before_flush();
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .force_flush();
    });
    terminal
        .lines()
        .filter(|l| {
            l.contains("HB:")
                || l.split("| ").nth(1).is_some_and(crate::time_format::heartbeat_payload_has_wall_clock_prefix)
        })
        .count()
}

fn heartbeat_display_log_line() -> (String, String) {
    crate::output::stdout_heartbeat_display_and_log_line(
        crate::output::MALVIN_WHO,
        "HB: 20260524.000000",
        Some("20260524.000000.000"),
    )
}

#[test]
fn defer_hooks_invoke_active_paths() {
    let shared = aged_defer_shared("hook");
    register_active_sink(Arc::clone(&shared));
    assert!(defer_tagged_stdout_hook("d", "l"));
    assert!(defer_heartbeat_hook("hb-d", "hb-l"));
    unregister_active_sink();
    assert!(!defer_tagged_stdout_hook("d", "l"));
    assert!(!defer_heartbeat_hook("hb-d", "hb-l"));
}

#[test]
fn log_with_heartbeat_marks_clock_when_live_terminal_published() {
    use std::time::Instant;

    let shared = aged_defer_shared("hb_clock");
    register_active_sink(Arc::clone(&shared));
    arm_due_heartbeat();
    log_with_heartbeat(
        &mut shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner),
        build_display_log_entry("tag".into(), "tag".into()),
    );
    assert!(
        crate::output::heartbeat_rendered_if_due(Instant::now(), false).is_none(),
        "interval clock must advance once bundled heartbeat is on the live terminal"
    );
    unregister_active_sink();
}

#[test]
fn write_heartbeat_then_log_with_heartbeat_enqueues_one_heartbeat() {
    let shared = begin_defer_heartbeat_session("hb_dup");
    let (display, log_line) = heartbeat_display_log_line();
    let count = flush_heartbeat_terminal_count(&shared, || {
        crate::output::write_heartbeat_log_line(&display, &log_line);
        log_with_heartbeat(
            &mut shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner),
            build_display_log_entry("tag".into(), "tag".into()),
        );
    });
    unregister_active_sink();
    assert_eq!(count, 1, "defer sink must not accumulate duplicate bundled heartbeats");
}

#[test]
fn log_with_heartbeat_pushes_due_inline_heartbeat() {
    let shared = aged_defer_shared("hb_inline");
    register_active_sink(Arc::clone(&shared));
    arm_due_heartbeat();
    log_with_heartbeat(
        &mut shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner),
        build_display_log_entry("tag".into(), "tag".into()),
    );
    assert_eq!(
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .queue_len(),
        2,
        "due inline heartbeat plus tagged entry must enqueue on defer sink"
    );
    unregister_active_sink();
}

#[test]
fn double_try_emit_while_deferred_enqueues_one_heartbeat() {
    let shared = begin_defer_heartbeat_session("double_try_emit");
    let (display, log_line) = crate::output::heartbeat_rendered_if_due(
        std::time::Instant::now(),
        false,
    )
    .expect("heartbeat due");
    let count = flush_heartbeat_terminal_count(&shared, || {
        crate::output::write_heartbeat_log_line(&display, &log_line);
        assert_eq!(
            shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .queue_len(),
            1
        );
        crate::output::write_heartbeat_log_line(&display, &log_line);
        assert_eq!(
            shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .queue_len(),
            1,
            "second try_emit must not enqueue duplicate deferred heartbeat"
        );
    });
    unregister_active_sink();
    assert_eq!(count, 1);
}

#[test]
fn unlocked_double_write_heartbeat_enqueues_one_heartbeat() {
    let shared = begin_defer_heartbeat_session("double_write_hb");
    let (display, log_line) = heartbeat_display_log_line();
    let count = flush_heartbeat_terminal_count(&shared, || {
        crate::output::write_heartbeat_log_line(&display, &log_line);
        assert_eq!(
            shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .queue_len(),
            1
        );
        crate::output::write_heartbeat_log_line(&display, &log_line);
        assert_eq!(
            shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .queue_len(),
            1,
            "unlocked double write_heartbeat must not enqueue duplicate heartbeat"
        );
    });
    unregister_active_sink();
    assert_eq!(count, 1);
}

#[test]
fn defer_log_test_ctx_builds_active_session() {
    let ctx = super::test_fixtures::defer_log_test_ctx(false);
    assert!(ctx.log_path.ends_with("stdout.log"));
    drop(ctx);
}
