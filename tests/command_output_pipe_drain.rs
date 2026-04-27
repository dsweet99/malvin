#![cfg(unix)]

mod common;

use common::command_output_with_timeout;
use std::process::Command;
use std::time::Duration;

#[test]
fn command_output_with_timeout_drains_large_stdout_without_deadlock() {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg("head -c 400000 /dev/zero; printf done");
    let out = command_output_with_timeout(&mut cmd, Duration::from_secs(8)).expect("completed");
    assert!(out.status.success());
    assert_eq!(out.stdout.len(), 400_004);
    assert!(out.stdout.ends_with(b"done"));
}
