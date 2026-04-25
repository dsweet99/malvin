mod common;

use std::path::Path;
use std::process::Command;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const INIT_TEMPLATE_GITIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/gitignore"
));
#[cfg(unix)]
use common::{
    acp_mock_code_streaming_bold_markdown_js, acp_mock_code_streaming_update_js,
    command_output_with_timeout, test_home_workspace, write_fake_kiss, write_mock_executable,
    MALVIN_TEST_CMD_TIMEOUT,
};

#[cfg(unix)]
const MAX_LOOPS_EXHAUSTED: &str = "Did not receive LGTM for review_1.md within max loops.";

fn check_ignored(repo: &Path, rel_path: &str) -> bool {
    Command::new("git")
        .current_dir(repo)
        .args(["check-ignore", "-q", rel_path])
        .status()
        .unwrap_or_else(|e| panic!("git check-ignore spawn failed: {e}"))
        .success()
}

#[cfg(unix)]
fn run_code_max_loops_zero_with_mock_opts(no_tee: bool) -> std::process::Output {
    let (root, home, workspace) = test_home_workspace();
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let mock = root.path().join("mock-agent-acp-code");
    write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
    let kiss = bin_dir.join("kiss");
    write_fake_kiss(&kiss);
    let original_path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}:{original_path}", bin_dir.display());
    let mut args = vec!["code", "--trust-the-plan", "--no-learn", "--max-loops", "0", "ship it"];
    if no_tee {
        args.insert(0, "--no-tee");
    }
    command_output_with_timeout(
        Command::new(env!("CARGO_BIN_EXE_malvin"))
            .current_dir(&workspace)
            .env("HOME", &home)
            .env("CURSOR_AGENT_API_KEY", "test-key")
            .env("MALVIN_AGENT_ACP_BIN", &mock)
            .env("PATH", path)
            .args(args),
        MALVIN_TEST_CMD_TIMEOUT,
    )
    .expect("spawn malvin code")
}

#[cfg(unix)]
fn run_code_max_loops_zero_with_mock() -> std::process::Output {
    run_code_max_loops_zero_with_mock_opts(true)
}

#[cfg(unix)]
fn run_code_max_loops_zero_with_mock_stdout() -> std::process::Output {
    run_code_max_loops_zero_with_mock_opts(false)
}

#[test]
#[cfg(unix)]
fn max_loops_zero_skips_review_attempts_and_fails() {
    let out = run_code_max_loops_zero_with_mock();
    assert!(!out.status.success(), "malvin code unexpectedly succeeded: {out:?}");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains(MAX_LOOPS_EXHAUSTED),
        "expected max_loops=0 review skip failure: {combined:?}"
    );
    assert_eq!(
        combined.matches("Implement").count(),
        1,
        "expected one implement phase: {combined:?}"
    );
    assert!(
        !combined.contains("Review-1 (attempt 1)"),
        "review attempt must not run when --max-loops=0: {combined:?}"
    );
}

#[test]
#[cfg(unix)]
fn code_stdout_shows_plain_output_without_jsonrpc_lines() {
    let out = run_code_max_loops_zero_with_mock_stdout();
    assert!(!out.status.success(), "expected max-loops failure path: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("agent message"),
        "expected parsed agent output on stdout: {stdout:?}"
    );
    assert!(
        !stdout.contains("\"jsonrpc\""),
        "stdout leaked JSON-RPC protocol lines: {stdout:?}"
    );
}

#[cfg(all(unix, target_os = "linux"))]
fn run_code_max_loops_zero_under_script(extra_args: &[&str]) -> std::process::Output {
    let (root, home, workspace) = test_home_workspace();
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let mock = root.path().join("mock-agent-acp-code-md");
    write_mock_executable(&mock, &acp_mock_code_streaming_bold_markdown_js());
    let kiss = bin_dir.join("kiss");
    write_fake_kiss(&kiss);
    let malvin = env!("CARGO_BIN_EXE_malvin");
    let sh = root.path().join("run-code.sh");
    let mut args_line = String::from("code --trust-the-plan --no-learn --max-loops 0 ship");
    for a in extra_args {
        args_line.push(' ');
        args_line.push_str(a);
    }
    let body = format!(
        "#!/bin/sh\nunset NO_COLOR\nexport PATH=\"{}:$PATH\"\nexport HOME=\"{}\"\nexport CURSOR_AGENT_API_KEY=test\nexport MALVIN_AGENT_ACP_BIN=\"{}\"\ncd \"{}\"\nexec \"{}\" {}\n",
        bin_dir.display(),
        home.display(),
        mock.display(),
        workspace.display(),
        malvin,
        args_line
    );
    std::fs::write(&sh, body).expect("write run-code.sh");
    let mut perms = std::fs::metadata(&sh).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&sh, perms).expect("chmod");
    let mut cmd = Command::new("script");
    cmd.args([
        "-q",
        "-e",
        "-c",
        sh.to_str().expect("run-code.sh utf8"),
        "/dev/null",
    ]);
    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("script malvin code")
}

#[test]
#[cfg(all(unix, target_os = "linux"))]
fn code_pty_markdown_strips_bold_markers_without_no_markdown() {
    let out = run_code_max_loops_zero_under_script(&[]);
    assert!(
        !out.status.success(),
        "expected max-loops failure exit from script -e: {out:?}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("**boldline**"),
        "expected termimad to consume ** markers on TTY stdout: {stdout:?}"
    );
    assert!(
        stdout.contains("\x1b[1m"),
        "expected termimad bold ANSI on TTY stdout: {stdout:?}"
    );
    assert!(
        !stdout.contains("\"jsonrpc\""),
        "stdout leaked JSON-RPC protocol lines: {stdout:?}"
    );
}

#[test]
#[cfg(all(unix, target_os = "linux"))]
fn code_pty_no_markdown_preserves_bold_markers() {
    let out = run_code_max_loops_zero_under_script(&["--no-markdown"]);
    assert!(
        !out.status.success(),
        "expected max-loops failure exit from script -e: {out:?}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("**boldline**"),
        "expected plain stdout to preserve markdown markers: {stdout:?}"
    );
    assert!(
        !stdout.contains("\"jsonrpc\""),
        "stdout leaked JSON-RPC protocol lines: {stdout:?}"
    );
}

#[test]
fn help_lists_global_no_markdown_once() {
    #[cfg(unix)]
    let out = {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
        cmd.arg("--help");
        command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("malvin --help")
    };
    #[cfg(not(unix))]
    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .arg("--help")
        .output()
        .expect("malvin --help");
    assert!(
        out.status.success(),
        "help failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        s.matches("--no-markdown").count(),
        1,
        "expected exactly one --no-markdown in root help: {s}"
    );
}

#[test]
fn root_gitignore_ignores_malvin_logs_and_target() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        check_ignored(root, "_malvin/dummy_stamp/plan.md"),
        "expected _malvin/ run dirs to be ignored"
    );
    assert!(
        check_ignored(root, "log"),
        "expected root log file to be ignored"
    );
    assert!(
        check_ignored(root, "log_2"),
        "expected root log_2 to be ignored"
    );
    assert!(
        check_ignored(root, "target/debug/malvin"),
        "expected Rust target/ tree to be ignored"
    );
    assert!(
        !check_ignored(root, "README.md"),
        "expected README.md not to be ignored"
    );
}

#[test]
fn init_template_gitignore_is_consistent_with_git_check_ignore() {
    const TEMPLATE: &str = INIT_TEMPLATE_GITIGNORE;
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join(".gitignore"), TEMPLATE).unwrap();
    let st = Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .status()
        .expect("git init");
    assert!(st.success(), "git init failed");
    assert!(
        check_ignored(tmp.path(), "_malvin/x/plan.md"),
        "template should ignore _malvin/ runs"
    );
    assert!(
        check_ignored(tmp.path(), "log"),
        "template should ignore root log"
    );
    assert!(
        check_ignored(tmp.path(), "log_2"),
        "template should ignore root log_2"
    );
    assert!(
        check_ignored(tmp.path(), "target/release/foo"),
        "template should ignore Rust target/"
    );
    assert!(
        !check_ignored(tmp.path(), "src/lib.rs"),
        "template should not ignore normal sources"
    );
    assert!(
        check_ignored(tmp.path(), "pkg/__pycache__/x.py"),
        "template should ignore sources under nested __pycache__ dirs (not only *.pyc)"
    );
    assert!(
        check_ignored(tmp.path(), "lib/foo.pyc"),
        "template should ignore .pyc via **/*.py[cod]"
    );
}

