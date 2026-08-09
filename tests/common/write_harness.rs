use std::path::Path;
use std::process::Command;

use super::{
    INTEGRATION_TEST_MALVIN_ARGS, MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout,
};

pub struct WriteSpawn<'a> {
    pub workspace: &'a Path,
    pub home: &'a Path,
    pub mock: &'a Path,
    pub path_var: &'a str,
    pub request: &'a str,
    pub extra_args: &'a [&'a str],
}

pub fn seed_stale_default_write_outputs(workspace: &Path) {
    std::fs::write(workspace.join("write.tex"), "STALE\n").expect("write stale tex");
    std::fs::write(workspace.join("write.pdf"), b"%PDF-1.4 stale").expect("write stale pdf");
}

pub fn assert_default_write_sibling_outputs(workspace: &Path) {
    let stale = std::fs::read_to_string(workspace.join("write.tex")).expect("read stale tex");
    assert_eq!(stale, "STALE\n", "original write.tex must be untouched");
    let tex = std::fs::read_to_string(workspace.join("write_1.tex")).expect("read allocated tex");
    assert!(
        tex.contains("Explain") || tex.contains("document"),
        "allocated write.tex must contain explanation body: {tex:?}"
    );
}

pub fn spawn_write(t: &WriteSpawn<'_>) -> std::process::Output {
    spawn_write_with_timeout(t, MALVIN_TEST_CMD_TIMEOUT)
}

/// Empty-PDF / multi-loop review paths need headroom beyond the default 12s kill.
pub fn spawn_write_with_timeout(
    t: &WriteSpawn<'_>,
    timeout: std::time::Duration,
) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(t.workspace)
        .env("HOME", t.home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", t.mock)
        .env("PATH", t.path_var);
    let mut args: Vec<&str> = vec!["write", t.request];
    args.extend_from_slice(INTEGRATION_TEST_MALVIN_ARGS);
    args.extend_from_slice(t.extra_args);
    cmd.args(args);
    command_output_with_timeout(&mut cmd, timeout).expect("spawn malvin")
}
