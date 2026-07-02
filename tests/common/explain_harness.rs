use std::path::Path;
use std::process::Command;

use super::{
    INTEGRATION_TEST_MALVIN_ARGS, MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout,
};

pub struct ExplainSpawn<'a> {
    pub workspace: &'a Path,
    pub home: &'a Path,
    pub mock: &'a Path,
    pub path_var: &'a str,
    pub request: &'a str,
    pub extra_args: &'a [&'a str],
}

pub fn seed_stale_default_explain_outputs(workspace: &Path) {
    std::fs::write(workspace.join("explain.tex"), "STALE\n").expect("write stale tex");
    std::fs::write(workspace.join("explain.pdf"), b"%PDF-1.4 stale").expect("write stale pdf");
}

pub fn assert_default_explain_sibling_outputs(workspace: &Path) {
    let stale = std::fs::read_to_string(workspace.join("explain.tex")).expect("read stale tex");
    assert_eq!(stale, "STALE\n", "original explain.tex must be untouched");
    let tex = std::fs::read_to_string(workspace.join("explain_1.tex")).expect("read allocated tex");
    assert!(
        tex.contains("Revised"),
        "explain must chain malvin revise on allocated path: {tex:?}"
    );
}

pub fn spawn_explain(t: &ExplainSpawn<'_>) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(t.workspace)
        .env("HOME", t.home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", t.mock)
        .env("PATH", t.path_var);
    let mut args: Vec<&str> = vec!["explain", t.request];
    args.extend_from_slice(INTEGRATION_TEST_MALVIN_ARGS);
    args.extend_from_slice(t.extra_args);
    cmd.args(args);
    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin")
}
