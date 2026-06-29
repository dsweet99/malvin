use super::SandboxSpawnPolicyAspect;

#[test]
fn sandbox_spawn_policy_aspect_all_has_five_variants() {
    assert_eq!(SandboxSpawnPolicyAspect::all().len(), 5);
}

#[test]
fn sandbox_spawn_policy_aspect_malvin_std_command_flags() {
    assert!(SandboxSpawnPolicyAspect::ProcessGroupIsolation.applied_by_malvin_std_command());
    assert!(SandboxSpawnPolicyAspect::MallocArenaCap.applied_by_malvin_std_command());
    assert!(!SandboxSpawnPolicyAspect::DeadBeforeNextSpawn.applied_by_malvin_std_command());
    assert!(!SandboxSpawnPolicyAspect::SessionRssMonitor.applied_by_malvin_std_command());
    assert!(!SandboxSpawnPolicyAspect::AcpSpawnLock.applied_by_malvin_std_command());
}

#[test]
fn sandbox_spawn_policy_aspect_variants_exist() {
    let _ = (
        SandboxSpawnPolicyAspect::ProcessGroupIsolation,
        SandboxSpawnPolicyAspect::MallocArenaCap,
        SandboxSpawnPolicyAspect::DeadBeforeNextSpawn,
        SandboxSpawnPolicyAspect::SessionRssMonitor,
        SandboxSpawnPolicyAspect::AcpSpawnLock,
    );
}
