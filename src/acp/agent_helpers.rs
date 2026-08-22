use std::path::{Path, PathBuf};

use super::AgentError;

pub(crate) const MALVIN_TEST_NO_REAL_AGENT_ENV: &str = "MALVIN_TEST_NO_REAL_AGENT";

pub(crate) const NO_FORCE_MSG: &str =
    "--no-force is not supported (malvin runs tools headlessly; no interactive approval)";

/// Idle-timeout prefixes. Drain emit sites and teardown needles share these so a
/// timeout cannot miss session recycle.
pub(crate) const DRAIN_IDLE_PREFIX_BRIDGE: &str = "bridge timed out";
pub(crate) const DRAIN_IDLE_PREFIX_PI: &str = "pi rpc timed out";
pub(crate) const DRAIN_IDLE_PREFIX_CODEX: &str = "codex timed out";

pub(crate) fn test_no_real_agent_enabled() -> bool {
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

#[cfg(test)]
#[allow(unsafe_code)]
pub(crate) fn with_env(key: &str, value: Option<&str>, f: impl FnOnce()) {
    let prior = std::env::var_os(key);
    unsafe {
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
    f();
    unsafe {
        match prior {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
}

#[cfg(test)]
mod agent_helpers_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn resolve_acp_session_cwd_expands_dot() {
        let cwd = std::env::current_dir().expect("cwd");
        let resolved = resolve_acp_session_cwd(Path::new(".")).expect("resolve");
        assert!(resolved.is_absolute());
        assert_eq!(resolved, cwd.canonicalize().unwrap_or(cwd));
    }

    #[test]
    fn smoke_agent_helper_symbols() {
        let _ = test_no_real_agent_enabled();
        let _ = has_api_key();
        let _ = env_key_nonempty("CURSOR_API_KEY");
        let _ = NO_FORCE_MSG;
        let _ = DRAIN_IDLE_PREFIX_BRIDGE;
        let _ = DRAIN_IDLE_PREFIX_PI;
        let _ = DRAIN_IDLE_PREFIX_CODEX;
    }
}
