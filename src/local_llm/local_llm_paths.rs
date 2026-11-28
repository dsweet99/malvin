use std::path::PathBuf;
use std::time::Duration;

const IDLE_ENV: &str = "MALVIN_TIME_SINCE_LAST_CALL_SECONDS";
const SOCK_ENV: &str = "MALVIN_LOCAL_LLM_MANAGER_SOCK";
const LOCK_ENV: &str = "MALVIN_LOCAL_LLM_MANAGER_LOCK";
const OLLAMA_BIN_ENV: &str = "MALVIN_OLLAMA";

pub(crate) fn idle_duration() -> Duration {
    Duration::from_secs(idle_secs())
}

pub(crate) fn idle_secs() -> u64 {
    if let Ok(raw) = std::env::var(IDLE_ENV) {
        let trimmed = raw.trim();
        if let Ok(n) = trimmed.parse::<u64>()
            && n > 0
        {
            return n;
        }
    }
    crate::config::malvin_config_file::load_malvin_config().time_since_last_call_seconds
}

pub(crate) fn manager_sock_path() -> PathBuf {
    env_path(SOCK_ENV).unwrap_or_else(|| {
        crate::workspace::workspace_paths::malvin_user_home_root().join("local_llm_manager.sock")
    })
}

pub(crate) fn manager_lock_path() -> PathBuf {
    env_path(LOCK_ENV).unwrap_or_else(|| {
        crate::workspace::workspace_paths::malvin_user_home_root().join("local_llm_manager.lock")
    })
}

fn env_path(key: &str) -> Option<PathBuf> {
    let raw = std::env::var(key).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

pub(crate) fn ollama_bin() -> String {
    std::env::var(OLLAMA_BIN_ENV)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "ollama".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_process::with_env;

    #[test]
    fn idle_secs_env_overrides_default() {
        with_env(IDLE_ENV, Some("12"), || {
            assert_eq!(idle_secs(), 12);
        });
        with_env(IDLE_ENV, Some("0"), || {
            assert_eq!(
                idle_secs(),
                crate::config::malvin_config_file::DEFAULT_TIME_SINCE_LAST_CALL_SECONDS
            );
        });
    }

    #[test]
    fn sock_and_lock_env_override_home() {
        with_env(SOCK_ENV, Some("/tmp/malvin-llm-test.sock"), || {
            assert_eq!(
                manager_sock_path(),
                PathBuf::from("/tmp/malvin-llm-test.sock")
            );
        });
        with_env(LOCK_ENV, Some("/tmp/malvin-llm-test.lock"), || {
            assert_eq!(
                manager_lock_path(),
                PathBuf::from("/tmp/malvin-llm-test.lock")
            );
        });
    }
}
