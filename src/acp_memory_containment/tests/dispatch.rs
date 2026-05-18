use crate::acp_memory_containment::{
    AcpMemoryContainment, ContainmentHandle, begin_containment_for_command,
    complete_containment_after_spawn, finalize_containment_cgroup, write_containment_unavailable_warn,
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

#[test]
fn write_containment_unavailable_warn_message() {
    let mut buf = Vec::new();
    write_containment_unavailable_warn(&mut buf).expect("write");
    let line = String::from_utf8(buf).expect("utf8");
    assert!(line.contains("ACP memory containment unavailable"));
}

#[cfg(target_os = "linux")]
mod active_containment_linux {
    use super::*;

    #[tokio::test]
    async fn true_child_gets_active_containment_when_cgroups_available() {
        let mut cmd = tokio::process::Command::new("true");
        let handle = begin_containment_for_command(&mut cmd);
        let mut child = cmd.spawn().expect("spawn true");
        let c = complete_containment_after_spawn(child.id(), handle).await;
        if !crate::acp_memory_containment::test_support::writable_cgroups_on_host() {
            eprintln!(
                "SKIP true_child_gets_active_containment_when_cgroups_available: no writable cgroups"
            );
            finalize_containment_cgroup(&c);
            let _ = child.wait().await;
            return;
        }
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
        let Some((containment, mut child)) =
            crate::acp_memory_containment::test_support::active_via_true_child_spawn().await
        else {
            eprintln!(
                "SKIP spawn_path_activates_containment_when_cgroups_available: cgroup spawn inactive or unavailable"
            );
            return;
        };
        assert!(containment.active());
        finalize_containment_cgroup(&containment);
        let _ = child.wait().await;
    }
}
