use crate::acp_memory_containment::{
    AcpMemoryContainment, ContainmentHandle, begin_containment_for_command,
    complete_containment_after_spawn, finalize_containment_cgroup,
};

#[test]
fn remove_inactive_containment_is_noop() {
    finalize_containment_cgroup(&AcpMemoryContainment::inactive());
}

#[tokio::test]
async fn complete_inactive_handle_is_inactive() {
    let c = complete_containment_after_spawn(Some(1), ContainmentHandle::Inactive).await;
    assert!(!c.active());
}

#[tokio::test]
async fn begin_containment_on_true_command_does_not_panic() {
    let mut cmd = tokio::process::Command::new("true");
    let handle = begin_containment_for_command(&mut cmd);
    let mut child = cmd.spawn().expect("spawn true");
    let c = complete_containment_after_spawn(child.id(), handle).await;
    finalize_containment_cgroup(&c);
    let _ = child.wait().await;
}

#[cfg(all(target_os = "linux", malvin_have_writable_cgroups))]
mod active_containment_linux {
    use super::*;

    #[tokio::test]
    async fn true_child_gets_active_containment_when_cgroups_available() {
        crate::acp_memory_containment::test_support::require_cgroup_integration_test();
        let mut cmd = tokio::process::Command::new("true");
        let handle = begin_containment_for_command(&mut cmd);
        let mut child = cmd.spawn().expect("spawn true");
        let c = complete_containment_after_spawn(child.id(), handle).await;
        assert!(
            c.active(),
            "expected active containment when cgroups are writable on this host"
        );
        finalize_containment_cgroup(&c);
        assert!(!c.active());
        let _ = child.wait().await;
    }

    #[tokio::test]
    async fn spawn_path_activates_containment_when_cgroups_available() {
        crate::acp_memory_containment::test_support::require_cgroup_integration_test();
        let (containment, mut child) =
            crate::acp_memory_containment::test_support::active_via_true_child_spawn()
                .await
                .expect("expected cgroup-backed true child when cgroups are writable");
        assert!(containment.active());
        finalize_containment_cgroup(&containment);
        let _ = child.wait().await;
    }
}
