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
