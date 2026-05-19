use crate::acp_memory_containment::{
    OomBaseline, half_physical_memory_bytes, memory_limit_exceeded_since_baseline,
    memory_limit_oom_baseline_at, remove_cgroup_dir_at,
};

#[cfg(not(target_os = "linux"))]
#[test]
fn kiss_smoke_mod_wrappers() {
    let _ = half_physical_memory_bytes();
    let dir = tempfile::tempdir().expect("tempdir");
    let _ = memory_limit_oom_baseline_at(dir.path());
    let baseline = OomBaseline::default();
    let _ = memory_limit_exceeded_since_baseline(dir.path(), baseline);
    remove_cgroup_dir_at(dir.path());
}

#[cfg(target_os = "linux")]
mod linux_kiss_smoke {
    use super::{
        OomBaseline, half_physical_memory_bytes, memory_limit_exceeded_since_baseline,
        memory_limit_oom_baseline_at, remove_cgroup_dir_at,
    };
    use crate::acp_memory_containment::memory_limit_exceeded_at;
    use crate::acp_memory_containment::next_cgroup_suffix;

    #[test]
    fn kiss_smoke_mod_wrappers() {
        let _ = half_physical_memory_bytes();
        let _ = next_cgroup_suffix();
        let dir = tempfile::tempdir().expect("tempdir");
        let _ = memory_limit_oom_baseline_at(dir.path());
        let baseline = OomBaseline::default();
        let _ = memory_limit_exceeded_since_baseline(dir.path(), baseline);
        let _ = memory_limit_exceeded_at(dir.path());
        remove_cgroup_dir_at(dir.path());
    }
}

#[cfg(target_os = "linux")]
mod linux_kiss {
    use crate::acp_memory_containment::resolve_writable_cgroup_parent;
    use crate::acp_memory_containment::try_prepare_cgroup_spawn_plan;
    #[test]
    fn kiss_smoke_linux_impl_enforcement_paths() {
        let _ = resolve_writable_cgroup_parent();
        let _ = try_prepare_cgroup_spawn_plan("kiss-smoke");
    }

}

#[cfg(all(target_os = "linux", malvin_have_writable_cgroups))]
mod linux_cgroup_integration {
    use crate::acp_memory_containment::discard_prepared_cgroup_after_failed_join;
    use crate::acp_memory_containment::verify_pid_in_cgroup;
    use crate::acp_memory_containment::acp_memory_containment_unit_tests::cgroup_helpers::spawn_sleep_in_prepared_cgroup;

    #[tokio::test]
    async fn verify_pid_in_cgroup_true_for_joined_sleep_child() {
        let Some((mut child, pid, _cgroup_dir, plan)) =
            spawn_sleep_in_prepared_cgroup(&format!("kiss-verify-{}", std::process::id())).await
        else {
            crate::acp_memory_containment::test_support::require_cgroup_integration_test();
            panic!("spawn_sleep_in_prepared_cgroup failed on host with writable cgroups");
        };
        assert!(
            verify_pid_in_cgroup(pid, &plan),
            "joined child must pass cgroup membership and memory.max verify"
        );
        let _ = child.kill().await;
        let _ = child.wait().await;
        discard_prepared_cgroup_after_failed_join(pid, &plan.cgroup_dir);
    }
}
