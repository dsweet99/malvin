//! Minimal live stdio session (`cat`) for tests that need `coder_session: Some(...)` without `agent acp`.

use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use super::session_channels::{SessionChannelState, SessionReaderTelemetry};
use super::session_types::{AcpSession, AcpSpawnArgs};
use tokio::process::{Child, Command};

fn spawn_cat_child_with_stdin(cwd: &Path) -> (Child, tokio::process::ChildStdin) {
    let mut cmd = Command::new("cat");
    cmd.kill_on_drop(true);
    cmd.current_dir(cwd);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());
    let mut child = cmd.spawn().expect("spawn cat");
    let stdin = child.stdin.take().expect("stdin");
    let _stdout = child.stdout.take().expect("stdout");
    (child, stdin)
}

fn default_test_spawn_args(cwd: &Path) -> AcpSpawnArgs<'_> {
    AcpSpawnArgs {
        cwd,
        bin_override: None,
        api_key: Some("test-key"),
        auth_token: Some("test-token"),
        rpc_timeout: Duration::from_secs(30),
        acp_verbose: false,
        george_acp_lane: None,
        ui_idle_notify: None,
        model: None,
        force: false,
        tee_trace_stdout: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        prompts_log_run_dir: None,
        log_full_outgoing_prompts: false,
    }
}

fn telemetry_for_test_args(args: &AcpSpawnArgs<'_>) -> SessionReaderTelemetry {
    SessionReaderTelemetry {
        acp_verbose: args.acp_verbose,
        raw_output: args.raw_output,
        show_thoughts_on_stdout: args.show_thoughts_on_stdout,
        emit_stdout_markdown: args.emit_stdout_markdown,
        prompts_log_run_dir: args.prompts_log_run_dir.map(std::path::Path::to_path_buf),
        log_full_outgoing_prompts: args.log_full_outgoing_prompts,
    }
}

/// Spawns `cat` with piped stdio and wraps it as an [`AcpSession`] for guard tests only.
pub(super) fn captive_cat_acp_session_for_tests(cwd: &Path) -> AcpSession {
    let (child, stdin) = spawn_cat_child_with_stdin(cwd);
    let args = default_test_spawn_args(cwd);
    let ch = SessionChannelState::new(stdin, &args);
    let telemetry = telemetry_for_test_args(&args);
    AcpSession(Arc::new(ch.into_session_inner(
        child,
        "test-session-id".into(),
        args.rpc_timeout,
        telemetry,
    )))
}
