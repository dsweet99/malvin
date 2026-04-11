// Resolve Cursor CLI credentials for the `agent acp` child (`GEORGE_CURSOR_*` vs process `CURSOR_*`).

pub(crate) fn nonempty_explicit_or_env_var(explicit: Option<&str>, env_var: &str) -> Option<String> {
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
