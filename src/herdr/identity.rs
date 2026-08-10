//! Herdr live `name` vs report `agent` label, and display title selection.

/// Build a unique Herdr live agent name from a run-dir session id.
///
/// Herdr names must match `[a-z][a-z0-9_-]{0,31}`. Report/`display_agent` stay
/// `"malvin"`; this value is only for `agent.rename` / name targeting.
#[must_use]
pub fn herdr_live_name(session_id: &str) -> String {
    let short = session_id.rsplit('_').next().unwrap_or(session_id);
    let body: String = short
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        .map(|c| c.to_ascii_lowercase())
        .collect();
    let mut name = format!("m{body}");
    if name.len() > 32 {
        name.truncate(32);
    }
    if name.len() < 2 {
        name = format!("m{}", std::process::id());
        name.truncate(32);
    }
    name
}

/// Prefer malvin session `--name` / auto slot for `title`; never the run-dir basename.
///
/// Returns `None` when the process only has the pid fallback slot (no session name).
#[must_use]
pub fn display_title() -> Option<String> {
    let slot = crate::acp_spawn_lock::active_acp_lock_slot();
    if is_pid_fallback_slot(&slot) {
        None
    } else {
        Some(slot)
    }
}

fn is_pid_fallback_slot(slot: &str) -> bool {
    slot
        .strip_prefix("pid")
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::{display_title, herdr_live_name, is_pid_fallback_slot};

    #[test]
    fn herdr_live_name_prefixes_short_suffix() {
        let n = herdr_live_name("20260804_140533_4gk60f1m");
        assert_eq!(n, "m4gk60f1m");
        assert!(n.chars().next().is_some_and(|c| c.is_ascii_lowercase()));
        assert!(n.len() <= 32);
    }

    #[test]
    fn pid_fallback_slot_detection() {
        assert!(is_pid_fallback_slot("pid12345"));
        assert!(!is_pid_fallback_slot("probe"));
        assert!(!is_pid_fallback_slot("pid"));
        let _ = display_title();
    }
}
