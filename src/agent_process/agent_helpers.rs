use std::path::{Path, PathBuf};

use super::AgentError;

pub(crate) const MALVIN_TEST_NO_REAL_AGENT_ENV: &str = "MALVIN_TEST_NO_REAL_AGENT";

pub fn test_no_real_agent_enabled() -> bool {
    std::env::var_os(MALVIN_TEST_NO_REAL_AGENT_ENV).is_some_and(|v| !v.is_empty() && v != "0")
}

pub(crate) fn has_api_key() -> bool {
    for key in ["CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY"] {
        if env_key_nonempty(key) {
            return true;
        }
    }
    false
}

#[must_use]
pub(crate) fn env_key_nonempty(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|v| !v.trim().trim_start_matches('\u{feff}').is_empty())
}

pub(crate) fn resolve_acp_session_cwd(cwd: &Path) -> Result<PathBuf, AgentError> {
    let base = if cwd.as_os_str().is_empty() || cwd == Path::new(".") {
        std::env::current_dir().map_err(|e| AgentError(e.to_string()))?
    } else if cwd.is_absolute() {
        cwd.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| AgentError(e.to_string()))?
            .join(cwd)
    };
    Ok(base.canonicalize().unwrap_or(base))
}

/// Runs `f` with `key` set (or removed), then restores the prior value even if `f` panics.
#[cfg(test)]
#[allow(unsafe_code)]
pub(crate) fn with_env(key: &str, value: Option<&str>, f: impl FnOnce()) {
    let _restore = crate::test_support::test_utils::SavedEnvVars::capture(&[key]);
    unsafe {
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
    f();
}

#[cfg(test)]
mod agent_helpers_tests {
    use super::*;
    use std::path::Path;
    #[test]
    fn resolve_acp_session_cwd_expands_dot_and_smoke_agent_helper_symbols() {
        {
            let _lock = crate::test_support::test_utils::test_env_lock();
            let key = "MALVIN_TEST_WITH_ENV_PANIC_PROBE";
            with_env(key, Some("before"), || {
                let panicked = std::panic::catch_unwind(|| {
                    with_env(key, Some("during"), || panic!("boom"));
                });
                assert!(panicked.is_err());
                assert_eq!(std::env::var(key).as_deref(), Ok("before"));
            });
            assert!(std::env::var_os(key).is_none());
        }
        {
            let cwd = std::env::current_dir().expect("cwd");
            let resolved = resolve_acp_session_cwd(Path::new(".")).expect("resolve");
            assert!(resolved.is_absolute());
            assert_eq!(resolved, cwd.canonicalize().unwrap_or(cwd));
        }
        {
            let _ = test_no_real_agent_enabled();
            let _ = has_api_key();
            let _ = env_key_nonempty("CURSOR_API_KEY");
        }
    }
}
