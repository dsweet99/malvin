//! Integration smoke: `malvin generate-script`.

mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use common::{combined_cli_output, test_home_workspace, MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout};

fn run_generate_script(workspace: &std::path::Path, recipe: &str) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(workspace)
        .args(["--no-tee", "generate-script", recipe]);
    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn generate-script")
}

#[cfg(unix)]
#[test]
fn generate_script_writes_json_and_stubs() {
    let (_root, _home, workspace) = test_home_workspace();
    let out = run_generate_script(
        &workspace,
        "run-10-steps: startup.sh, task.sh, exit.sh",
    );
    let combined = combined_cli_output(&out);
    assert!(out.status.success(), "generate-script failed: {combined:?}");
    assert!(combined.contains("wrote auto-script.json"), "{combined:?}");
    assert!(combined.contains("created startup.sh"), "{combined:?}");

    let json_path = workspace.join("auto-script.json");
    let text = fs::read_to_string(&json_path).expect("read json");
    let doc: serde_json::Value = serde_json::from_str(&text).expect("parse json");
    assert_eq!(doc["version"], 1);
    assert_eq!(doc["recipe"], "run-10-steps");
    assert_eq!(doc["atomic"], true);
    assert_eq!(doc["steps"].as_array().map(Vec::len), Some(10));

    for name in ["startup.sh", "task.sh", "exit.sh"] {
        let path = workspace.join(name);
        assert!(path.is_file(), "missing stub {name}");
        let mode = fs::metadata(&path).expect("meta").permissions().mode() & 0o777;
        assert_eq!(mode, 0o755, "{name} must be executable");
        let body = fs::read_to_string(&path).expect("read stub");
        assert!(body.contains("malvin-script-stub"), "{body:?}");
    }
}

#[cfg(unix)]
#[test]
fn generate_script_rerun_does_not_overwrite_stubs() {
    let (_root, _home, workspace) = test_home_workspace();
    let recipe = "run-3-steps: startup.sh, task.sh, exit.sh";
    assert!(run_generate_script(&workspace, recipe).status.success());

    let startup = workspace.join("startup.sh");
    fs::write(&startup, "#!/bin/sh\nkeep-me\n").expect("seed startup");

    let out = run_generate_script(&workspace, recipe);
    let combined = combined_cli_output(&out);
    assert!(out.status.success(), "{combined:?}");
    assert!(
        !combined.contains("created startup.sh"),
        "must not recreate existing stub: {combined:?}"
    );
    assert_eq!(
        fs::read_to_string(&startup).expect("read"),
        "#!/bin/sh\nkeep-me\n"
    );
}
