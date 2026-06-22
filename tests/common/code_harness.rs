use std::path::Path;
use std::process::Command;

use super::{INTEGRATION_TEST_MALVIN_ARGS, MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout};

pub struct CodeSpawn<'a> {
    pub workspace: &'a Path,
    pub home: &'a Path,
    pub mock: &'a Path,
    pub path_var: &'a str,
    pub extra_args: &'a [&'a str],
    pub request: &'a str,
}

pub fn spawn_code(c: &CodeSpawn<'_>) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(c.workspace)
        .env("HOME", c.home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", c.mock)
        .env("PATH", c.path_var)
        .env("MALVIN_TEST_NO_REAL_AGENT", "1")
        .args(["code"]);
    cmd.args(INTEGRATION_TEST_MALVIN_ARGS);
    cmd.args(c.extra_args);
    cmd.arg(c.request);
    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin code")
}
