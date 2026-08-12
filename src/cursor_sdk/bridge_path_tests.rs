use super::bridge_path::{cursor_sdk_marker_present, resolve_bridge_js};
use std::fs;
use tempfile::tempdir;

#[test]
fn resolve_bridge_js_finds_repo_dist() {
    let path = resolve_bridge_js().expect("bridge.js in repo");
    assert!(path.ends_with("cursor-sdk-bridge/dist/bridge.js"));
    assert!(path.is_file());
}

#[test]
fn cursor_sdk_marker_present_requires_package_json() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    assert!(!cursor_sdk_marker_present(root));
    let marker = root.join("cursor-sdk-bridge/node_modules/@cursor/sdk/package.json");
    fs::create_dir_all(marker.parent().unwrap()).unwrap();
    fs::write(&marker, b"{}").unwrap();
    assert!(cursor_sdk_marker_present(root));
}
