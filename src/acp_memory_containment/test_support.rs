use super::{
    AcpMemoryContainment, begin_containment_for_command, complete_containment_after_spawn,
    spawn_should_warn_containment_unavailable,
};

#[cfg(target_os = "linux")]
#[must_use]
pub fn writable_cgroups_on_host() -> bool {
    crate::acp_memory_containment::resolve_writable_cgroup_parent().is_some()
}

#[cfg(target_os = "linux")]
pub fn require_cgroup_integration_test() {
    if writable_cgroups_on_host() {
        return;
    }
    crate::output::print_log_warning(
        "SKIP: cgroup integration test requires writable cgroups on this host",
    );
    panic!("cgroup integration test requires writable cgroups on this host");
}

#[must_use]
pub fn active_with_cgroup_dir(cgroup_dir: std::path::PathBuf) -> AcpMemoryContainment {
    AcpMemoryContainment::from_parts(true, Some(cgroup_dir))
}

#[must_use]
pub async fn active_via_true_child_spawn()
-> Option<(AcpMemoryContainment, tokio::process::Child)> {
    let mut cmd = tokio::process::Command::new("true");
    let handle = begin_containment_for_command(&mut cmd);
    let mut child = cmd.spawn().ok()?;
    let pid = child.id()?;
    let containment = complete_containment_after_spawn(Some(pid), handle).await;
    if !containment.active() {
        let _ = child.kill().await;
        let _ = child.wait().await;
        return None;
    }
    Some((containment, child))
}

#[cfg(test)]
mod tests {
    use super::spawn_should_warn_containment_unavailable;

    #[test]
    fn spawn_warn_only_when_containment_inactive() {
        assert!(spawn_should_warn_containment_unavailable(false));
        assert!(!spawn_should_warn_containment_unavailable(true));
    }
}
