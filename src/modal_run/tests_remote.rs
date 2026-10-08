use std::ffi::OsString;
use std::path::Path;

use serde_json::json;

use super::backend_setup::setup_for;
use super::backends::{PI_ENTRY, local_pi_version, parse_version_token};
use super::remote::{FINISH_SCRIPT, RemotePaths, SETUP_SCRIPT};
use super::sweep::{TAG_HOST, TAG_PID, host_name, run_tags, stale_ids};
use super::{reject_unsupported, remote_args};

#[test]
fn remote_paths_mirror_local_paths() {
    let p = RemotePaths::new(Path::new("/home/alice/proj"), Path::new("/home/alice"));
    assert_eq!(p.work_dir, "/home/alice/proj");
    assert!(p.logs_dir.starts_with("/home/alice/.malvinconf/logs/"));
    assert!(
        p.logs_dir
            .ends_with(&crate::workspace_logs_hash(Path::new("/home/alice/proj")))
    );
    assert_eq!(p.config_path(), "/home/alice/.malvinconf/config.toml");
    assert_eq!(p.script_env()["MALVIN_MODAL_WORK"], "/home/alice/proj");
    let env = p.malvin_env([("K".to_string(), "v".to_string())].into());
    assert_eq!(
        (env["HOME"].as_str(), env["K"].as_str()),
        ("/home/alice", "v")
    );
    assert!(SETUP_SCRIPT.contains("git rev-parse HEAD") && FINISH_SCRIPT.contains("--binary"));
}

#[test]
fn stale_ids_picks_dead_runs_on_this_host() {
    let listed = json!({ "sandboxes": [
        { "sandbox_id": "dead", "tags": { TAG_HOST: "h", TAG_PID: "1" } },
        { "sandbox_id": "live", "tags": { TAG_HOST: "h", TAG_PID: "2" } },
        { "sandbox_id": "other", "tags": { TAG_HOST: "x", TAG_PID: "1" } },
        { "sandbox_id": "nopid", "tags": { TAG_HOST: "h" } },
    ]});
    assert_eq!(
        stale_ids(&listed, "h", |pid| pid == 2),
        vec!["dead", "nopid"]
    );
    assert!(stale_ids(&json!({}), "h", |_| false).is_empty());
    assert_eq!(run_tags("r", "h", 7)[TAG_PID], "7");
    assert!(!host_name().is_empty());
}

#[test]
fn remote_args_drop_program_and_modal_flag() {
    let raw: Vec<OsString> = ["malvin", "--remote=modal:sandbox", "--do", "x"]
        .map(OsString::from)
        .to_vec();
    assert_eq!(
        remote_args(&raw),
        vec![OsString::from("--do"), OsString::from("x")]
    );
    assert!(
        reject_unsupported(&["--watch"])
            .unwrap_err()
            .contains("`--watch`")
    );
    assert!(
        reject_unsupported(&["-g", "--iml"])
            .unwrap_err()
            .contains("`--iml`")
    );
    assert!(reject_unsupported(&["-g"]).is_ok());
}

#[test]
fn backend_layers_install_the_cli_each_model_needs() {
    let tmp = tempfile::tempdir().unwrap();
    let pkg = tmp
        .path()
        .join(".malvinconf/sdk-bridges/node_modules/@earendil-works/pi-coding-agent");
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::write(pkg.join("package.json"), r#"{"version":"1.2.3"}"#).unwrap();
    assert_eq!(local_pi_version(tmp.path()), "1.2.3");
    assert_eq!(local_pi_version(&tmp.path().join("none")), "latest");
    let pi = setup_for("pi:openai/x")
        .unwrap()
        .image_layer(tmp.path())
        .unwrap();
    assert!(
        pi[0].ends_with("@earendil-works/pi-coding-agent@1.2.3"),
        "{pi:?}"
    );
    assert!(
        setup_for("codex:gpt")
            .unwrap()
            .image_layer(tmp.path())
            .unwrap()[0]
            .contains("@openai/codex@")
    );
    assert!(
        setup_for("cursor:auto")
            .unwrap()
            .image_layer(tmp.path())
            .is_none()
    );
    assert_eq!(
        setup_for("pi:openai/x").unwrap().sandbox_env()["MALVIN_PI"],
        PI_ENTRY
    );
    assert!(setup_for("cursor:auto").unwrap().sandbox_env().is_empty());
    assert_eq!(
        parse_version_token("codex-cli 0.155.0\n").as_deref(),
        Some("0.155.0")
    );
    assert_eq!(parse_version_token("v1.2").as_deref(), Some("1.2"));
    assert_eq!(parse_version_token("nope"), None);
    assert_eq!(parse_version_token("glibc 2.31\n").as_deref(), Some("2.31"));
}

#[test]
fn non_git_upload_over_the_cap_is_refused_and_git_trees_are_not_measured() {
    use super::workspace::check_upload_size;
    let tmp = tempfile::tempdir().unwrap();
    let d = tmp.path();
    std::fs::create_dir_all(d.join("sub")).unwrap();
    std::fs::write(d.join("sub/data"), vec![0u8; 3000]).unwrap();
    std::fs::create_dir_all(d.join("target")).unwrap();
    std::fs::write(d.join("target/big"), vec![0u8; 50_000]).unwrap();
    assert!(check_upload_size(d, 10_000).is_ok());
    let err = check_upload_size(d, 2000).unwrap_err();
    assert!(
        err.contains("not in a git work tree") && err.contains("smaller directory"),
        "{err}"
    );
    let git = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(d)
        .status()
        .unwrap();
    assert!(git.success());
    assert!(check_upload_size(d, 2000).is_ok());
}
