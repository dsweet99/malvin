use super::bridge_install::{
    PAYLOAD, fnv1a64, install_failed_message, install_into, npm_deps_current,
};
use super::bridge_install_npm::{path_with_node_first, resolve_npm};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn fake_npm(dir: &Path, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("fake-npm");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

const FAKE_NPM_OK: &str =
    "mkdir -p node_modules/@cursor/sdk && echo '{}' > node_modules/@cursor/sdk/package.json";

#[test]
fn fnv1a64_is_stable() {
    assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
    assert_ne!(fnv1a64(b"a"), fnv1a64(b"b"));
}

#[test]
fn payload_lists_every_non_test_dist_file() {
    let dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("cursor-sdk-bridge/dist");
    let mut on_disk: Vec<String> = fs::read_dir(&dist)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "js"))
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .filter(|n| !n.ends_with("_test.js"))
        .map(|n| format!("dist/{n}"))
        .collect();
    on_disk.sort();
    let mut embedded: Vec<String> = PAYLOAD
        .iter()
        .map(|(rel, _)| (*rel).to_string())
        .filter(|rel| rel.starts_with("dist/"))
        .collect();
    embedded.sort();
    assert_eq!(embedded, on_disk);
}

#[test]
fn install_into_runs_npm_once_then_reuses_install() {
    let _g = crate::test_utils::test_env_lock();
    let tmp = tempdir().unwrap();
    let npm = fake_npm(tmp.path(), FAKE_NPM_OK);
    let dest = tmp.path().join("bridge");
    crate::acp::with_env("MALVIN_NPM", Some(npm.to_str().unwrap()), || {
        let bridge = install_into(&dest).expect("install");
        assert_eq!(bridge, dest.join("dist/bridge.js"));
    });
    for (rel, bytes) in PAYLOAD {
        assert_eq!(fs::read(dest.join(rel)).unwrap(), *bytes, "{rel}");
    }
    assert!(npm_deps_current(&dest));
    let missing = tmp.path().join("missing-npm");
    crate::acp::with_env("MALVIN_NPM", Some(missing.to_str().unwrap()), || {
        install_into(&dest).expect("up-to-date install must not run npm");
    });
}

#[test]
fn install_into_rewrites_stale_payload_files() {
    let _g = crate::test_utils::test_env_lock();
    let tmp = tempdir().unwrap();
    let npm = fake_npm(tmp.path(), FAKE_NPM_OK);
    let dest = tmp.path().join("bridge");
    fs::create_dir_all(dest.join("dist")).unwrap();
    fs::write(dest.join("dist/bridge.js"), b"old").unwrap();
    crate::acp::with_env("MALVIN_NPM", Some(npm.to_str().unwrap()), || {
        install_into(&dest).expect("install");
    });
    assert_ne!(fs::read(dest.join("dist/bridge.js")).unwrap(), b"old");
}

#[test]
fn install_into_reports_npm_failure() {
    let _g = crate::test_utils::test_env_lock();
    let tmp = tempdir().unwrap();
    let npm = fake_npm(tmp.path(), "exit 3");
    let dest = tmp.path().join("bridge");
    let mut err = String::new();
    crate::acp::with_env("MALVIN_NPM", Some(npm.to_str().unwrap()), || {
        err = install_into(&dest).expect_err("npm failure");
    });
    assert!(err.contains("failed"), "{err}");
    assert!(!npm_deps_current(&dest));
}

#[test]
fn install_failed_message_names_node_free_backends() {
    let msg = install_failed_message("npm not found");
    assert!(msg.contains("npm not found"));
    assert!(msg.contains("MALVIN_CURSOR_SDK_BRIDGE"));
    assert!(msg.contains("22.13"));
    assert!(msg.contains("rpi:") && msg.contains("codex:"));
}

#[test]
fn resolve_npm_prefers_env_then_node_sibling() {
    let _g = crate::test_utils::test_env_lock();
    let tmp = tempdir().unwrap();
    let node = tmp.path().join("node");
    let sibling = fake_npm(tmp.path(), "exit 0");
    fs::rename(&sibling, tmp.path().join("npm")).unwrap();
    crate::acp::with_env("MALVIN_NPM", None, || {
        assert_eq!(resolve_npm(&node).unwrap(), tmp.path().join("npm"));
    });
    crate::acp::with_env("MALVIN_NPM", Some("/x/npm"), || {
        assert_eq!(resolve_npm(&node).unwrap(), PathBuf::from("/x/npm"));
    });
}

#[test]
fn path_with_node_first_prepends_node_dir() {
    let path = path_with_node_first(Path::new("/opt/node22/bin/node"));
    let first = std::env::split_paths(&path).next().unwrap();
    assert_eq!(first, PathBuf::from("/opt/node22/bin"));
}
