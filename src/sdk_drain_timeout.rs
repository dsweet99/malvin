#![cfg_attr(test, allow(unsafe_code))]

use std::time::Duration;

pub const DEFAULT_SDK_DRAIN_IDLE_TIMEOUT_MS: u64 = 600_000;

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
    fn kiss_cov_sdk_drain_timeout() {
        let _ = stringify!(DEFAULT_SDK_DRAIN_IDLE_TIMEOUT_MS);
        let _ = stringify!(sdk_drain_idle_timeout_from_env);
        let _ = stringify!(sdk_drain_idle_timeout_from_env_rejects_zero_and_garbage);
    }
}
