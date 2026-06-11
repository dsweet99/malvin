//! Spawn helpers for opt-in live cursor-agent integration tests.
//!
//! Unlike [`super::command_output_with_timeout`], these do **not** set
//! `MALVIN_TEST_NO_REAL_AGENT=1` (which disables deferred-log enrichment).

#[cfg(unix)]
use std::process::Command;
#[cfg(unix)]
use std::time::{Duration, Instant};

#[cfg(unix)]
use super::child_wait::{spawn_piped_process_group, wait_child_with_timeout};

#[cfg(unix)]
pub const LIVE_AGENT_CMD_TIMEOUT: Duration = Duration::from_secs(180);

#[cfg(unix)]
pub fn live_agent_prereqs_met() -> bool {
    malvin::agent_or_cursor_agent_bin().is_some() && live_agent_auth_available()
}

#[cfg(unix)]
fn live_agent_auth_available() -> bool {
    for key in ["CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY"] {
        if std::env::var_os(key).is_some_and(|v| !v.is_empty()) {
            return true;
        }
    }
    ["agent", "cursor-agent"].into_iter().any(|bin| {
        std::process::Command::new(bin)
            .args(["auth", "status"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    })
}

/// Run malvin with real `OpenRouter` mini backend (no mock, no test-agent env).
#[cfg(unix)]
pub fn command_output_mini_live(cmd: &mut Command) -> std::io::Result<std::process::Output> {
    cmd.env_remove("MALVIN_TEST_NO_REAL_AGENT");
    cmd.env_remove("MALVIN_AGENT_ACP_BIN");
    let (child, stdout_jh, stderr_jh) = spawn_piped_process_group(cmd)?;
    wait_child_with_timeout(
        child,
        stdout_jh,
        stderr_jh,
        Instant::now() + LIVE_AGENT_CMD_TIMEOUT,
    )
}

/// Run malvin against the real cursor-agent (no mock, no test-agent env).
#[cfg(unix)]
pub fn command_output_live_agent(cmd: &mut Command) -> std::io::Result<std::process::Output> {
    cmd.env_remove("MALVIN_TEST_NO_REAL_AGENT");
    cmd.env_remove("MALVIN_AGENT_ACP_BIN");
    let (child, stdout_jh, stderr_jh) = spawn_piped_process_group(cmd)?;
    wait_child_with_timeout(
        child,
        stdout_jh,
        stderr_jh,
        Instant::now() + LIVE_AGENT_CMD_TIMEOUT,
    )
}
