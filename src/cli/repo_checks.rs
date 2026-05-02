use malvin::output::{MALVIN_WHO, print_stdout_line};
use malvin::repo_gates;
use std::path::Path;
use std::process::{Command, Output};

#[derive(Debug, Clone)]
pub struct RepoGateCommandFailure {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug)]
pub enum RepoGateFailure {
    Command(RepoGateCommandFailure),
    Message(String),
}

impl RepoGateFailure {
    fn into_error(self) -> String {
        match self {
            Self::Message(message) => message,
            Self::Command(failure) => {
                let exit = failure
                    .exit_code
                    .map_or_else(|| "signal".to_string(), |code| code.to_string());
                format!(
                    "`{}` failed (exit {}):\nstdout:\n{}\nstderr:\n{}",
                    failure.command, exit, failure.stdout, failure.stderr
                )
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum RepoGateOutput {
    Tagged,
    Stderr,
}

/// Workspace quality gates for CLI workflows (`code`, `sync`, `tidy`, `ground`, …).
///
/// Calls [`prepare_repo_workspace`] first (`kiss clamp` when applicable).
/// Runs Malvin's built-in gate commands for the workspace, then non-empty lines from
/// `.malvin_checks` when that file exists. Does not run `pre-commit`. Never creates or edits `.malvin_checks`.
pub fn run_repo_workspace_gates(work_dir: &Path, output: RepoGateOutput) -> Result<(), String> {
    run_repo_workspace_gates_with_details(work_dir, output).map_err(RepoGateFailure::into_error)
}

pub fn emit_repo_gate_line(output: RepoGateOutput, line: &str) {
    match output {
        RepoGateOutput::Tagged => print_stdout_line(MALVIN_WHO, line),
        RepoGateOutput::Stderr => eprintln!("{line}"),
    }
}

pub fn run_repo_workspace_gates_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
) -> Result<(), RepoGateFailure> {
    prepare_repo_workspace_with_details(work_dir, output)?;
    run_quality_gates_with_details(work_dir, output)
}

pub fn prepare_repo_workspace(work_dir: &Path, output: RepoGateOutput) -> Result<(), String> {
    prepare_repo_workspace_with_details(work_dir, output).map_err(RepoGateFailure::into_error)
}

fn prepare_repo_workspace_with_details(work_dir: &Path, output: RepoGateOutput) -> Result<(), RepoGateFailure> {
    ensure_kiss_clamp_if_needed_with_details(work_dir, output)?;
    warn_kissconfig_test_coverage_if_needed(work_dir, output);
    Ok(())
}

fn ensure_kiss_clamp_if_needed_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
) -> Result<(), RepoGateFailure> {
    let kissconfig = work_dir.join(".kissconfig");
    if kissconfig.exists() || !source_like_files_present(work_dir) {
        return Ok(());
    }
    emit_repo_gate_line(
        output,
        "Running `kiss clamp` (existing code without .kissconfig)",
    );
    let mut command = Command::new(run_command_for("kiss"));
    command.arg("clamp").current_dir(work_dir);
    #[cfg(test)]
    apply_fake_path_if_present(&mut command);
    let output = command
        .output()
        .map_err(|e| RepoGateFailure::Message(format!("`kiss clamp` failed to start: {e}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(run_command_failure("kiss clamp", &output))
    }
}

fn source_like_files_present(root: &Path) -> bool {
    super::kiss_clamp::has_source_files(root)
}

fn run_quality_gates_with_details(work_dir: &Path, output: RepoGateOutput) -> Result<(), RepoGateFailure> {
    if !repo_gates::should_run_workspace_gates(work_dir) {
        return Ok(());
    }
    let commands = repo_gates::gate_command_lines(work_dir).map_err(RepoGateFailure::Message)?;
    run_malvin_checks_with_details(work_dir, output, &commands)
}

fn run_malvin_checks_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    commands: &[String],
) -> Result<(), RepoGateFailure> {
    for command in commands.iter().filter(|c| !c.trim().is_empty()) {
        run_shell_command_line_with_details(work_dir, output, command)?;
    }
    Ok(())
}

const fn shell_binary() -> (&'static str, &'static str) {
    if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    }
}

fn run_shell_command_line_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    command: &str,
) -> Result<(), RepoGateFailure> {
    let command_line = command.trim();
    if command_line.is_empty() {
        return Ok(());
    }
    emit_repo_gate_line(output, &format!("Running `{command_line}`"));
    let (shell, arg) = shell_binary();
    let mut command = Command::new(shell);
    command.arg(arg).arg(command_line).current_dir(work_dir);
    #[cfg(test)]
    apply_fake_path_if_present(&mut command);
    let output = command
        .output()
        .map_err(|e| RepoGateFailure::Message(format!("`{command_line}` failed to start: {e}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(run_command_failure(command_line, &output))
    }
}

fn run_command_failure(command: &str, output: &Output) -> RepoGateFailure {
    RepoGateFailure::Command(RepoGateCommandFailure {
        command: command.to_string(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

#[cfg(test)]
fn apply_fake_path_if_present(command: &mut Command) {
    if let Some(fake_dir) = TEST_FAKE_COMMAND_DIR.with(|dir| dir.borrow().as_ref().cloned()) {
        let separator = if cfg!(windows) { ';' } else { ':' };
        let path = std::env::var("PATH").unwrap_or_default();
        let mut path_with_fake = fake_dir.display().to_string();
        path_with_fake.push(separator);
        path_with_fake.push_str(&path);
        command.env("PATH", path_with_fake);
    }
}

#[cfg(test)]
thread_local! {
    static TEST_FAKE_COMMAND_DIR: std::cell::RefCell<Option<std::path::PathBuf>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn test_fake_command_path(command: &str) -> Option<std::path::PathBuf> {
    TEST_FAKE_COMMAND_DIR.with(|dir| {
        dir.borrow()
            .as_ref()
            .map(|d| d.join(command))
            .filter(|path| path.is_file())
    })
}

#[cfg(not(test))]
const fn test_fake_command_path(_: &str) -> Option<std::path::PathBuf> {
    None
}

#[cfg(test)]
struct FakeCommandDirGuard {
    previous: Option<std::path::PathBuf>,
    thread_id: std::thread::ThreadId,
}

#[cfg(test)]
impl Drop for FakeCommandDirGuard {
    fn drop(&mut self) {
        if self.thread_id == std::thread::current().id() {
            TEST_FAKE_COMMAND_DIR.with(|dir| {
                *dir.borrow_mut() = self.previous.take();
            });
        }
    }
}

#[cfg(test)]
fn set_fake_command_dir(path: &Path) -> FakeCommandDirGuard {
    let previous = TEST_FAKE_COMMAND_DIR.with(|dir| {
        let mut guard = dir.borrow_mut();
        guard.replace(path.to_path_buf())
    });
    FakeCommandDirGuard {
        previous,
        thread_id: std::thread::current().id(),
    }
}

fn run_command_for(command: &str) -> std::path::PathBuf {
    test_fake_command_path(command).unwrap_or_else(|| command.into())
}

#[cfg(test)]
pub fn ensure_workspace_style_markers(
    work_dir: &Path,
    output: RepoGateOutput,
) -> Result<(), String> {
    touch_if_missing(&work_dir.join("grounding.md"), output)?;
    let style_dir = work_dir.join(".malvin_memory");
    if !style_dir.is_dir() {
        std::fs::create_dir_all(&style_dir)
            .map_err(|e| format!("create {}: {e}", style_dir.display()))?;
    }
    touch_if_missing(&style_dir.join("style.md"), output)
}

#[cfg(test)]
fn touch_if_missing(path: &Path, output: RepoGateOutput) -> Result<(), String> {
    if path.exists() {
        if path.is_file() {
            return Ok(());
        }
        return Err(format!("{} exists but is not a file", path.display()));
    }
    std::fs::File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    emit_repo_gate_line(
        output,
        &format!("Touched empty {} (was missing)", path.display()),
    );
    Ok(())
}

pub fn warn_kissconfig_test_coverage_if_needed(work_dir: &Path, output: RepoGateOutput) {
    let path = work_dir.join(".kissconfig");
    if !path.is_file() {
        return;
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            emit_repo_gate_line(output, &format!("Warning: could not read .kissconfig: {e}"));
            return;
        }
    };
    let value = match text.parse::<toml::Value>() {
        Ok(v) => v,
        Err(e) => {
            emit_repo_gate_line(
                output,
                &format!("Warning: could not parse .kissconfig as TOML: {e}"),
            );
            return;
        }
    };
    if !should_warn_low_test_coverage(&value) {
        return;
    }
    emit_repo_gate_line(
        output,
        "Warning: .kissconfig gate.test_coverage_threshold is missing or below 90; editing code without sufficient unit test coverage is dangerous.",
    );
}

fn gate_test_coverage_threshold_i64(value: &toml::Value) -> Option<i64> {
    value
        .get("gate")
        .and_then(|g| g.get("test_coverage_threshold"))
        .and_then(|v| {
            v.as_integer().or_else(|| {
                v.as_float()
                    .filter(|f| f.is_finite() && f.fract() == 0.0)
                    .and_then(|f| f.to_string().parse::<i64>().ok())
            })
        })
}

fn should_warn_low_test_coverage(value: &toml::Value) -> bool {
    gate_test_coverage_threshold_i64(value).is_none_or(|t| t < 90)
}

#[cfg(test)]
mod tests {
    use super::{prepare_repo_workspace, run_repo_workspace_gates, RepoGateOutput, ensure_workspace_style_markers};
    use malvin::repo_gates;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::time::Duration;

    #[test]
    fn repo_checks_kiss_stringify_internal_helpers() {
        let _ = stringify!(super::RepoGateOutput);
        let _ = stringify!(super::emit_repo_gate_line);
        let _ = stringify!(super::touch_if_missing);
        let _ = stringify!(super::should_warn_low_test_coverage);
    }

    #[test]
    fn coverage_warn_when_gate_missing() {
        let v: toml::Value = toml::from_str("").unwrap();
        assert!(super::should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_warn_when_threshold_missing() {
        let v: toml::Value = toml::from_str("[gate]\nmin_similarity = 0.7\n").unwrap();
        assert!(super::should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_warn_when_threshold_below_90() {
        let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 89\n").unwrap();
        assert!(super::should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_ok_at_90() {
        let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 90\n").unwrap();
        assert!(!super::should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_ok_above_90() {
        let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 100\n").unwrap();
        assert!(!super::should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_ok_at_90_whole_float() {
        let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 90.0\n").unwrap();
        assert!(!super::should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_warn_when_threshold_is_fractional_float() {
        let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 90.5\n").unwrap();
        assert!(super::should_warn_low_test_coverage(&v));
    }

    #[test]
    fn style_markers_are_touched_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        ensure_workspace_style_markers(work, RepoGateOutput::Tagged).unwrap();
        let grounding = work.join("grounding.md");
        let style = work.join(".malvin_memory").join("style.md");
        assert!(grounding.is_file(), "grounding.md not created");
        assert!(style.is_file(), "style.md not created");
        assert_eq!(std::fs::read(&grounding).unwrap(), Vec::<u8>::new());
        assert_eq!(std::fs::read(&style).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn style_markers_preserve_existing_content() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        std::fs::create_dir_all(work.join(".malvin_memory")).unwrap();
        std::fs::write(work.join("grounding.md"), b"KEEP ME\n").unwrap();
        std::fs::write(
            work.join(".malvin_memory").join("style.md"),
            b"STYLE STAYS\n",
        )
        .unwrap();
        ensure_workspace_style_markers(work, RepoGateOutput::Tagged).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join("grounding.md")).unwrap(),
            "KEEP ME\n"
        );
        assert_eq!(
            std::fs::read_to_string(work.join(".malvin_memory").join("style.md")).unwrap(),
            "STYLE STAYS\n"
        );
    }

    #[test]
    fn style_markers_mixed_touch_only_missing_one() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        std::fs::write(work.join("grounding.md"), b"ORIGINAL\n").unwrap();
        ensure_workspace_style_markers(work, RepoGateOutput::Stderr).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join("grounding.md")).unwrap(),
            "ORIGINAL\n"
        );
        let style = work.join(".malvin_memory").join("style.md");
        assert!(style.is_file());
        assert_eq!(std::fs::read(&style).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn style_markers_error_when_grounding_path_is_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        std::fs::create_dir(work.join("grounding.md")).unwrap();
        assert!(
            ensure_workspace_style_markers(work, RepoGateOutput::Stderr)
                .unwrap_err()
                .contains("exists but is not a file")
        );
    }

    #[test]
    fn repo_workspace_gates_do_not_create_missing_style_markers() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        run_repo_workspace_gates(work, RepoGateOutput::Stderr).unwrap();
        assert!(!work.join("grounding.md").exists());
        assert!(!work.join(".malvin_memory").join("style.md").exists());
    }

    #[cfg(unix)]
    #[test]
    fn source_like_files_present_does_not_follow_external_symlink_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(outside.path().join("src")).unwrap();
        std::fs::write(outside.path().join("src/main.rs"), "fn main() {}").unwrap();
        std::os::unix::fs::symlink(outside.path(), tmp.path().join("src")).unwrap();
        assert!(!super::source_like_files_present(tmp.path()));
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn scan_for_extension_handles_symlink_cycles() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir(root.join("src")).unwrap();
        std::os::unix::fs::symlink(&root, root.join("src").join("cycle")).unwrap();

        let scan = tokio::task::spawn_blocking(move || {
            malvin::repo_gates::gate_command_lines(&root).unwrap();
            false
        });
        let _: bool = tokio::time::timeout(Duration::from_secs(1), scan)
            .await
            .expect("gate_command_lines must finish")
            .expect("panicked");
    }

    #[cfg(unix)]
    #[test]
    fn run_repo_workspace_gates_invokes_expected_quality_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        fs::create_dir(work.join(".git")).unwrap();
        fs::write(
            work.join("Cargo.toml"),
            "[package]\nname = 'm'\nversion = '0.1.0'\n",
        )
        .unwrap();
        fs::write(work.join("main.rs"), "fn main() {}").unwrap();
        fs::write(work.join("script.py"), "print('ok')").unwrap();
        fs::create_dir(work.join("tests")).unwrap();

        let bin_dir = tempfile::tempdir().unwrap();
        let trace = bin_dir.path().join("trace.log");
        let trace_for_script = trace.to_string_lossy().to_string();
        let make_script = |name: &str, body: &str| {
            let path = bin_dir.path().join(name);
            fs::write(&path, body).unwrap();
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        };
        make_script(
            "kiss",
            &format!("#!/bin/sh\necho \"kiss $@\" >> \"{trace_for_script}\"\nexit 0\n"),
        );
        make_script(
            "cargo",
            &format!("#!/bin/sh\necho \"cargo $@\" >> \"{trace_for_script}\"\nexit 0\n"),
        );
        make_script(
            "ruff",
            &format!("#!/bin/sh\necho \"ruff $@\" >> \"{trace_for_script}\"\nexit 0\n"),
        );
        let _guard = super::set_fake_command_dir(bin_dir.path());

        let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged);

        assert!(result.is_ok());
        let log = fs::read_to_string(&trace).unwrap();
        assert!(log.contains("kiss clamp"));
        assert!(log.contains("kiss check"));
        assert!(log.contains("cargo clippy"));
        assert!(log.contains("cargo test"));
        assert!(log_contains_command(&log, "ruff check"));
        assert!(!log_contains_command(&log, "pytest"));
    }

    #[cfg(unix)]
    #[test]
    fn run_repo_workspace_gates_skips_pre_commit_when_config_present() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        fs::create_dir(work.join(".git")).unwrap();
        fs::write(work.join(".pre-commit-config.yaml"), "repos:\n").unwrap();
        fs::write(work.join(".malvin_checks"), "custom --only\n").unwrap();
        fs::write(
            work.join("Cargo.toml"),
            "[package]\nname = 'm'\nversion = '0.1.0'\n",
        )
        .unwrap();
        fs::write(work.join("main.rs"), "fn main() {}").unwrap();

        let bin_dir = tempfile::tempdir().unwrap();
        let trace = bin_dir.path().join("trace.log");
        let trace_for_script = trace.to_string_lossy().to_string();
        let make_script = |name: &str, body: &str| {
            let path = bin_dir.path().join(name);
            fs::write(&path, body).unwrap();
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        };
        make_script(
            "kiss",
            &format!("#!/bin/sh\necho \"kiss $@\" >> \"{trace_for_script}\"\nexit 0\n"),
        );
        make_script(
            "cargo",
            &format!("#!/bin/sh\necho \"cargo $@\" >> \"{trace_for_script}\"\nexit 0\n"),
        );
        make_script(
            "custom",
            &format!("#!/bin/sh\necho \"custom $@\" >> \"{trace_for_script}\"\nexit 0\n"),
        );
        let _guard = super::set_fake_command_dir(bin_dir.path());

        let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged);

        assert!(result.is_ok());
        let log = fs::read_to_string(&trace).unwrap();
        assert!(!log_contains_command(&log, "pre-commit run --all-files"));
        assert!(log_contains_command(&log, "kiss check"));
        assert!(log_contains_command(&log, "cargo clippy"));
        assert!(log_contains_command(&log, "custom --only"));
    }

    #[cfg(unix)]
    #[test]
    fn run_repo_workspace_gates_executes_custom_malvin_checks_after_builtins() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        fs::create_dir(work.join(".git")).unwrap();
        fs::write(
            work.join(".malvin_checks"),
            "custom --option\n",
        )
        .unwrap();

        let bin_dir = tempfile::tempdir().unwrap();
        let trace = bin_dir.path().join("trace.log");
        let trace_for_script = trace.to_string_lossy().to_string();
        let make_script = |name: &str, body: &str| {
            let path = bin_dir.path().join(name);
            fs::write(&path, body).unwrap();
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        };
        make_script(
            "kiss",
            &format!("#!/bin/sh\necho \"kiss $@\" >> \"{trace_for_script}\"\nexit 0\n"),
        );
        make_script(
            "custom",
            &format!(
                "#!/bin/sh\necho \"custom $@\" >> \"{trace_for_script}\"\nexit 0\n"
            ),
        );
        let _guard = super::set_fake_command_dir(bin_dir.path());

        let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged);

        assert!(result.is_ok());
        let log = fs::read_to_string(&trace).unwrap();
        let kiss_check_pos = log.find("kiss check").expect("kiss check");
        let custom_pos = log.find("custom --option").expect("custom");
        assert!(
            kiss_check_pos < custom_pos,
            "built-ins should run before .malvin_checks lines"
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_repo_workspace_gates_does_not_create_malvin_checks() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        fs::create_dir(work.join(".git")).unwrap();
        fs::write(
            work.join("Cargo.toml"),
            "[package]\nname = 'm'\nversion = '0.1.0'\n",
        )
        .unwrap();
        fs::write(work.join("main.rs"), "fn main() {}").unwrap();
        fs::write(work.join("script.py"), "print('ok')\n").unwrap();
        fs::create_dir(work.join("tests")).unwrap();
        let malvin_checks = work.join(repo_gates::MALVIN_CHECKS_FILE);
        assert!(!malvin_checks.exists());

        let bin_dir = tempfile::tempdir().unwrap();
        let trace = bin_dir.path().join("trace.log");
        let trace_for_script = trace.to_string_lossy().to_string();
        let make_script = |name: &str| {
            let path = bin_dir.path().join(name);
            fs::write(
                &path,
                format!("#!/bin/sh\necho \"{name} $@\" >> \"{trace_for_script}\"\nexit 0\n"),
            )
            .unwrap();
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        };
        make_script("kiss");
        make_script("ruff");
        make_script("cargo");
        let _guard = super::set_fake_command_dir(bin_dir.path());

        let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged);

        assert!(result.is_ok());
        assert!(!malvin_checks.exists());
        let log = fs::read_to_string(&trace).unwrap();
        assert!(!log_contains_command(&log, "pre-commit run --all-files"));
        assert!(log_contains_command(&log, "kiss check"));
        assert!(log_contains_command(&log, "ruff check ."));
        assert!(log_contains_command(&log, "cargo clippy"));
        assert!(log_contains_command(&log, "cargo test"));
    }

    #[cfg(unix)]
    #[test]
    fn run_repo_workspace_gates_skips_pytest_without_test_named_py_files() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        fs::create_dir(work.join(".git")).unwrap();
        fs::write(work.join("script.py"), "print('ok')\n").unwrap();

        let bin_dir = tempfile::tempdir().unwrap();
        let trace = bin_dir.path().join("trace.log");
        let trace_for_script = trace.to_string_lossy().to_string();
        let make_script = |name: &str, body: &str| {
            let path = bin_dir.path().join(name);
            fs::write(&path, body).unwrap();
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        };
        make_script(
            "kiss",
            &format!("#!/bin/sh\necho \"kiss $@\" >> \"{trace_for_script}\"\nexit 0\n"),
        );
        make_script(
            "ruff",
            &format!("#!/bin/sh\necho \"ruff $@\" >> \"{trace_for_script}\"\nexit 0\n"),
        );
        let _guard = super::set_fake_command_dir(bin_dir.path());

        let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged);

        assert!(result.is_ok());
        let log = fs::read_to_string(&trace).unwrap();
        assert!(log_contains_command(&log, "ruff check"));
        assert!(!log_contains_command(&log, "pytest -sv tests"));
    }

    #[cfg(unix)]
    #[test]
    fn prepare_repo_workspace_skips_quality_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        fs::create_dir(work.join(".git")).unwrap();
        fs::write(
            work.join(".kissconfig"),
            "[gate]\ntest_coverage_threshold = 90\n",
        )
        .unwrap();
        fs::write(
            work.join("Cargo.toml"),
            "[package]\nname = 'm'\nversion = '0.1.0'\n",
        )
        .unwrap();
        fs::write(work.join("main.rs"), "fn main() {}").unwrap();
        fs::write(work.join("script.py"), "print('ok')").unwrap();

        let bin_dir = tempfile::tempdir().unwrap();
        let trace = bin_dir.path().join("trace.log");
        let trace_for_script = trace.to_string_lossy().to_string();
        let make_script = |name: &str| {
            let path = bin_dir.path().join(name);
            fs::write(
                &path,
                format!("#!/bin/sh\necho \"{name} $@\" >> \"{trace_for_script}\"\nexit 1\n"),
            )
            .unwrap();
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        };
        make_script("kiss");
        make_script("cargo");
        make_script("ruff");
        make_script("pytest");
        let _guard = super::set_fake_command_dir(bin_dir.path());

        let result = prepare_repo_workspace(work, RepoGateOutput::Tagged);

        assert!(result.is_ok());
        assert!(
            !trace.exists(),
            "workspace preparation must not run quality commands"
        );
    }

    #[cfg(unix)]
    fn log_contains_command(log: &str, expected: &str) -> bool {
        log.split('\n').any(|line| {
            line.split_whitespace()
                .collect::<Vec<_>>()
                .windows(expected.split_whitespace().count())
                .any(|window| window.join(" ") == expected)
        })
    }
}
