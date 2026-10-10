use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn package_json(bridge: &str) -> Value {
    let path = manifest_dir().join(bridge).join("package.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

#[test]
fn cursor_bridge_depends_on_cursor_sdk() {
    let pkg = package_json("cursor-sdk-bridge");
    let deps = pkg["dependencies"]
        .as_object()
        .expect("dependencies object");
    assert!(
        deps.contains_key("@cursor/sdk"),
        "cursor-sdk-bridge must depend on @cursor/sdk: {deps:?}"
    );
}

fn build_script_sources() -> Vec<(PathBuf, String)> {
    let mut paths = vec![manifest_dir().join("build.rs")];
    for entry in fs::read_dir(manifest_dir().join("src/sdk_bridge_build")).expect("read_dir") {
        paths.push(entry.expect("entry").path());
    }
    paths
        .into_iter()
        .map(|p| {
            let text =
                fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
            (p, text)
        })
        .collect()
}

#[test]
fn build_script_needs_no_node_or_npm() {
    for (path, text) in build_script_sources() {
        for needle in ["npm", "node", "sdk-bridges", ".malvinconf"] {
            assert!(
                !text.contains(needle),
                "{} mentions {needle:?}; the Cursor SDK is installed at run time, not by build.rs",
                path.display()
            );
        }
    }
}

#[test]
fn cursor_sdk_is_installed_at_run_time() {
    let path = manifest_dir().join("src/backends/cursor_sdk/bridge_install.rs");
    let text = fs::read_to_string(&path).expect("bridge_install.rs");
    let shared = fs::read_to_string(manifest_dir().join("src/npm_bridge_install.rs"))
        .expect("npm_bridge_install.rs");
    assert!(shared.contains("sdk-bridges") && text.contains("@cursor/sdk"));
    assert!(text.contains("include_bytes!") && text.contains("NpmBridgePackage"));
}
