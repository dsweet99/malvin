//! PATH lookup, argv capture for traces, and run-dir display paths.

use std::path::{Path, PathBuf};

pub use crate::user_home::user_home_dir;
use std::sync::OnceLock;

static COMMAND_LINE: OnceLock<String> = OnceLock::new();

#[must_use]
pub fn lookup_bin_on_path(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(bin))
        .find(|candidate| candidate.is_file())
}

#[must_use]
pub fn agent_or_cursor_agent_bin() -> Option<PathBuf> {
    for name in ["agent", "cursor-agent"] {
        #[cfg(test)]
        if let Some(fake) = crate::repo_checks::test_fake_command_path(name) {
            return Some(fake);
        }
        if let Some(path) = lookup_bin_on_path(name) {
            return Some(path);
        }
    }
    None
}

#[allow(clippy::missing_errors_doc)]
pub fn require_kiss_for_malvin(subcommand: &str) -> Result<(), String> {
    if lookup_bin_on_path("kiss").is_some() {
        return Ok(());
    }
    Err(format!(
        "`kiss` is not installed or not on PATH; install it with `cargo install kiss-ai` before running `malvin {subcommand}`."
    ))
}

pub fn init_from_env() {
    let _ = COMMAND_LINE.get_or_init(|| std::env::args().collect::<Vec<_>>().join(" "));
    crate::malvin_sandbox::init_malvin_spawn_baseline();
    crate::tracing_init::init_tracing();
}

#[must_use]
pub fn command_line() -> Option<&'static str> {
    COMMAND_LINE.get().map(String::as_str)
}

pub const DEFAULT_CLI_MODEL: &str = "auto";

/// Default bounded attempts per ACP spawn or `session/prompt` (1s / 3s backoff between tries).
pub const DEFAULT_MAX_ACP_RETRIES: u32 = 3;

pub const DEFAULT_ACP_RPC_TIMEOUT_SECS: u64 = 600;

#[must_use]
pub fn acp_rpc_timeout_secs_from_env() -> u64 {
    std::env::var("MALVIN_ACP_RPC_TIMEOUT_SECS")
        .ok()
        .map_or(DEFAULT_ACP_RPC_TIMEOUT_SECS, |s| {
            s.parse::<u64>().map_or_else(
                |_| {
                    tracing::warn!(
                        target: "malvin::support_paths",
                        value = %s,
                        "MALVIN_ACP_RPC_TIMEOUT_SECS is not a positive integer; using default"
                    );
                    DEFAULT_ACP_RPC_TIMEOUT_SECS
                },
                |n| n.max(1),
            )
        })
}

#[allow(clippy::missing_errors_doc)]
pub fn format_logs_dir(run_dir: &Path) -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let cwd_abs = cwd.canonicalize().map_err(|e| e.to_string())?;
    let run_abs = run_dir.canonicalize().map_err(|e| e.to_string())?;
    Ok(run_abs.strip_prefix(&cwd_abs).map_or_else(
        |_| run_abs.display().to_string(),
        |p| format!("./{}", p.display()),
    ))
}

#[cfg(all(test, unix))]
mod env_path_tests {
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

    #[test]
    fn require_kiss_for_malvin_errors_with_install_hint_when_kiss_missing() {
        let _guard = test_env_lock();
        let tmp = tempdir().unwrap();
        let isolated = tmp.path().join("bin");
        fs::create_dir_all(&isolated).unwrap();

        let old_path = std::env::var_os("PATH");
        let err = unsafe {
            std::env::set_var("PATH", &isolated);
            let e = require_kiss_for_malvin("init").unwrap_err();
            match &old_path {
                Some(v) => std::env::set_var("PATH", v),
                None => std::env::remove_var("PATH"),
            }
            e
        };

        assert!(
            err.contains("cargo install kiss-ai") && err.contains("malvin init"),
            "unexpected message: {err:?}"
        );
    }
}

#[cfg(test)]
mod invocation_tests {
    use super::*;

    #[test]
    fn init_records_argv() {
        init_from_env();
        let line = command_line().expect("command line after init");
        assert!(!line.is_empty());
    }

    #[test]
    fn agent_bin_and_rpc_timeout_from_env() {
        let _ = agent_or_cursor_agent_bin();
        assert!(acp_rpc_timeout_secs_from_env() >= 1);
    }
}

#[cfg(test)]
mod log_paths_tests {
    use crate::test_utils::test_env_lock;

    use super::format_logs_dir;

    #[test]
    fn relative_when_run_dir_is_under_cwd() {
        let _g = test_env_lock();
        let old = std::env::current_dir().unwrap();
        let base = tempfile::tempdir().unwrap();
        let run = base.path().join("nest").join("run");
        std::fs::create_dir_all(&run).unwrap();
        std::env::set_current_dir(base.path()).unwrap();
        let out = format_logs_dir(&run).unwrap();
        std::env::set_current_dir(&old).unwrap();
        assert_eq!(out, "./nest/run");
    }

    #[test]
    fn absolute_when_run_dir_not_under_cwd() {
        let _g = test_env_lock();
        let old = std::env::current_dir().unwrap();
        let cwd_tmp = tempfile::tempdir().unwrap();
        let run_tmp = tempfile::tempdir().unwrap();
        let run = run_tmp.path().join("outside");
        std::fs::create_dir_all(&run).unwrap();
        std::env::set_current_dir(cwd_tmp.path()).unwrap();
        let out = format_logs_dir(&run).unwrap();
        std::env::set_current_dir(&old).unwrap();
        assert!(
            out.starts_with('/'),
            "expected absolute path when run is not under cwd, got {out:?}"
        );
        assert!(out.contains("outside"));
    }

    #[test]
    fn errors_when_run_dir_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("nope").join("run");
        let err = format_logs_dir(&p).unwrap_err();
        assert!(!err.is_empty());
    }
}
