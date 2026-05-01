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

#[test]
fn command_output_with_timeout_surfaces_captured_streams_on_timeout() {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg("printf 'stdout-seen\\n'; printf 'stderr-seen\\n' >&2; sleep 2");
    let err = command_output_with_timeout(&mut cmd, Duration::from_millis(250))
        .expect_err("timed out");
    assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
    let msg = err.to_string();
    assert!(
        msg.contains("stdout-seen") && msg.contains("stderr-seen"),
        "expected timed out output in error context: {msg}"
    );
}
