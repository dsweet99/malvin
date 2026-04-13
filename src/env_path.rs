//! Resolve executable names on `PATH` (shared by ACP spawn resolution and CLI helpers).

use std::path::PathBuf;

/// Returns an absolute path to `bin` if it exists as a regular file on `PATH`.
#[must_use]
pub fn lookup_bin_on_path(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(bin))
        .find(|candidate| candidate.is_file())
}

/// Prefer `agent`, then `cursor-agent` (same default order as ACP spawn when `MALVIN_AGENT_ACP_BIN` is unset).
#[must_use]
pub fn agent_or_cursor_agent_bin() -> Option<PathBuf> {
    lookup_bin_on_path("agent").or_else(|| lookup_bin_on_path("cursor-agent"))
}

#[cfg(all(test, unix))]
mod tests {
    #![allow(unsafe_code)]

    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use tempfile::tempdir;

    use crate::test_utils::test_env_lock;

    use super::*;

    #[test]
    fn lookup_bin_on_path_accepts_first_is_file_even_without_execute_bit() {
        let _guard = test_env_lock();
        let tmp = tempdir().unwrap();
        let probe_name = "malvin_path_probe_bin";
        let candidate = tmp.path().join(probe_name);
        fs::write(&candidate, "").unwrap();
        fs::set_permissions(&candidate, fs::Permissions::from_mode(0o644)).unwrap();

        let old_path = std::env::var_os("PATH");
        let combined = old_path.as_ref().map_or_else(
            || tmp.path().display().to_string(),
            |rest| format!("{}:{}", tmp.path().display(), rest.to_string_lossy()),
        );

        let got = unsafe {
            std::env::set_var("PATH", &combined);
            let out = lookup_bin_on_path(probe_name);
            match &old_path {
                Some(v) => std::env::set_var("PATH", v),
                None => std::env::remove_var("PATH"),
            }
            out
        };

        assert_eq!(got, Some(candidate));
    }
}
