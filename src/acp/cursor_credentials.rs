//! Resolve Cursor CLI credentials for the `agent acp` child (`GEORGE_CURSOR_*` vs process `CURSOR_*`).
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

fn nonempty_explicit_or_env_var(explicit: Option<&str>, env_var: &str) -> Option<String> {
    if let Some(s) = explicit {
        let t = s.trim().trim_start_matches('\u{feff}');
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    let Ok(s) = std::env::var(env_var) else {
        return None;
    };
    let t = s.trim().trim_start_matches('\u{feff}');
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Prefer nonempty `explicit` (from [`crate::config::Config`] / `GEORGE_CURSOR_API_KEY`), else
/// nonempty `CURSOR_API_KEY` in the process environment.
pub(crate) fn effective_cursor_api_key(explicit: Option<&str>) -> Option<String> {
    nonempty_explicit_or_env_var(explicit, "CURSOR_API_KEY")
}

/// Prefer nonempty `explicit` (`GEORGE_CURSOR_AUTH_TOKEN`), else nonempty `CURSOR_AUTH_TOKEN`.
pub(crate) fn effective_cursor_auth_token(explicit: Option<&str>) -> Option<String> {
    nonempty_explicit_or_env_var(explicit, "CURSOR_AUTH_TOKEN")
}

#[cfg(test)]
mod tests {
    #![allow(unsafe_code)]

    use super::{effective_cursor_api_key, effective_cursor_auth_token};
    use crate::test_utils::test_env_lock;

    fn clear_cursor_cred_env() {
        unsafe {
            std::env::remove_var("CURSOR_API_KEY");
            std::env::remove_var("CURSOR_AUTH_TOKEN");
        }
    }

    #[test]
    fn api_key_explicit_nonempty() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        assert_eq!(
            effective_cursor_api_key(Some("k")).as_deref(),
            Some("k")
        );
    }

    #[test]
    fn api_key_trims_whitespace_and_bom_on_explicit() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        assert_eq!(
            effective_cursor_api_key(Some("  abc  ")).as_deref(),
            Some("abc")
        );
        assert_eq!(
            effective_cursor_api_key(Some("\u{feff}key_123")).as_deref(),
            Some("key_123")
        );
    }

    #[test]
    fn api_key_env_only_whitespace_yields_none() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        unsafe {
            std::env::set_var("CURSOR_API_KEY", "   ");
        }
        assert!(effective_cursor_api_key(None).is_none());
        clear_cursor_cred_env();
    }

    #[test]
    fn api_key_explicit_empty_uses_env() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        unsafe {
            std::env::set_var("CURSOR_API_KEY", "from-env");
        }
        assert_eq!(
            effective_cursor_api_key(Some("")).as_deref(),
            Some("from-env")
        );
        clear_cursor_cred_env();
    }

    #[test]
    fn api_key_explicit_none_uses_env() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        unsafe {
            std::env::set_var("CURSOR_API_KEY", "e");
        }
        assert_eq!(effective_cursor_api_key(None).as_deref(), Some("e"));
        clear_cursor_cred_env();
    }

    #[test]
    fn api_key_explicit_nonempty_overrides_env() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        unsafe {
            std::env::set_var("CURSOR_API_KEY", "env");
        }
        assert_eq!(
            effective_cursor_api_key(Some("win")).as_deref(),
            Some("win")
        );
        clear_cursor_cred_env();
    }

    #[test]
    fn api_key_unset_when_missing_and_no_explicit() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        assert!(effective_cursor_api_key(None).is_none());
    }

    #[test]
    fn api_key_unset_when_env_empty_and_no_explicit() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        unsafe {
            std::env::set_var("CURSOR_API_KEY", "");
        }
        assert!(effective_cursor_api_key(None).is_none());
        clear_cursor_cred_env();
    }

    #[test]
    fn api_key_explicit_empty_env_missing() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        assert!(effective_cursor_api_key(Some("")).is_none());
    }

    #[test]
    fn auth_token_explicit_nonempty() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        assert_eq!(
            effective_cursor_auth_token(Some("t")).as_deref(),
            Some("t")
        );
    }

    #[test]
    fn auth_token_explicit_empty_uses_env() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        unsafe {
            std::env::set_var("CURSOR_AUTH_TOKEN", "tok-env");
        }
        assert_eq!(
            effective_cursor_auth_token(Some("")).as_deref(),
            Some("tok-env")
        );
        clear_cursor_cred_env();
    }

    #[test]
    fn auth_token_explicit_none_uses_env() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        unsafe {
            std::env::set_var("CURSOR_AUTH_TOKEN", "x");
        }
        assert_eq!(effective_cursor_auth_token(None).as_deref(), Some("x"));
        clear_cursor_cred_env();
    }

    #[test]
    fn auth_token_explicit_nonempty_overrides_env() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        unsafe {
            std::env::set_var("CURSOR_AUTH_TOKEN", "env");
        }
        assert_eq!(
            effective_cursor_auth_token(Some("mine")).as_deref(),
            Some("mine")
        );
        clear_cursor_cred_env();
    }

    #[test]
    fn auth_token_unset_when_missing_and_no_explicit() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        assert!(effective_cursor_auth_token(None).is_none());
    }

    #[test]
    fn auth_token_unset_when_env_empty_and_no_explicit() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        unsafe {
            std::env::set_var("CURSOR_AUTH_TOKEN", "");
        }
        assert!(effective_cursor_auth_token(None).is_none());
        clear_cursor_cred_env();
    }

    #[test]
    fn auth_token_explicit_empty_env_missing() {
        let _g = test_env_lock();
        clear_cursor_cred_env();
        assert!(effective_cursor_auth_token(Some("")).is_none());
    }

    const ARBITRARY_ENV: &str = "MALVIN_TEST_ACP_CRED_RESOLVER";

    #[test]
    fn resolver_unset_env_without_explicit() {
        let _g = test_env_lock();
        unsafe {
            std::env::remove_var(ARBITRARY_ENV);
        }
        assert!(super::nonempty_explicit_or_env_var(None, ARBITRARY_ENV).is_none());
    }

    #[test]
    fn resolver_nonempty_env_without_explicit() {
        let _g = test_env_lock();
        unsafe {
            std::env::set_var(ARBITRARY_ENV, "arbitrary");
        }
        assert_eq!(
            super::nonempty_explicit_or_env_var(None, ARBITRARY_ENV).as_deref(),
            Some("arbitrary")
        );
        unsafe {
            std::env::remove_var(ARBITRARY_ENV);
        }
    }

    #[test]
    fn resolver_empty_env_without_explicit() {
        let _g = test_env_lock();
        unsafe {
            std::env::set_var(ARBITRARY_ENV, "");
        }
        assert!(super::nonempty_explicit_or_env_var(None, ARBITRARY_ENV).is_none());
        unsafe {
            std::env::remove_var(ARBITRARY_ENV);
        }
    }

    #[test]
    fn resolver_explicit_nonempty_ignores_env() {
        let _g = test_env_lock();
        unsafe {
            std::env::set_var(ARBITRARY_ENV, "env");
        }
        assert_eq!(
            super::nonempty_explicit_or_env_var(Some("ex"), ARBITRARY_ENV).as_deref(),
            Some("ex")
        );
        unsafe {
            std::env::remove_var(ARBITRARY_ENV);
        }
    }

    #[test]
    fn resolver_explicit_empty_reads_env() {
        let _g = test_env_lock();
        unsafe {
            std::env::set_var(ARBITRARY_ENV, "fallback");
        }
        assert_eq!(
            super::nonempty_explicit_or_env_var(Some(""), ARBITRARY_ENV).as_deref(),
            Some("fallback")
        );
        unsafe {
            std::env::remove_var(ARBITRARY_ENV);
        }
    }
}
