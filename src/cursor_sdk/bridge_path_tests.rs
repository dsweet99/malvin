use super::bridge_path::resolve_bridge_js;

#[test]
fn resolve_bridge_js_finds_repo_dist() {
    let path = resolve_bridge_js().expect("bridge.js in repo");
    assert!(path.ends_with("cursor-sdk-bridge/dist/bridge.js"));
    assert!(path.is_file());
}
