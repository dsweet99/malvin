
use crate::acp::{AuthError, has_api_key};

#[must_use]
pub fn effective_sdk_api_key() -> Option<String> {
    for key in ["CURSOR_API_KEY", "CURSOR_AGENT_API_KEY", "AGENT_API_KEY"] {
        if let Ok(v) = std::env::var(key) {
            let t = v.trim().trim_start_matches('\u{feff}');
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

pub fn ensure_sdk_authenticated() -> Result<(), AuthError> {
    if has_api_key() || effective_sdk_api_key().is_some() {
        return Ok(());
    }
    Err(AuthError(
        "Cursor SDK backend is not authenticated. Set CURSOR_API_KEY (or CURSOR_AGENT_API_KEY / AGENT_API_KEY). `agent login` alone is not enough."
            .to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_sdk_authenticated_ok_with_key() {
        let _guard = crate::test_utils::test_env_lock();
        unsafe {
            std::env::set_var("CURSOR_API_KEY", "test-key");
        }
        assert!(ensure_sdk_authenticated().is_ok());
        unsafe {
            std::env::remove_var("CURSOR_API_KEY");
        }
    }
}
