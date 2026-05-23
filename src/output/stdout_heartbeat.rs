use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::{MALVIN_WHO, print_stdout_line, timestamp_now_string};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(60);

#[cfg(test)]
const HEARTBEAT_POLL_INTERVAL: Duration = Duration::from_millis(10);
#[cfg(not(test))]
const HEARTBEAT_POLL_INTERVAL: Duration = Duration::from_secs(1);

static LAST_HEARTBEAT: Mutex<Option<Instant>> = Mutex::new(None);
static WALL_CLOCK_POLLER: OnceLock<()> = OnceLock::new();

#[cfg(test)]
pub(crate) static HEARTBEAT_TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn emit_heartbeat_line() {
    print_stdout_line(MALVIN_WHO, &timestamp_now_string());
}

pub(crate) fn heartbeat_due(last: Instant, now: Instant) -> bool {
    now.checked_duration_since(last)
        .is_some_and(|elapsed| elapsed >= HEARTBEAT_INTERVAL)
}

pub(crate) fn try_emit_heartbeat_if_due(now: Instant, arm_if_unarmed: bool) {
    let mut guard = LAST_HEARTBEAT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match *guard {
        None if arm_if_unarmed => {
            *guard = Some(now);
        }
        None => {}
        Some(last) if heartbeat_due(last, now) => {
            *guard = Some(now);
            drop(guard);
            emit_heartbeat_line();
        }
        Some(_) => {}
    }
}

pub(crate) fn maybe_emit_stdout_heartbeat() {
    try_emit_heartbeat_if_due(Instant::now(), true);
}

pub(crate) fn poll_wall_clock_heartbeat_if_due() {
    try_emit_heartbeat_if_due(Instant::now(), false);
}

pub(crate) fn wall_clock_poller_loop() {
    loop {
        std::thread::sleep(HEARTBEAT_POLL_INTERVAL);
        poll_wall_clock_heartbeat_if_due();
    }
}

pub(crate) fn spawn_wall_clock_poller_if_needed() {
    WALL_CLOCK_POLLER.get_or_init(|| {
        std::thread::spawn(wall_clock_poller_loop);
    });
}

#[cfg(test)]
pub(crate) fn reset_stdout_heartbeat_for_test() {
    *LAST_HEARTBEAT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

#[cfg(test)]
pub(crate) fn test_set_last_heartbeat_elapsed(elapsed: Duration) {
    let last = Instant::now()
        .checked_sub(elapsed)
        .expect("elapsed heartbeat offset must not exceed now");
    *LAST_HEARTBEAT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(last);
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, Instant};

    use super::{
        HEARTBEAT_TEST_LOCK, emit_heartbeat_line, heartbeat_due, maybe_emit_stdout_heartbeat,
        poll_wall_clock_heartbeat_if_due, reset_stdout_heartbeat_for_test,
        test_set_last_heartbeat_elapsed, try_emit_heartbeat_if_due, wall_clock_poller_loop,
    };
    use crate::output::{
        MALVIN_WHO, format_log_tag_inner, init_stdout_style, is_log_timestamp_token,
        print_stdout_line, set_stdout_log_path,
    };

    #[test]
    fn heartbeat_helpers_smoke() {
        let now = Instant::now();
        assert!(!heartbeat_due(now, now));
        let _ = try_emit_heartbeat_if_due;
        let _ = poll_wall_clock_heartbeat_if_due;
        let _ = wall_clock_poller_loop;
    }

    #[test]
    fn heartbeat_log_line_routes_timestamp_through_logger_payload() {
        let _guard = HEARTBEAT_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("stdout.log");
        set_stdout_log_path(Some(path.clone()));
        emit_heartbeat_line();
        set_stdout_log_path(None);
        let text = std::fs::read_to_string(&path).expect("read");
        let inner = format_log_tag_inner(MALVIN_WHO);
        let line = text.lines().next().expect("heartbeat line");
        assert!(
            is_log_timestamp_token(line.split_whitespace().next().unwrap_or("")),
            "stdout.log heartbeat line must start with wall-clock prefix; got {line:?}"
        );
        let payload = line
            .split_once(&format!("[{inner}] "))
            .map_or("", |(_, rest)| rest);
        assert!(
            is_log_timestamp_token(payload),
            "heartbeat log payload must remain the wall-clock token; got: {payload:?}"
        );
    }

    #[test]
    fn heartbeat_emits_once_when_interval_not_elapsed() {
        let _guard = HEARTBEAT_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        reset_stdout_heartbeat_for_test();
        test_set_last_heartbeat_elapsed(Duration::from_secs(61));
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("stdout.log");
        set_stdout_log_path(Some(path.clone()));
        maybe_emit_stdout_heartbeat();
        maybe_emit_stdout_heartbeat();
        set_stdout_log_path(None);
        let text = std::fs::read_to_string(path).expect("read");
        assert_eq!(
            text.matches('[').count(),
            1,
            "expected one heartbeat: {text:?}"
        );
    }

    #[test]
    fn first_tagged_stdout_line_is_not_preceded_by_immediate_heartbeat() {
        let _guard = HEARTBEAT_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        reset_stdout_heartbeat_for_test();
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("stdout.log");
        set_stdout_log_path(Some(path.clone()));
        print_stdout_line("u", "payload");
        set_stdout_log_path(None);
        let text = std::fs::read_to_string(path).expect("read");
        assert!(
            !text.contains(&format!("[{MALVIN_WHO}]")),
            "first stdout line should not be preceded by an immediate heartbeat: {text:?}"
        );
        assert!(
            text.contains("] payload"),
            "expected tagged payload in log: {text:?}"
        );
    }

    #[test]
    fn heartbeat_emits_during_stdout_silence_when_interval_elapsed() {
        let _guard = HEARTBEAT_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        reset_stdout_heartbeat_for_test();
        test_set_last_heartbeat_elapsed(Duration::from_secs(61));
        init_stdout_style(true);
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("stdout.log");
        set_stdout_log_path(Some(path.clone()));
        thread::sleep(Duration::from_millis(50));
        set_stdout_log_path(None);
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let inner = format_log_tag_inner(MALVIN_WHO);
        assert!(
            text.contains(&format!("[{inner}] ")),
            "expected wall-clock heartbeat during stdout silence: {text:?}"
        );
    }
}
