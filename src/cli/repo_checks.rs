use malvin::output::{MALVIN_WHO, print_stdout_line};
use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy)]
pub enum RepoGateOutput {
    Tagged,
    Stderr,
}

pub fn emit_repo_gate_line(output: RepoGateOutput, line: &str) {
    match output {
        RepoGateOutput::Tagged => print_stdout_line(MALVIN_WHO, line),
        RepoGateOutput::Stderr => eprintln!("{line}"),
    }
}

pub fn run_repo_workspace_gates(work_dir: &Path, output: RepoGateOutput) -> Result<(), String> {
    prepare_repo_workspace(work_dir, output)?;
    run_quality_gates(work_dir, output)
}

pub fn prepare_repo_workspace(work_dir: &Path, output: RepoGateOutput) -> Result<(), String> {
    ensure_kiss_clamp_if_needed(work_dir, output)?;
    warn_kissconfig_test_coverage_if_needed(work_dir, output);
    Ok(())
}

fn ensure_kiss_clamp_if_needed(work_dir: &Path, output: RepoGateOutput) -> Result<(), String> {
    let kissconfig = work_dir.join(".kissconfig");
    if kissconfig.exists() || !source_like_files_present(work_dir) {
        return Ok(());
    }
    emit_repo_gate_line(
        output,
        "Running `kiss clamp` (existing code without .kissconfig)",
    );
    let status = std::process::Command::new(run_command_for("kiss"))
        .arg("clamp")
        .current_dir(work_dir)
        .status()
        .map_err(|e| format!("`kiss clamp` failed to start: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("`kiss clamp` failed".to_string())
    }
}

fn source_like_files_present(root: &Path) -> bool {
    super::kiss_clamp::has_source_files(root)
}

fn run_quality_gates(work_dir: &Path, output: RepoGateOutput) -> Result<(), String> {
    if has_path(work_dir.join(".pre-commit-config.yaml")) {
        return run_pre_commit_checks(work_dir, output);
    }
    if has_path(work_dir.join(".git")) {
        run_check_command(work_dir, output, &["kiss", "check"], "kiss check")?;
    }
    if has_path(work_dir.join("Cargo.toml")) {
        run_check_command(
            work_dir,
            output,
            &[
                "cargo",
                "clippy",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
                "-W",
                "clippy::pedantic",
                "-W",
                "clippy::nursery",
                "-W",
                "clippy::cargo",
                "-A",
                "clippy::must_use_candidate",
                "-A",
                "clippy::missing_errors_doc",
                "-A",
                "clippy::missing_panics_doc",
            ],
            "cargo clippy",
        )?;
        run_check_command(work_dir, output, &["cargo", "test"], "cargo test")?;
    }
    if has_python_file(work_dir) {
        run_check_command(work_dir, output, &["ruff", "check", "."], "ruff check")?;
        if has_tests_dir(work_dir) {
            run_check_command(
                work_dir,
                output,
                &["pytest", "-sv", "tests"],
                "pytest -sv tests",
            )?;
        }
    }
    Ok(())
}

fn run_pre_commit_checks(work_dir: &Path, output: RepoGateOutput) -> Result<(), String> {
    emit_repo_gate_line(output, "Running `pre-commit run --all-files`");
    let status = Command::new(run_command_for("pre-commit"))
        .args(["run", "--all-files"])
        .current_dir(work_dir)
        .status()
        .map_err(|e| format!("`pre-commit run --all-files` failed to start: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("`pre-commit run --all-files` failed".to_string())
    }
}

fn has_path(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    path.is_file() || path.is_dir()
}

fn has_python_file(root: &Path) -> bool {
    super::kiss_clamp::has_extension_files(root, "py")
}

fn has_tests_dir(root: &Path) -> bool {
    root.join("tests").is_dir()
}

#[allow(dead_code)]
fn scan_for_extension(root: &Path, ext: &str) -> bool {
    super::kiss_clamp::has_extension_files(root, ext)
}

#[cfg(test)]
thread_local! {
    static TEST_FAKE_COMMAND_DIR: std::cell::RefCell<Option<std::path::PathBuf>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn test_fake_command_path(command: &str) -> Option<std::path::PathBuf> {
    TEST_FAKE_COMMAND_DIR
        .with(|dir| {
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
        // Protect against tests on other threads reading unrelated state.
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

fn run_check_command(
    work_dir: &Path,
    output: RepoGateOutput,
    command_args: &[&str],
    friendly_label: &str,
) -> Result<(), String> {
    if command_args.is_empty() {
        return Err("invalid check command".to_string());
    }
    emit_repo_gate_line(output, &format!("Running `{friendly_label}`"));
    let command_binary = run_command_for(command_args[0]);
    let status = Command::new(command_binary)
        .args(&command_args[1..])
        .current_dir(work_dir)
        .status()
        .map_err(|e| format!("`{friendly_label}` failed to start: {e}"))?;
    if status.success() {
        return Ok(());
    }
    Err(format!(
        "`{friendly_label}` failed (exit {}): {}",
        status
            .code()
            .map_or_else(|| "signal".to_string(), |code| code.to_string()),
        friendly_label
    ))
}

/// Touch `<work_dir>/grounding.md` and `<work_dir>/.malvin_memory/style.md` when missing.
/// Existing files are never touched.
/// Returns an error string if a file or the `.malvin_memory` directory cannot be created.
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
    use super::{
        RepoGateOutput, ensure_workspace_style_markers, prepare_repo_workspace,
        run_repo_workspace_gates, scan_for_extension, should_warn_low_test_coverage,
    };
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
        assert!(should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_warn_when_threshold_missing() {
        let v: toml::Value = toml::from_str("[gate]\nmin_similarity = 0.7\n").unwrap();
        assert!(should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_warn_when_threshold_below_90() {
        let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 89\n").unwrap();
        assert!(should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_ok_at_90() {
        let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 90\n").unwrap();
        assert!(!should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_ok_above_90() {
        let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 100\n").unwrap();
        assert!(!should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_ok_at_90_whole_float() {
        let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 90.0\n").unwrap();
        assert!(!should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_warn_when_threshold_is_fractional_float() {
        let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 90.5\n").unwrap();
        assert!(should_warn_low_test_coverage(&v));
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

        let scan = tokio::task::spawn_blocking(move || scan_for_extension(&root, "rs"));
        let found = tokio::time::timeout(Duration::from_secs(1), scan)
            .await
            .expect("scan_for_extension must finish even with symlink cycles")
            .expect("scan_for_extension panicked");

        assert!(!found);
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
            "kiss",
            &format!("#!/bin/sh\necho \"kiss $@\" >> \"{trace_for_script}\"\nexit 0\n"),
        );
        make_script(
            "ruff",
            &format!("#!/bin/sh\necho \"ruff $@\" >> \"{trace_for_script}\"\nexit 0\n"),
        );
        make_script(
            "pytest",
            &format!("#!/bin/sh\necho \"pytest $@\" >> \"{trace_for_script}\"\nexit 0\n"),
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
        assert!(log_contains_command(&log, "pytest -sv tests"));
    }

    #[cfg(unix)]
    #[test]
    fn run_repo_workspace_gates_uses_pre_commit_when_config_present() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        fs::create_dir(work.join(".git")).unwrap();
        fs::write(work.join(".pre-commit-config.yaml"), "repos:\n").unwrap();
        fs::write(work.join("Cargo.toml"), "[package]\nname = 'm'\nversion = '0.1.0'\n").unwrap();
        fs::write(work.join("main.rs"), "fn main() {}").unwrap();
        fs::write(work.join("script.py"), "print('ok')").unwrap();
        fs::create_dir(work.join("tests")).unwrap();

        let bin_dir = tempfile::tempdir().unwrap();
        let trace = bin_dir.path().join("trace.log");
        let trace_for_script = trace.to_string_lossy().to_string();
        let pre_commit = bin_dir.path().join("pre-commit");
        fs::write(
            &pre_commit,
            format!(
                "#!/bin/sh\necho \"pre-commit $@\" >> \"{trace_for_script}\"\nexit 0\n"
            ),
        )
        .unwrap();
        let mut perms = fs::metadata(&pre_commit).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_commit, perms).unwrap();
        let kiss = bin_dir.path().join("kiss");
        fs::write(
            &kiss,
            format!("#!/bin/sh\necho \"kiss $@\" >> \"{trace_for_script}\"\nexit 0\n"),
        )
        .unwrap();
        let mut kiss_perms = fs::metadata(&kiss).unwrap().permissions();
        kiss_perms.set_mode(0o755);
        fs::set_permissions(&kiss, kiss_perms).unwrap();
        let _guard = super::set_fake_command_dir(bin_dir.path());

        let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged);

        assert!(result.is_ok());
        let log = fs::read_to_string(&trace).unwrap();
        assert!(log_contains_command(&log, "pre-commit run --all-files"));
        assert!(!log_contains_command(&log, "kiss check"));
        assert!(!log_contains_command(&log, "cargo clippy"));
        assert!(!log_contains_command(&log, "ruff check"));
    }

    #[cfg(unix)]
    #[test]
    fn run_repo_workspace_gates_skips_pytest_when_tests_dir_missing() {
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
            "ruff",
            &format!("#!/bin/sh\necho \"ruff $@\" >> \"{trace_for_script}\"\nexit 0\n"),
        );
        make_script(
            "pytest",
            &format!("#!/bin/sh\necho \"pytest $@\" >> \"{trace_for_script}\"\nexit 0\n"),
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
