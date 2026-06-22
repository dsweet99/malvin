//! Bash execution via malvin sandbox.

use std::path::Path;

use crate::malvin_sandbox::malvin_std_command;

/// Returns an error when `bash` is not executable on `PATH`.
pub fn ensure_bash_on_path() -> Result<(), String> {
    let path = which_bash()?;
    if !path.is_file() {
        return Err("bash is not available on PATH (required for --mini)".to_string());
    }
    Ok(())
}

fn which_bash() -> Result<std::path::PathBuf, String> {
    let output = malvin_std_command("which")
        .arg("bash")
        .output()
        .map_err(|e| format!("failed to probe bash on PATH: {e}"))?;
    if !output.status.success() {
        return Err("bash is not available on PATH (required for --mini)".to_string());
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        return Err("bash is not available on PATH (required for --mini)".to_string());
    }
    Ok(std::path::PathBuf::from(path))
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BashExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Run one bash `-c` command in `cwd`.
///
/// # Errors
///
/// Returns an error when spawn or wait fails.
pub fn run_bash_command(cwd: &Path, command: &str) -> Result<BashExecResult, String> {
    let output = malvin_std_command("bash")
        .arg("-c")
        .arg(command)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("bash exec failed: {e}"))?;
    let exit_code = output.status.code().unwrap_or(-1);
    Ok(BashExecResult {
        exit_code,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

use std::fmt::Write;

pub fn format_observation(results: &[BashExecResult]) -> String {
    let mut out = String::new();
    for (i, r) in results.iter().enumerate() {
        if i > 0 {
            out.push_str("\n---\n");
        }
        let _ = write!(
            out,
            "Exit code {}\nstdout:\n{}\nstderr:\n{}",
            r.exit_code, r.stdout, r.stderr
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mini_init_errors_when_bash_missing() {
        if ensure_bash_on_path().is_ok() {
            return;
        }
        let err = ensure_bash_on_path().expect_err("bash missing");
        assert!(err.contains("bash"));
    }

    #[test]
    fn run_bash_command_and_format_observation_round_trip() {
        if ensure_bash_on_path().is_err() {
            return;
        }
        let tmp = tempfile::tempdir().expect("tempdir");
        let result = run_bash_command(tmp.path(), "echo hi").expect("bash");
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hi"));
        let obs = format_observation(&[result]);
        assert!(obs.contains("Exit code 0"));
        assert!(obs.contains("stdout:"));
    }
}
