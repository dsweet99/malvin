use std::ffi::OsStr;

use super::parent_death_signal::install_parent_death_signal;

#[cfg(unix)]
pub fn isolate_child_process_group(cmd: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    cmd.process_group(0);
}

#[cfg(not(unix))]
pub fn isolate_child_process_group(_: &mut std::process::Command) {}

fn apply_sandbox_resource_limits(cmd: &mut std::process::Command) {
    cmd.env("MALLOC_ARENA_MAX", "2");
}

#[must_use]
pub fn malvin_std_command(program: impl AsRef<OsStr>) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    isolate_child_process_group(&mut cmd);
    install_parent_death_signal(&mut cmd);
    apply_sandbox_resource_limits(&mut cmd);
    crate::herdr::strip_herdr_env(&mut cmd);
    cmd
}

#[must_use]
pub fn malvin_tokio_command(program: impl AsRef<OsStr>) -> tokio::process::Command {
    tokio::process::Command::from(malvin_std_command(program))
}
