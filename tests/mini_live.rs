//! Opt-in live integration tests for `--mini` (`OpenRouter` + bash loop).
//!
//! Run manually:
//! ```text
//! MALVIN_LIVE_MINI=1 cargo nextest run mini_live -- --ignored
//! ```
//!
//! Requires: `OPENROUTER_API_KEY`, `bash` on `PATH`, and network access.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
use common::{command_output_mini_live, only_run_dir, test_home_workspace, MALVIN_TEST_CMD_TIMEOUT};

#[cfg(unix)]
fn mini_live_prereqs_met() -> bool {
    std::env::var_os("MALVIN_LIVE_MINI").is_some_and(|v| v == "1")
        && std::env::var_os("OPENROUTER_API_KEY").is_some_and(|v| !v.is_empty())
        && Command::new("bash")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
}

#[cfg(unix)]
fn malvin_bin() -> String {
    std::env::var("MALVIN_BIN").unwrap_or_else(|_| env!("CARGO_BIN_EXE_malvin").to_string())
}

#[cfg(unix)]
fn run_mini_live_in_workspace(args: &[&str]) -> (tempfile::TempDir, std::process::Output) {
    let (root, home, workspace) = test_home_workspace();
    let old_home = std::env::var_os("HOME");
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("HOME", &home);
    }
    let output = command_output_mini_live(
        Command::new(malvin_bin())
            .current_dir(&workspace)
            .args(args),
    )
    .expect("malvin --mini live");
    #[allow(unsafe_code)]
    unsafe {
        match old_home {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
    }
    (root, output)
}

#[cfg(unix)]
#[test]
#[ignore = "live OpenRouter e2e; MALVIN_LIVE_MINI=1 cargo nextest run mini_live -- --ignored"]
fn mini_live_do_echo() {
    if !mini_live_prereqs_met() {
        eprintln!("skip: set MALVIN_LIVE_MINI=1 and OPENROUTER_API_KEY to run");
        return;
    }
    let (root, output) = run_mini_live_in_workspace(&[
        "do",
        "--mini",
        "--no-tee",
        "--no-markdown",
        "--max-acp-retries",
        "1",
        "run echo hello in bash",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let workspace = root.path().join("workspace");
    let home = root.path().join("home");
    let run_dir = only_run_dir(&workspace, &home);
    assert!(run_dir.join("prompts.log").is_file());
}

#[cfg(unix)]
#[test]
#[ignore = "live OpenRouter e2e; MALVIN_LIVE_MINI=1 cargo nextest run mini_live -- --ignored"]
fn mini_live_kpop_exp_log() {
    if !mini_live_prereqs_met() {
        eprintln!("skip: set MALVIN_LIVE_MINI=1 and OPENROUTER_API_KEY to run");
        return;
    }
    let (root, output) = run_mini_live_in_workspace(&[
        "kpop",
        "--mini",
        "--no-tee",
        "--max-loops",
        "1",
        "--max-hypotheses",
        "1",
        "--max-acp-retries",
        "1",
        "why is the sky blue?",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let workspace = root.path().join("workspace");
    let home = root.path().join("home");
    let run_dir = only_run_dir(&workspace, &home);
    assert!(run_dir.join("_kpop").is_dir());
}

#[cfg(unix)]
#[test]
#[ignore = "live OpenRouter models listing; MALVIN_LIVE_MINI=1 cargo nextest run mini_live -- --ignored"]
fn mini_live_models_listing() {
    if !mini_live_prereqs_met() {
        eprintln!("skip: set MALVIN_LIVE_MINI=1 and OPENROUTER_API_KEY to run");
        return;
    }
    let (_root, output) = run_mini_live_in_workspace(&["models", "--mini", "--no-color"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("anthropic/"));
    assert!(stdout.contains("Default mini model: nvidia/nemotron-3-ultra-550b-a55b:free"));
}

#[cfg(unix)]
#[test]
fn mini_live_tests_compile_and_skip_without_env() {
    assert!(!mini_live_prereqs_met() || std::env::var_os("OPENROUTER_API_KEY").is_some());
    let _ = MALVIN_TEST_CMD_TIMEOUT;
}
