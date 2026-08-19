use std::path::{Path, PathBuf};

use super::AgentError;

pub(crate) const MALVIN_TEST_NO_REAL_AGENT_ENV: &str = "MALVIN_TEST_NO_REAL_AGENT";

pub(crate) fn test_no_real_agent_enabled() -> bool {
    std::env::var_os(MALVIN_TEST_NO_REAL_AGENT_ENV).is_some_and(|v| !v.is_empty() && v != "0")
}

pub(crate) fn has_api_key() -> bool {
    for key in ["CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY"] {
        if std::env::var_os(key).is_some_and(|v| !v.is_empty()) {
            return true;
        }
    }
    false
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
    }
}
