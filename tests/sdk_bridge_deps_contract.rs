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

#[test]
fn build_rs_requires_sdk_bridge_install() {
    let path = manifest_dir().join("build.rs");
    let text = fs::read_to_string(&path).expect("build.rs");
    assert!(
        text.contains("sdk_bridge_build") && text.contains("run_build_script"),
        "build.rs must delegate SDK bridge install to sdk_bridge_build"
    );
    let logic = fs::read_to_string(manifest_dir().join("src/sdk_bridge_build/mod.rs"))
        .expect("sdk_bridge_build");
    assert!(
        logic.contains("@cursor/sdk") && logic.contains("sdk-bridges"),
        "sdk_bridge_build must install Cursor SDK npm deps under sdk-bridges"
    );
    assert!(
        logic.contains("MALVIN_SKIP_SDK_BRIDGES"),
        "sdk_bridge_build must document the skip escape hatch"
    );
}
