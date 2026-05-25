use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct DeferredLogConfig {
    pub max_age: Duration,
    pub max_drain_per_log: usize,
    pub cursor_dir: PathBuf,
}

impl Default for DeferredLogConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl DeferredLogConfig {
    pub fn from_env() -> Self {
        Self {
            max_age: defer_log_max_age_from_env(),
            max_drain_per_log: defer_log_max_drain_from_env(),
            cursor_dir: defer_log_cursor_dir_from_env(),
        }
    }
}

pub fn defer_log_enabled_from_env() -> bool {
    if env_is_zero("MALVIN_DEFER_LOG") {
        return false;
    }
    if std::env::var("MALVIN_TEST_NO_REAL_AGENT")
        .ok()
        .is_some_and(|v| v == "1")
    {
        return false;
    }
    true
}

pub(crate) fn env_is_zero(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .is_some_and(|v| v == "0" || v.eq_ignore_ascii_case("false"))
}

pub(crate) fn defer_log_max_age_from_env() -> Duration {
    std::env::var("MALVIN_DEFER_LOG_MAX_AGE_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map_or(Duration::from_millis(1000), Duration::from_millis)
}

pub(crate) fn defer_log_max_drain_from_env() -> usize {
    std::env::var("MALVIN_DEFER_LOG_MAX_DRAIN")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(64)
}

pub(crate) fn defer_log_cursor_dir_from_env() -> PathBuf {
    std::env::var("MALVIN_CURSOR_DIR")
        .ok()
        .map_or_else(|| crate::user_home_dir().join(".cursor"), PathBuf::from)
}
