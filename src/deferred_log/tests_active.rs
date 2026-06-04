use std::sync::Arc;
use std::time::Duration;

use super::test_fixtures::{
    aged_defer_shared, defer_log_test_ctx, zero_age_defer_shared, DeferLogTestCtx,
};
use super::{
    build_display_log_entry, install_stdout_hooks,
    log_with_heartbeat, register_active_sink, unregister_active_sink, DeferredLogSink,
};

fn push_tagged_entry(shared: &Arc<std::sync::Mutex<DeferredLogSink>>, display: &str, log: &str) {
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push_entry(build_display_log_entry(display.to_string(), log.to_string()));
}

fn sink_queue_len(shared: &Arc<std::sync::Mutex<DeferredLogSink>>) -> usize {
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .queue_len()
}

fn finish_defer_log_test(ctx: DeferLogTestCtx) -> String {
    flush_active_sink(ctx.shared);
    crate::output::set_stdout_log_path(None);
    std::fs::read_to_string(ctx.log_path).unwrap_or_default()
}

fn flush_active_sink(shared: Arc<std::sync::Mutex<DeferredLogSink>>) {
    unregister_active_sink();
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .force_flush();
}

fn exercise_active_defer_hooks(shared: &Arc<std::sync::Mutex<DeferredLogSink>>) {
    let (display, log) =
        crate::output::stdout_tagged_display_and_log_line("malvin", "defer probe", None);
    assert!(crate::output::try_defer_tagged_stdout(&display, &log));
    log_with_heartbeat(
        &mut shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        build_display_log_entry("d".to_string(), "l".to_string()),
    );
    crate::output::test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    let (display, log) = crate::output::heartbeat_rendered_if_due(
        std::time::Instant::now(),
        false,
    )
    .expect("heartbeat due");
    assert!(crate::output::try_defer_heartbeat(&display, &log));
}

#[test]
fn active_sink_routes_stdout_and_heartbeats() {
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_stdout_heartbeat_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let log_path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(log_path.clone()));
    let shared = zero_age_defer_shared("sess");
    register_active_sink(Arc::clone(&shared));
    install_stdout_hooks();
    exercise_active_defer_hooks(&shared);
    flush_active_sink(shared);
    assert!(!crate::output::try_defer_tagged_stdout("x", "y"));
    crate::output::set_stdout_log_path(None);
    let text = std::fs::read_to_string(log_path).unwrap_or_default();
    assert!(text.contains("defer probe"));
}

#[test]
fn wall_clock_poller_skips_defer_sink_while_session_active() {
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _heartbeat_guard = crate::output::HEARTBEAT_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_stdout_heartbeat_for_test();
    let shared = aged_defer_shared("sess");
    register_active_sink(Arc::clone(&shared));
    install_stdout_hooks();
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push_entry(build_display_log_entry(
            "queued".to_string(),
            "log".to_string(),
        ));
    assert_eq!(
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .queue_len(),
        1
    );
    crate::output::test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    crate::output::poll_wall_clock_heartbeat_if_due();
    assert_eq!(
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .queue_len(),
        2,
        "wall-clock heartbeat must enqueue after existing deferred entries (FIFO)"
    );
    flush_active_sink(shared);
}

fn silence_heartbeat_log_under_active_defer() -> String {
    let ctx = defer_log_test_ctx(true);
    crate::output::test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    crate::output::poll_wall_clock_heartbeat_if_due();
    finish_defer_log_test(ctx)
}

#[test]
fn wall_clock_heartbeat_log_order_follows_defer_queue_fifo() {
    let text = {
        let ctx = defer_log_test_ctx(true);
        push_tagged_entry(&ctx.shared, "QUEUED_FIRST", "QUEUED_FIRST");
        assert_eq!(sink_queue_len(&ctx.shared), 1);
        crate::output::test_set_last_heartbeat_elapsed(Duration::from_secs(61));
        crate::output::poll_wall_clock_heartbeat_if_due();
        finish_defer_log_test(ctx)
    };
    let queued = text.find("QUEUED_FIRST").expect("queued defer entry in stdout.log");
    let heartbeat = crate::output::heartbeat_log_offset(&text).expect("heartbeat in stdout.log");
    assert!(
        queued < heartbeat,
        "plan FIFO: heartbeat must not land in stdout.log before older deferred entries; log={text:?}"
    );
}

#[test]
fn active_defer_session_emits_heartbeat_during_stdout_silence() {
    let text = silence_heartbeat_log_under_active_defer();
    assert!(
        crate::output::log_contains_heartbeat(&text),
        "plan phase 4: heartbeats must still appear during stdout silence while defer session is active"
    );
}

#[test]
fn active_defer_session_shows_heartbeat_on_live_terminal_before_log_flush() {
    let ctx = defer_log_test_ctx(true);
    crate::output::enable_stdout_capture();
    crate::output::test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    crate::output::poll_wall_clock_heartbeat_if_due();
    let terminal = crate::output::take_captured_stdout();
    let log_before_flush =
        std::fs::read_to_string(&ctx.log_path).unwrap_or_default();
    assert!(
        crate::output::log_contains_heartbeat(&terminal),
        "heartbeat must reach live terminal while defer session blocks log drain; terminal={terminal:?}"
    );
    assert!(
        !crate::output::log_contains_heartbeat(&log_before_flush),
        "heartbeat log line must stay queued until flush; log={log_before_flush:?}"
    );
    finish_defer_log_test(ctx);
}

#[cfg(test)]
mod kiss_cov_active_tests {
    use crate::deferred_log::test_fixtures::defer_log_test_ctx;

    #[test]
    fn defer_log_test_ctx_creates_guarded_log_path() {
        let ctx = defer_log_test_ctx(true);
        assert!(ctx.log_path.ends_with("stdout.log"));
        drop(ctx);
    }
}
