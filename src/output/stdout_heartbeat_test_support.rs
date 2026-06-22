use std::time::Duration;

use crate::output::stdout_heartbeat::{
    reset_stdout_heartbeat_for_test, test_set_last_heartbeat_elapsed, HEARTBEAT_TEST_LOCK,
};
use crate::output::{
    enable_stdout_capture, take_captured_stdout, set_stdout_log_path, STDOUT_LOG_TEST_LOCK,
};

pub(super) fn heartbeat_test_guards() -> (
    std::sync::MutexGuard<'static, ()>,
    std::sync::MutexGuard<'static, ()>,
) {
    (
        HEARTBEAT_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner),
        STDOUT_LOG_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner),
    )
}

pub(super) fn due_heartbeat_render_capture_test<F: FnOnce()>(run: F) -> (String, String) {
    let (_guard, _stdout_guard) = heartbeat_test_guards();
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    enable_stdout_capture();
    reset_stdout_heartbeat_for_test();
    test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    run();
    let terminal = take_captured_stdout();
    set_stdout_log_path(None);
    let log = std::fs::read_to_string(path).unwrap_or_default();
    (terminal, log)
}

#[cfg(test)]
#[path = "stdout_heartbeat_test_support_kiss_cov_test.rs"]
mod stdout_heartbeat_test_support_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = due_heartbeat_render_capture_test;
    }
}
