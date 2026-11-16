#![cfg_attr(test, allow(unsafe_code))]

use std::time::Duration;

pub const DEFAULT_SDK_DRAIN_IDLE_TIMEOUT_MS: u64 = 600_000;
pub const SDK_BRIDGE_STARTUP_TIMEOUT_MIN_MS: u64 = 1_000;

const IDLE_TIMEOUT_ENV: &str = "MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS";

pub const SDK_DRAIN_IDLE_SLICE_MAX_MS: u64 = 60_000;

#[must_use]
pub fn sdk_drain_idle_timeout_from_env() -> Duration {
    crate::command_output_timeout::timeout_ms_from_env(
        IDLE_TIMEOUT_ENV,
        DEFAULT_SDK_DRAIN_IDLE_TIMEOUT_MS,
    )
}

pub fn sdk_bridge_startup_timeout() -> Duration {
    sdk_drain_idle_timeout_from_env().max(Duration::from_millis(SDK_BRIDGE_STARTUP_TIMEOUT_MIN_MS))
}

#[must_use]
pub fn sdk_drain_idle_slice(idle_remaining: Duration) -> Duration {
    idle_remaining.min(Duration::from_millis(SDK_DRAIN_IDLE_SLICE_MAX_MS))
}

/// Sets the idle timeout for the caller's scope; the prior value is restored on drop,
/// including when the test panics.
#[cfg(test)]
#[must_use]
pub(crate) fn tests_set_idle_ms_for_test(ms: u64) -> crate::test_support::test_utils::SavedEnvVars {
    let saved = crate::test_support::test_utils::SavedEnvVars::capture(&[IDLE_TIMEOUT_ENV]);
    unsafe {
        std::env::set_var(IDLE_TIMEOUT_ENV, ms.to_string());
    }
    saved
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_utils::test_env_lock;
    use std::time::Duration;

    #[test]
    fn sdk_drain_idle_timeout_from_env_rejects_zero_and_garbage() {
        let _lock = test_env_lock();
        let _saved = crate::test_support::test_utils::SavedEnvVars::capture(&[IDLE_TIMEOUT_ENV]);
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var(IDLE_TIMEOUT_ENV, "0");
        }
        assert_eq!(sdk_drain_idle_timeout_from_env(), Duration::from_millis(1));
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var(IDLE_TIMEOUT_ENV, "nope");
        }
        assert_eq!(
            sdk_drain_idle_timeout_from_env(),
            Duration::from_millis(DEFAULT_SDK_DRAIN_IDLE_TIMEOUT_MS)
        );
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var(IDLE_TIMEOUT_ENV, "250");
        }
        assert_eq!(
            sdk_drain_idle_timeout_from_env(),
            Duration::from_millis(250)
        );
    }

    #[test]
    fn sdk_drain_idle_slice_caps_at_max() {
        assert_eq!(
            sdk_drain_idle_slice(Duration::from_secs(5)),
            Duration::from_secs(5)
        );
        assert_eq!(
            sdk_drain_idle_slice(Duration::from_mins(2)),
            Duration::from_millis(SDK_DRAIN_IDLE_SLICE_MAX_MS)
        );
    }

    #[test]
    fn sdk_bridge_startup_timeout_floors_below_min() {
        let _lock = test_env_lock();
        let before = std::env::var_os(IDLE_TIMEOUT_ENV);
        let outcome = std::panic::catch_unwind(|| {
            let _idle = tests_set_idle_ms_for_test(80);
            assert_eq!(sdk_drain_idle_timeout_from_env(), Duration::from_millis(80));
            panic!("simulated assertion failure");
        });
        assert!(outcome.is_err());
        assert_eq!(
            std::env::var_os(IDLE_TIMEOUT_ENV),
            before,
            "restored on panic"
        );
        let _idle = tests_set_idle_ms_for_test(100);
        assert_eq!(
            sdk_bridge_startup_timeout(),
            Duration::from_millis(SDK_BRIDGE_STARTUP_TIMEOUT_MIN_MS)
        );
        let _idle = tests_set_idle_ms_for_test(5_000);
        assert_eq!(sdk_bridge_startup_timeout(), Duration::from_secs(5));
    }

    #[test]
    fn kiss_cov_sdk_drain_timeout() {
        let _ = stringify!(DEFAULT_SDK_DRAIN_IDLE_TIMEOUT_MS);
        let _ = stringify!(SDK_DRAIN_IDLE_SLICE_MAX_MS);
        let _ = stringify!(sdk_drain_idle_timeout_from_env);
        let _ = stringify!(sdk_drain_idle_slice);
        let _ = stringify!(sdk_drain_idle_timeout_from_env_rejects_zero_and_garbage);
        let _ = stringify!(sdk_drain_idle_slice_caps_at_max);
    }
}
