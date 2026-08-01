use super::SandboxSpawnPolicyAspect;

#[test]
fn sandbox_spawn_policy_aspect_all_has_five_variants() {
    assert_eq!(SandboxSpawnPolicyAspect::all().len(), 5);
}

#[test]
fn all_aspects_have_runtime_references() {
    let sources = [
        include_str!("../malvin_sandbox.rs"),
        include_str!("../acp_spawn_lock.rs"),
        include_str!("../process_group_rss/mod.rs"),
    ];
    for aspect in SandboxSpawnPolicyAspect::all() {
        let needle = format!("SandboxSpawnPolicyAspect::{aspect:?}");
        assert!(
            sources.iter().any(|src| src.contains(&needle)),
            "missing production reference for {aspect:?}"
        );
    }
}
