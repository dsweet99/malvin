#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
use super::{
    acp_mock_code_streaming_update_js, command_output_with_timeout, test_home_workspace,
    write_fake_kiss, write_mock_executable, MALVIN_TEST_CMD_TIMEOUT,
};

#[cfg(unix)]
pub const MAX_LOOPS_EXHAUSTED: &str =
    "Did not receive LGTM for review_1.md within max loops.";

pub fn check_ignored(repo: &std::path::Path, rel_path: &str) -> bool {
    Command::new("git")
        .current_dir(repo)
        .args(["check-ignore", "-q", rel_path])
        .status()
        .unwrap_or_else(|e| panic!("git check-ignore spawn failed: {e}"))
        .success()
}

#[cfg(unix)]
pub struct CodeRunOpts {
    pub no_tee: bool,
    pub trust_plan: bool,
}

#[cfg(unix)]
pub struct SyncRunOpts {
    pub no_tee: bool,
    pub with_kissconfig: bool,
}

#[cfg(unix)]
fn prep_acp_mock_on_path(
    root: &tempfile::TempDir,
    mock_rel: &str,
    mock_js: &str,
) -> (std::path::PathBuf, std::path::PathBuf, String) {
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let mock = root.path().join(mock_rel);
    write_mock_executable(&mock, mock_js);
    write_fake_kiss(&bin_dir.join("kiss"));
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    (bin_dir, mock, path)
}

#[cfg(unix)]
pub fn run_code_with_mock_js_trust_plan(
    mock_js: &str,
    extra_args: &[&str],
    opts: &CodeRunOpts,
) -> std::process::Output {
    let (root, home, workspace) = test_home_workspace();
    let (_bin_dir, mock, path) = prep_acp_mock_on_path(&root, "mock-agent-acp-code", mock_js);
    let mut args = vec!["code", "--no-learn"];
    if opts.trust_plan {
        args.push("--trust-the-plan");
    }
    args.extend_from_slice(extra_args);
    args.push("ship it");
    if opts.no_tee {
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
pub fn run_code_with_mock_js(
    mock_js: &str,
    extra_args: &[&str],
    no_tee: bool,
) -> std::process::Output {
    run_code_with_mock_js_trust_plan(
        mock_js,
        extra_args,
        &CodeRunOpts {
            no_tee,
            trust_plan: true,
        },
    )
}

#[cfg(unix)]
pub fn run_code_max_loops_zero_with_mock_opts(no_tee: bool) -> std::process::Output {
    run_code_with_mock_js(
        &acp_mock_code_streaming_update_js(),
        &["--max-loops", "0"],
        no_tee,
    )
}

#[cfg(unix)]
pub fn run_code_max_loops_zero_with_mock() -> std::process::Output {
    run_code_max_loops_zero_with_mock_opts(true)
}

#[cfg(unix)]
pub fn run_code_max_loops_zero_with_mock_stdout() -> std::process::Output {
    run_code_max_loops_zero_with_mock_opts(false)
}

#[cfg(unix)]
pub fn run_code_max_loops_zero_with_mock_without_trust_plan() -> std::process::Output {
    run_code_with_mock_js_trust_plan(
        &acp_mock_code_streaming_update_js(),
        &["--max-loops", "0"],
        &CodeRunOpts {
            no_tee: true,
            trust_plan: false,
        },
    )
}

#[cfg(unix)]
pub fn run_sync_with_mock_js_and_workspace(
    mock_js: &str,
    extra_args: &[&str],
    opts: &SyncRunOpts,
) -> (std::process::Output, tempfile::TempDir, PathBuf) {
    let (root, home, workspace) = test_home_workspace();
    let (_bin_dir, mock, path) = prep_acp_mock_on_path(&root, "mock-agent-acp-sync", mock_js);
    if opts.with_kissconfig {
        std::fs::write(workspace.join(".kissconfig"), "k\n").expect("write kissconfig");
    }
    let mut args = vec!["sync", "--no-learn"];
    args.extend_from_slice(extra_args);
    if opts.no_tee {
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
pub fn run_sync_with_mock_js(
    mock_js: &str,
    extra_args: &[&str],
    no_tee: bool,
) -> std::process::Output {
    run_sync_with_mock_js_and_workspace(
        mock_js,
        extra_args,
        &SyncRunOpts {
            no_tee,
            with_kissconfig: false,
        },
    )
    .0
}

#[cfg(unix)]
pub fn run_sync_with_mock_js_max_loops_zero() -> std::process::Output {
    run_sync_with_mock_js(
        &acp_mock_code_streaming_update_js(),
        &["--max-loops", "0"],
        true,
    )
}
