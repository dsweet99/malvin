mod common;

#[cfg(all(unix, target_os = "linux"))]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
#[cfg(all(unix, target_os = "linux"))]
use std::path::PathBuf;
use std::process::Command;
#[cfg(all(unix, target_os = "linux"))]
use std::process::Output;

const INIT_TEMPLATE_GITIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/gitignore"
));
#[cfg(unix)]
use common::{
    MALVIN_TEST_CMD_TIMEOUT, acp_mock_code_abort_after_implement_js,
    acp_mock_code_abort_result_after_check_plan_lgtm_js,
    acp_mock_code_check_plan_tampers_grounding_then_implement_verifies_restore_js,
    acp_mock_code_review_lgtm_to_artifact_js, acp_mock_code_streaming_update_js,
    command_output_with_timeout,
    test_home_workspace, write_fake_kiss, write_mock_executable,
};
#[cfg(all(unix, target_os = "linux"))]
use common::{
    acp_mock_code_streaming_bold_markdown_js, acp_mock_code_streaming_long_bold_markdown_js,
    acp_mock_code_streaming_rich_markdown_js, acp_mock_code_check_sync_then_review_lgtm_js,
};

#[cfg(unix)]
const MAX_LOOPS_EXHAUSTED: &str = "Did not receive LGTM for review_1.md within max loops.";

#[cfg(all(unix, target_os = "linux"))]
struct PtyRun {
    _root: tempfile::TempDir,
    workspace: PathBuf,
    output: Output,
}

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
    run_code_with_mock_js(
        &acp_mock_code_streaming_update_js(),
        &["--max-loops", "0"],
        no_tee,
    )
}

#[cfg(unix)]
fn run_code_with_mock_js(mock_js: &str, extra_args: &[&str], no_tee: bool) -> std::process::Output {
    run_code_with_mock_js_trust_plan(mock_js, extra_args, no_tee, true)
}

#[cfg(unix)]
fn run_code_with_mock_js_trust_plan(
    mock_js: &str,
    extra_args: &[&str],
    no_tee: bool,
    trust_plan: bool,
) -> std::process::Output {
    let (root, home, workspace) = test_home_workspace();
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let mock = root.path().join("mock-agent-acp-code");
    write_mock_executable(&mock, mock_js);
    let kiss = bin_dir.join("kiss");
    write_fake_kiss(&kiss);
    let original_path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}:{original_path}", bin_dir.display());
    let mut args = vec!["code", "--no-learn"];
    if trust_plan {
        args.push("--trust-the-plan");
    }
    args.extend_from_slice(extra_args);
    args.push("ship it");
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

#[cfg(unix)]
fn run_sync_with_mock_js(
    mock_js: &str,
    extra_args: &[&str],
    no_tee: bool,
) -> std::process::Output {
    run_sync_with_mock_js_and_workspace(mock_js, extra_args, no_tee).0
}

#[cfg(unix)]
fn run_sync_with_mock_js_and_workspace(
    mock_js: &str,
    extra_args: &[&str],
    no_tee: bool,
) -> (std::process::Output, tempfile::TempDir, PathBuf) {
    let (root, home, workspace) = common::test_home_workspace();
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let mock = root.path().join("mock-agent-acp-sync");
    common::write_mock_executable(&mock, mock_js);
    let kiss = bin_dir.join("kiss");
    common::write_fake_kiss(&kiss);
    let original_path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}:{original_path}", bin_dir.display());
    let mut args = vec!["sync", "--no-learn"];
    args.extend_from_slice(extra_args);
    if no_tee {
        args.insert(0, "--no-tee");
    }
    let out = command_output_with_timeout(
        Command::new(env!("CARGO_BIN_EXE_malvin"))
            .current_dir(&workspace)
            .env("HOME", &home)
            .env("CURSOR_AGENT_API_KEY", "test-key")
            .env("MALVIN_AGENT_ACP_BIN", &mock)
            .env("PATH", path)
            .args(args),
        MALVIN_TEST_CMD_TIMEOUT,
    )
    .expect("spawn malvin sync");
    (out, root, workspace)
}

#[cfg(unix)]
fn run_sync_with_mock_js_max_loops_zero() -> std::process::Output {
    run_sync_with_mock_js(
        &common::acp_mock_code_streaming_update_js(),
        &["--max-loops", "0"],
        true,
    )
}

#[test]
#[cfg(unix)]
fn code_stops_when_implement_writes_abort_result() {
    let out = run_code_with_mock_js(
        &acp_mock_code_abort_after_implement_js(),
        &["--max-loops", "1"],
        true,
    );
    assert!(
        !out.status.success(),
        "expected ABORT failure path: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("ABORT: stop now"),
        "expected implement ABORT to stop the workflow: {combined:?}"
    );
    assert!(
        !combined.contains(MAX_LOOPS_EXHAUSTED),
        "workflow should stop on ABORT before review exhaustion: {combined:?}"
    );
}

#[test]
#[cfg(unix)]
fn code_stops_when_check_plan_writes_abort_result_with_lgtm_review() {
    let out = run_code_with_mock_js_trust_plan(
        &acp_mock_code_abort_result_after_check_plan_lgtm_js(),
        &["--max-loops", "1"],
        true,
        false,
    );
    assert!(
        !out.status.success(),
        "expected ABORT failure path: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("ABORT: after check plan"),
        "expected check_plan ABORT to stop the workflow: {combined:?}"
    );
    assert!(
        !combined.contains("implement_phase_ran"),
        "implement must not run after ABORT in result.md from check_plan: {combined:?}"
    );
}

#[test]
#[cfg(unix)]
fn check_plan_grounding_restore_happens_before_implement() {
    let out = run_code_with_mock_js_trust_plan(
        &acp_mock_code_check_plan_tampers_grounding_then_implement_verifies_restore_js(),
        &["--max-loops", "0"],
        false,
        false,
    );
    assert!(
        !out.status.success(),
        "expected max-loops failure path: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("implement ok"),
        "expected implement to see restored grounding: {combined:?}"
    );
    assert!(
        !combined.contains("ABORT: grounding leaked into implement"),
        "check_plan grounding mutation must not leak into implement: {combined:?}"
    );
    assert!(
        combined.contains(MAX_LOOPS_EXHAUSTED),
        "expected workflow to continue past implement into normal max-loops failure: {combined:?}"
    );
}

#[test]
#[cfg(unix)]
fn max_loops_zero_skips_review_attempts_and_fails() {
    let out = run_code_max_loops_zero_with_mock();
    assert!(
        !out.status.success(),
        "malvin code unexpectedly succeeded: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains(MAX_LOOPS_EXHAUSTED),
        "expected max_loops=0 review skip failure: {combined:?}"
    );
    assert!(
        !combined.contains("Review-1 (attempt 1)"),
        "review attempt must not run when --max-loops=0: {combined:?}"
    );
}

#[test]
#[cfg(unix)]
fn review_loop_accepts_lgtm_written_to_artifact_path() {
    let out = run_code_with_mock_js(
        &acp_mock_code_review_lgtm_to_artifact_js(),
        &["--max-loops", "1"],
        true,
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains(MAX_LOOPS_EXHAUSTED),
        "review loop should accept LGTM from artifact path: {combined:?}"
    );
    assert!(
        out.status.success(),
        "malvin code should succeed when reviewer writes LGTM to artifact: {combined:?}"
    );
}

#[test]
#[cfg(unix)]
fn sync_accepts_review_lgtm_written_to_artifact_path() {
    let out = run_sync_with_mock_js(
        &common::acp_mock_code_review_lgtm_to_artifact_js(),
        &["--max-loops", "1"],
        true,
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains(MAX_LOOPS_EXHAUSTED),
        "sync should succeed with LGTM from artifact: {combined:?}"
    );
    assert!(
        out.status.success(),
        "malvin sync should succeed when review writes LGTM: {combined:?}"
    );
}

#[test]
#[cfg(unix)]
fn sync_max_loops_zero_skips_review_attempts_and_fails() {
    let out = run_sync_with_mock_js_max_loops_zero();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.status.success(), "sync should fail without reviews: {combined:?}");
    assert!(
        combined.contains("Did not receive LGTM for check_sync.md within max loops.")
            || combined.contains(MAX_LOOPS_EXHAUSTED),
        "expected max_loops skip failure: {combined:?}"
    );
    assert!(
        !combined.contains("Review-1 (attempt 1)"),
        "review attempt must not run when --max-loops=0: {combined:?}"
    );
}

#[test]
#[cfg(unix)]
fn sync_runs_check_sync_before_review_1() {
    let (out, _root, workspace) = run_sync_with_mock_js_and_workspace(
        &acp_mock_code_check_sync_then_review_lgtm_js(),
        &["--max-loops", "2"],
        true,
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "sync should succeed when check_sync and review_1 both hit LGTM: {combined:?}"
    );

    let check_sync_index = combined
        .find("CheckSync (attempt 1)")
        .expect("check_sync progress line");
    let review_index = combined
        .find("Review-1 (attempt 1)")
        .expect("review-1 progress line");
    assert!(
        check_sync_index < review_index,
        "expected check_sync to run before review_1: {combined:?}"
    );
    let run_dir = only_run_dir(&workspace);
    let has_check_sync_log = std::fs::read_dir(&run_dir)
        .expect("run dir")
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().contains("coder_check_sync"));
    assert!(
        has_check_sync_log,
        "expected check_sync coder log to capture session/prompt request: {combined:?}"
    );
}

#[test]
#[cfg(unix)]
fn code_stdout_shows_plain_output_without_jsonrpc_lines() {
    let out = run_code_max_loops_zero_with_mock_stdout();
    assert!(
        !out.status.success(),
        "expected max-loops failure path: {out:?}"
    );
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
fn run_malvin_under_script_with_mock(
    mock_js: &str,
    malvin_args_line: &str,
    columns: Option<&str>,
) -> PtyRun {
    let (root, home, workspace) = test_home_workspace();
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let mock = root.path().join("mock-agent-acp-md");
    write_mock_executable(&mock, mock_js);
    let kiss = bin_dir.join("kiss");
    write_fake_kiss(&kiss);
    let malvin = env!("CARGO_BIN_EXE_malvin");
    let sh = root.path().join("run-malvin.sh");
    let columns_export = columns
        .map(|value| format!("export COLUMNS=\"{value}\"\n"))
        .unwrap_or_default();
    let body = format!(
        "#!/bin/sh\nunset NO_COLOR\nexport PATH=\"{}:$PATH\"\nexport HOME=\"{}\"\nexport CURSOR_AGENT_API_KEY=test\nexport MALVIN_AGENT_ACP_BIN=\"{}\"\n{}cd \"{}\"\nexec \"{}\" {}\n",
        bin_dir.display(),
        home.display(),
        mock.display(),
        columns_export,
        workspace.display(),
        malvin,
        malvin_args_line
    );
    std::fs::write(&sh, body).expect("write run-malvin.sh");
    let mut perms = std::fs::metadata(&sh).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&sh, perms).expect("chmod");
    let mut cmd = Command::new("script");
    cmd.args([
        "-q",
        "-e",
        "-c",
        sh.to_str().expect("run-malvin.sh utf8"),
        "/dev/null",
    ]);
    cmd.stdin(std::process::Stdio::null());
    let output =
        command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("script malvin");
    PtyRun {
        _root: root,
        workspace,
        output,
    }
}

#[cfg(all(unix, target_os = "linux"))]
fn run_malvin_under_script(malvin_args_line: &str) -> std::process::Output {
    run_malvin_under_script_with_mock(
        &acp_mock_code_streaming_bold_markdown_js(),
        malvin_args_line,
        None,
    )
    .output
}

#[cfg(all(unix, target_os = "linux"))]
fn run_code_max_loops_zero_under_script(extra_args: &[&str]) -> std::process::Output {
    let mut args_line = String::from("code --trust-the-plan --no-learn --max-loops 0 ship");
    for a in extra_args {
        args_line.push(' ');
        args_line.push_str(a);
    }
    run_malvin_under_script(&args_line)
}

#[cfg(all(unix, target_os = "linux"))]
fn run_kpop_catchup_under_script(extra_args: &[&str]) -> std::process::Output {
    let mut args_line =
        String::from("kpop --no-learn --p-creative 0 --max-hypotheses 50 investigate");
    for a in extra_args {
        args_line.push(' ');
        args_line.push_str(a);
    }
    run_malvin_under_script(&args_line)
}

#[cfg(all(unix, target_os = "linux"))]
fn run_do_under_script(global_lead: &[&str]) -> std::process::Output {
    let mut args_line = global_lead.join(" ");
    if !args_line.is_empty() {
        args_line.push(' ');
    }
    args_line.push_str("do \"say hi\"");
    run_malvin_under_script(&args_line)
}

#[cfg(all(unix, target_os = "linux"))]
fn only_run_dir(workspace: &Path) -> PathBuf {
    let run_root = workspace.join("_malvin");
    let dirs: Vec<PathBuf> = std::fs::read_dir(&run_root)
        .expect("read _malvin")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.is_dir())
        .collect();
    assert_eq!(dirs.len(), 1, "expected exactly one run dir, got {dirs:?}");
    dirs.into_iter().next().expect("run dir")
}

#[cfg(all(unix, target_os = "linux"))]
fn read_all_logs(run_dir: &Path) -> String {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(run_dir)
        .expect("read run dir")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "log"))
        .collect();
    paths.sort();
    let chunks: Vec<String> = paths
        .into_iter()
        .map(|path| std::fs::read_to_string(path).expect("read log"))
        .collect();
    chunks.join("\n")
}

#[cfg(all(unix, target_os = "linux"))]
fn assert_markdown_stdout_and_logs(run: &PtyRun, failure_context: &str) {
    assert!(
        !run.output.status.success(),
        "{failure_context}: {:?}",
        run.output
    );
    let stdout = String::from_utf8_lossy(&run.output.stdout);
    assert!(
        !stdout.contains("# md-heading-xyz"),
        "expected stdout markdown rendering to consume heading markers: {stdout:?}"
    );
    assert!(
        stdout.contains("md-item-xyz"),
        "expected stdout markdown rendering to keep list item text visible: {stdout:?}"
    );
    assert!(
        !stdout.contains("**md-bold-xyz**"),
        "expected styled stdout to consume bold markers: {stdout:?}"
    );
    assert!(
        stdout.contains("\x1b[1m"),
        "expected bold ANSI on TTY stdout: {stdout:?}"
    );
    assert!(
        !stdout.contains("\"jsonrpc\""),
        "stdout leaked JSON-RPC protocol lines: {stdout:?}"
    );
    let run_dir = only_run_dir(&run.workspace);
    let logs = read_all_logs(&run_dir);
    assert!(
        logs.contains("# md-heading-xyz"),
        "expected raw heading markdown in logs: {logs:?}"
    );
    assert!(
        logs.contains("- md-item-xyz"),
        "expected raw list markdown in logs: {logs:?}"
    );
    assert!(
        logs.contains("**md-bold-xyz**"),
        "expected raw bold markdown in logs: {logs:?}"
    );
    assert!(
        !logs.contains("\x1b[1m"),
        "run logs must stay raw without ANSI styling: {logs:?}"
    );
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
#[cfg(all(unix, target_os = "linux"))]
fn code_pty_no_color_disables_markdown_styling() {
    let out = run_code_max_loops_zero_under_script(&["--no-color"]);
    assert!(
        !out.status.success(),
        "expected max-loops failure exit from script -e: {out:?}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("**boldline**"),
        "expected --no-color to leave markdown markers plain on TTY stdout: {stdout:?}"
    );
    assert!(
        !stdout.contains("\x1b[1m"),
        "expected --no-color to suppress ANSI styling on TTY stdout: {stdout:?}"
    );
    assert!(
        !stdout.contains("\"jsonrpc\""),
        "stdout leaked JSON-RPC protocol lines: {stdout:?}"
    );
}

#[test]
#[cfg(all(unix, target_os = "linux"))]
fn code_stdout_markdown_styles_stdout_but_logs_stay_raw() {
    let run = run_malvin_under_script_with_mock(
        &acp_mock_code_streaming_rich_markdown_js(),
        "code --trust-the-plan --no-learn --max-loops 0 ship",
        None,
    );
    assert_markdown_stdout_and_logs(&run, "expected max-loops failure exit from script -e");
}

#[test]
#[cfg(all(unix, target_os = "linux"))]
fn kpop_stdout_markdown_styles_stdout_but_logs_stay_raw() {
    let run = run_malvin_under_script_with_mock(
        &acp_mock_code_streaming_rich_markdown_js(),
        "kpop --no-learn --p-creative 0 --max-hypotheses 50 investigate",
        None,
    );
    assert_markdown_stdout_and_logs(&run, "expected kpop catch-up failure exit from script -e");
}

#[test]
#[cfg(all(unix, target_os = "linux"))]
fn code_stdout_markdown_wrap_keeps_long_bold_span_styled() {
    let run = run_malvin_under_script_with_mock(
        &acp_mock_code_streaming_long_bold_markdown_js(),
        "code --trust-the-plan --no-learn --max-loops 0 ship",
        Some("40"),
    );
    assert!(
        !run.output.status.success(),
        "expected max-loops failure exit from script -e: {:?}",
        run.output
    );
    let stdout = String::from_utf8_lossy(&run.output.stdout);
    assert!(
        stdout.contains("\x1b[1m"),
        "expected bold ANSI on wrapped TTY stdout: {stdout:?}"
    );
    assert!(
        !stdout.contains("**wrap-bold-xyz"),
        "expected wrapped stdout to avoid leaking opening bold markers: {stdout:?}"
    );
    assert!(
        !stdout.contains("wrap-bold-xyz**"),
        "expected wrapped stdout to avoid leaking closing bold markers: {stdout:?}"
    );
    let run_dir = only_run_dir(&run.workspace);
    let logs = read_all_logs(&run_dir);
    assert!(
        logs.contains("**wrap-bold-xyz wrap-bold-xyz"),
        "expected raw wrapped-bold markdown in logs: {logs:?}"
    );
    assert!(
        !logs.contains("\x1b[1m"),
        "run logs must stay raw without ANSI styling: {logs:?}"
    );
}

#[test]
#[cfg(all(unix, target_os = "linux"))]
fn kpop_pty_markdown_strips_bold_markers_without_no_markdown() {
    let out = run_kpop_catchup_under_script(&[]);
    assert!(
        !out.status.success(),
        "expected kpop catch-up failure exit from script -e: {out:?}"
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
fn kpop_pty_no_markdown_preserves_bold_markers() {
    let out = run_kpop_catchup_under_script(&["--no-markdown"]);
    assert!(
        !out.status.success(),
        "expected kpop catch-up failure exit from script -e: {out:?}"
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
#[cfg(all(unix, target_os = "linux"))]
fn kpop_timing_uses_kpop_label_not_implement() {
    let run = run_malvin_under_script_with_mock(
        &acp_mock_code_streaming_update_js(),
        "kpop --no-learn --p-creative 0 --max-hypotheses 1 investigate",
        None,
    );
    assert!(
        !run.output.status.success(),
        "expected kpop failure exit from script -e: {:?}",
        run.output
    );
    let stdout = String::from_utf8_lossy(&run.output.stdout);
    assert!(
        stdout.contains("TIMING: "),
        "expected timing summary: {stdout:?}"
    );
    assert!(
        stdout.contains("kpop = "),
        "expected kpop timing label: {stdout:?}"
    );
    assert!(
        !stdout.contains("implement = "),
        "did not expect implement timing label in kpop output: {stdout:?}"
    );
    let run_dir = only_run_dir(&run.workspace);
    let timing_path = run_dir.join("run_timing.json");
    let timing_text = std::fs::read_to_string(&timing_path).expect("read run_timing.json");
    assert!(
        timing_text.contains("\"implement\": \"kpop\""),
        "expected kpop alias in run_timing.json: {timing_text:?}"
    );
    assert!(
        timing_text.contains("\"implement\":"),
        "expected implement phase bucket to remain present in run_timing.json: {timing_text:?}"
    );
}

#[test]
#[cfg(all(unix, target_os = "linux"))]
fn kpop_max_loops_alias_is_accepted() {
    let run = run_malvin_under_script_with_mock(
        &acp_mock_code_streaming_update_js(),
        "kpop --no-learn --p-creative 0 --max-loops 1 investigate",
        None,
    );
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(
        !stderr.contains("unexpected argument '--max-loops'"),
        "legacy --max-loops should be accepted as alias for --max-hypotheses: {stderr:?}"
    );
}

#[test]
#[cfg(all(unix, target_os = "linux"))]
fn do_pty_preserves_bold_markers_without_global_no_markdown() {
    let out = run_do_under_script(&[]);
    assert!(
        out.status.success(),
        "expected successful do run under PTY: {out:?}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("**boldline**"),
        "expected do stdout to preserve markdown markers (markdown off for do): {stdout:?}"
    );
    assert!(
        !stdout.contains("\"jsonrpc\""),
        "stdout leaked JSON-RPC protocol lines: {stdout:?}"
    );
}

#[test]
#[cfg(all(unix, target_os = "linux"))]
fn do_pty_preserves_bold_markers_with_global_no_markdown() {
    let out = run_do_under_script(&["--no-markdown"]);
    assert!(
        out.status.success(),
        "expected successful do run under PTY: {out:?}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("**boldline**"),
        "expected global --no-markdown to leave do stdout plain: {stdout:?}"
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
