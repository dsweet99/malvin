//! Run `kiss clamp` when a workspace has source files but no `.kissconfig`.

use std::path::Path;

use super::command_support::{apply_fake_path_if_present, run_command_failure, run_command_for};
use super::gate_log::try_append_command_output;
use super::types::RepoGateFailure;

#[must_use]
pub fn kiss_clamp_needed(work_dir: &Path) -> bool {
    !work_dir.join(".kissconfig").exists() && crate::source_detect::has_source_files(work_dir)
}

/// Generate `.kissconfig` via `kiss clamp` when the workspace has source but no config file.
#[allow(clippy::missing_errors_doc)]
pub fn ensure_kiss_clamp_if_needed(work_dir: &Path) -> Result<(), String> {
    ensure_kiss_clamp_if_needed_with_details(work_dir, None).map_err(RepoGateFailure::into_error)
}

pub(crate) fn ensure_kiss_clamp_if_needed_with_details(
    work_dir: &Path,
    run_log_dir: Option<&Path>,
) -> Result<(), RepoGateFailure> {
    if !kiss_clamp_needed(work_dir) {
        return Ok(());
    }
    let mut command = crate::malvin_sandbox::malvin_std_command(run_command_for("kiss"));
    command.arg("clamp").current_dir(work_dir);
    apply_fake_path_if_present(&mut command);
    let output = command
        .output()
        .map_err(|e| RepoGateFailure::Message(format!("`kiss clamp` failed to start: {e}")))?;
    try_append_command_output(run_log_dir, "kiss clamp", &output);
    if output.status.success() {
        Ok(())
    } else {
        Err(run_command_failure("kiss clamp", &output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiss_clamp_not_needed_without_source_files() {
        let tmp = tempfile::tempdir().expect("tempdir");
        assert!(!kiss_clamp_needed(tmp.path()));
        ensure_kiss_clamp_if_needed(tmp.path()).expect("empty dir");
    }

    #[test]
    fn kiss_clamp_not_needed_when_kissconfig_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("main.rs"), "fn main() {}").expect("main.rs");
        std::fs::write(tmp.path().join(".kissconfig"), "[gate]\n").expect(".kissconfig");
        assert!(!kiss_clamp_needed(tmp.path()));
    }

    #[cfg(unix)]
    fn install_fake_kiss_script(bin_dir: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;

        let kiss = bin_dir.join("kiss");
        std::fs::write(&kiss, body).expect("fake kiss");
        let mut perms = std::fs::metadata(&kiss).expect("meta").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&kiss, perms).expect("chmod");
    }

    #[cfg(unix)]
    fn source_workspace_without_kissconfig() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let work = tmp.path().to_path_buf();
        std::fs::write(work.join("main.rs"), "fn main() {}").expect("main.rs");
        (tmp, work)
    }

    #[cfg(unix)]
    #[test]
    fn kiss_clamp_writes_kissconfig_for_source_workspace() {
        let (_tmp, work) = source_workspace_without_kissconfig();
        let bin_dir = tempfile::tempdir().expect("bindir");
        install_fake_kiss_script(
            bin_dir.path(),
            "#!/bin/sh\ncd \"$PWD\"\necho '[gate]' > .kissconfig\necho 'test_coverage_threshold = 0' >> .kissconfig\nexit 0\n",
        );
        let _guard = super::super::command_support::set_fake_command_dir(bin_dir.path());

        ensure_kiss_clamp_if_needed(&work).expect("kiss clamp");
        assert!(work.join(".kissconfig").is_file());
    }

    #[cfg(unix)]
    #[test]
    fn kiss_clamp_fails_when_kiss_exits_nonzero() {
        let (_tmp, work) = source_workspace_without_kissconfig();
        let bin_dir = tempfile::tempdir().expect("bindir");
        install_fake_kiss_script(bin_dir.path(), "#!/bin/sh\nexit 1\n");
        let _guard = super::super::command_support::set_fake_command_dir(bin_dir.path());

        let err = ensure_kiss_clamp_if_needed_with_details(&work, None).unwrap_err();
        match err {
            RepoGateFailure::Command(failure) => {
                assert_eq!(failure.command, "kiss clamp");
                assert_eq!(failure.exit_code, Some(1));
            }
            RepoGateFailure::Message(other) => panic!("expected command failure, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn kiss_clamp_appends_to_run_log_on_success() {
        let (_tmp, work) = source_workspace_without_kissconfig();
        let bin_dir = tempfile::tempdir().expect("bindir");
        install_fake_kiss_script(
            bin_dir.path(),
            "#!/bin/sh\ncd \"$PWD\"\necho '[gate]' > .kissconfig\nexit 0\n",
        );
        let _guard = super::super::command_support::set_fake_command_dir(bin_dir.path());
        let log_root = tempfile::tempdir().expect("logdir");

        ensure_kiss_clamp_if_needed_with_details(&work, Some(log_root.path())).expect("clamp");
        let log_path = log_root.path().join(crate::artifacts::QUALITY_GATES_LOG);
        assert!(log_path.is_file(), "expected quality_gates.log");
        let content = std::fs::read_to_string(&log_path).expect("read log");
        assert!(content.contains("kiss clamp"));
    }

    #[cfg(unix)]
    #[test]
    fn kiss_clamp_fails_when_kiss_not_executable() {
        let (_tmp, work) = source_workspace_without_kissconfig();
        let bin_dir = tempfile::tempdir().expect("bindir");
        std::fs::write(bin_dir.path().join("kiss"), "#!/bin/sh\nexit 0\n").expect("fake kiss");
        let _guard = super::super::command_support::set_fake_command_dir(bin_dir.path());

        let err = ensure_kiss_clamp_if_needed_with_details(&work, None).unwrap_err();
        match err {
            RepoGateFailure::Message(message) => {
                assert!(message.contains("`kiss clamp` failed to start"));
            }
            RepoGateFailure::Command(other) => panic!("expected start failure, got {other:?}"),
        }
    }
}
