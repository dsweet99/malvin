use std::path::Path;
use std::process::Command;

use super::{
    INTEGRATION_TEST_MALVIN_ARGS, MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout,
};
use super::integration_cli_args::FAST_GATE_LOOP_TEST_ARGS;

pub struct DelightSpawn<'a> {
    pub workspace: &'a Path,
    pub home: &'a Path,
    pub mock: &'a Path,
    pub path_var: &'a str,
    pub extra_args: &'a [&'a str],
}

pub fn spawn_delight(t: &DelightSpawn<'_>) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(t.workspace)
        .env("HOME", t.home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", t.mock)
        .env("PATH", t.path_var);
    let mut args: Vec<&str> = vec!["delight"];
    args.extend_from_slice(INTEGRATION_TEST_MALVIN_ARGS);
    args.extend_from_slice(FAST_GATE_LOOP_TEST_ARGS);
    args.extend_from_slice(t.extra_args);
    cmd.args(args);
    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin")
}
