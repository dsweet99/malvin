//! Helpers for spawn-lock and process-group contract integration tests.

#[cfg(unix)]
use malvin::malvin_sandbox::malvin_std_command;
#[cfg(unix)]
use std::path::PathBuf;

#[cfg(unix)]
pub fn fresh_workdir(name: &str) -> PathBuf {
    let work = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).expect("mkdir work");
    work
}

#[cfg(unix)]
pub fn sleep_child(seconds: &str) -> std::process::Child {
    let mut cmd = malvin_std_command("sleep");
    cmd.arg(seconds);
    cmd.spawn().expect("spawn sleep")
}
