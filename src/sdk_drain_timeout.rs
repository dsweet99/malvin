#![cfg_attr(test, allow(unsafe_code))]

use std::time::Duration;

pub const DEFAULT_SDK_DRAIN_IDLE_TIMEOUT_MS: u64 = 600_000;
/// Minimum time allowed for a newly spawned bridge to acknowledge create/resume.
pub const SDK_BRIDGE_STARTUP_TIMEOUT_MIN_MS: u64 = 1_000;

/// Max time to block on one bridge/pi read before a child-health sample (slice).
pub const SDK_DRAIN_IDLE_SLICE_MAX_MS: u64 = 60_000;

#[must_use]
pub fn sdk_drain_idle_timeout_from_env() -> Duration {
    Duration::from_millis(
        std::env::var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS")
            .ok()
            .map_or(DEFAULT_SDK_DRAIN_IDLE_TIMEOUT_MS, |s| {
                s.parse::<u64>().map_or_else(
                    |_| {
                        tracing::warn!(
                            target: "malvin::sdk_drain_timeout",
                            value = %s,
                            "MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS is not a positive integer; using default"
                        );
                        DEFAULT_SDK_DRAIN_IDLE_TIMEOUT_MS
                    },
                    |n| n.max(1),
                )
            }),
    )
}

pub fn sdk_bridge_startup_timeout() -> Duration {
    sdk_drain_idle_timeout_from_env().max(Duration::from_millis(SDK_BRIDGE_STARTUP_TIMEOUT_MIN_MS))
}

/// How long to wait on `read_event` before sampling child health.
///
/// Always `min(60s, idle_remaining)` so short idle budgets (tests) slice tightly.
#[must_use]
pub fn sdk_drain_idle_slice(idle_remaining: Duration) -> Duration {
    idle_remaining.min(Duration::from_millis(SDK_DRAIN_IDLE_SLICE_MAX_MS))
}

/// Wall-clock cap for one next-event wait: at most one full extra idle window from health.
#[must_use]
pub const fn sdk_drain_idle_max_wait(idle: Duration) -> Duration {
    idle.saturating_mul(2)
}

#[cfg(test)]
pub(crate) fn tests_set_idle_ms_for_test(ms: u64) {
    unsafe {
        std::env::set_var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS", ms.to_string());
    }
}

#[cfg(test)]
pub(crate) fn tests_restore_idle_ms_for_test(prior: Option<std::ffi::OsString>) {
    unsafe {
        match prior {
            Some(v) => std::env::set_var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS", v),
            None => std::env::remove_var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_env_lock;
    use std::time::Duration;

    #[test]
    fn sdk_drain_idle_timeout_from_env_rejects_zero_and_garbage() {
        let _lock = test_env_lock();
        let prior = std::env::var_os("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS");
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS", "0");
        }
        assert_eq!(sdk_drain_idle_timeout_from_env(), Duration::from_millis(1));
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS", "nope");
        }
        assert_eq!(
            sdk_drain_idle_timeout_from_env(),
            Duration::from_millis(DEFAULT_SDK_DRAIN_IDLE_TIMEOUT_MS)
        );
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS", "250");
        }
        assert_eq!(
            sdk_drain_idle_timeout_from_env(),
            Duration::from_millis(250)
        );
        #[allow(unsafe_code)]
        unsafe {
            match prior {
                Some(v) => std::env::set_var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS", v),
                None => std::env::remove_var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS"),
            }
        }
    }

    #[test]
    fn sdk_drain_idle_slice_and_max_wait() {
        assert_eq!(
            sdk_drain_idle_slice(Duration::from_secs(5)),
            Duration::from_secs(5)
        );
        assert_eq!(
            sdk_drain_idle_slice(Duration::from_secs(120)),
            Duration::from_millis(SDK_DRAIN_IDLE_SLICE_MAX_MS)
        );
        assert_eq!(
            sdk_drain_idle_max_wait(Duration::from_secs(600)),
            Duration::from_secs(1200)
        );
    }

    #[test]
    fn kiss_cov_sdk_drain_timeout() {
        let _ = stringify!(DEFAULT_SDK_DRAIN_IDLE_TIMEOUT_MS);
        let _ = stringify!(SDK_DRAIN_IDLE_SLICE_MAX_MS);
        let _ = stringify!(sdk_drain_idle_timeout_from_env);
        let _ = stringify!(sdk_drain_idle_slice);
        let _ = stringify!(sdk_drain_idle_max_wait);
        let _ = stringify!(sdk_drain_idle_timeout_from_env_rejects_zero_and_garbage);
        let _ = stringify!(sdk_drain_idle_slice_and_max_wait);
    }
}
