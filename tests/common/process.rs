#[cfg(unix)]
use std::process::Command;
#[cfg(unix)]
use std::time::Instant;

#[cfg(unix)]
use super::child_wait::{spawn_piped_process_group, wait_child_with_timeout};

#[cfg(unix)]
pub const MALVIN_TEST_CMD_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(12);

#[cfg(unix)]
pub fn command_output_with_timeout(
    cmd: &mut Command,
    timeout: std::time::Duration,
) -> std::io::Result<std::process::Output> {
    let (child, stdout_jh, stderr_jh) = spawn_piped_process_group(cmd)?;
    wait_child_with_timeout(child, stdout_jh, stderr_jh, Instant::now() + timeout)
}

use std::path::Path;

pub struct PlanSpawn<'a> {
    pub workspace: &'a Path,
    pub home: &'a Path,
    pub mock_agent: &'a Path,
    pub path: String,
}

pub fn spawn_malvin_plan(sp: &PlanSpawn<'_>, args: &[&str]) -> std::process::Output {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(sp.workspace)
        .env("HOME", sp.home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", sp.mock_agent)
        .env("PATH", &sp.path)
        .arg("plan");
    for a in args {
        cmd.arg(a);
    }
    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin")
}
