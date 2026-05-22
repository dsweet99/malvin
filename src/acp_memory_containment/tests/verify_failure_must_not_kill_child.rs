#[cfg(all(target_os = "linux", malvin_have_writable_cgroups))]
mod linux {
    use crate::acp_memory_containment::acp_memory_containment_unit_tests::cgroup_helpers::{
        child_still_running, spawn_sleep_in_prepared_cgroup,
    };
    use crate::acp_memory_containment::pid_listed_in_leaf_cgroup;

    #[tokio::test]
    async fn join_wait_failure_must_evict_child_from_leaf_cgroup() {
        let Some((mut child, pid, cgroup_dir, plan)) =
            spawn_sleep_in_prepared_cgroup(&format!("regression-join-{}", std::process::id()))
                .await
        else {
            crate::acp_memory_containment::test_support::require_cgroup_integration_test();
            panic!("spawn_sleep_in_prepared_cgroup failed on host with writable cgroups");
        };
        let bad_plan = crate::acp_memory_containment::CgroupSpawnPlan {
            cgroup_dir: cgroup_dir.clone(),
            memory_max_bytes: plan.memory_max_bytes / 2,
        };
        assert!(!crate::acp_memory_containment::wait_for_cgroup_join(pid, &bad_plan).await);
        assert!(
            !pid_listed_in_leaf_cgroup(pid, &cgroup_dir),
            "join wait failure must not leave agent pid in leaf cgroup"
        );
        let _ = child.kill().await;
        let _ = child.wait().await;
        crate::acp_memory_containment::discard_prepared_cgroup_after_failed_join(pid, &cgroup_dir);
    }

    #[test]
    fn discard_must_not_leave_leaf_dir_when_pid_remains_after_failed_release() {
        let root = tempfile::tempdir().expect("tempdir");
        let leaf = root.path().join("malvin-acp-regression-leaf");
        std::fs::create_dir(&leaf).expect("leaf");
        let pid = std::process::id();
        std::fs::write(leaf.join("cgroup.procs"), format!("{pid}\n")).expect("procs");
        assert!(
            pid_listed_in_leaf_cgroup(pid, &leaf),
            "fixture requires pid listed in leaf cgroup.procs"
        );
        assert!(
            !crate::acp_memory_containment::release_pid_from_cgroup(pid, &leaf),
            "fixture requires release_pid_from_cgroup to fail without parent cgroup.procs"
        );
        crate::acp_memory_containment::discard_prepared_cgroup_after_failed_join(pid, &leaf);
        assert!(
            !leaf.exists(),
            "must not leave orphan malvin-acp leaf when cleanup cannot release pid"
        );
    }

    #[tokio::test]
    async fn discard_after_failed_release_must_not_kill_child_in_leaf() {
        let Some((mut child, pid, cgroup_dir, _plan)) =
            spawn_sleep_in_prepared_cgroup(&format!("regression-{}", std::process::id())).await
        else {
            crate::acp_memory_containment::test_support::require_cgroup_integration_test();
            panic!("spawn_sleep_in_prepared_cgroup failed on host with writable cgroups");
        };
        let bogus_parent = tempfile::tempdir().expect("tempdir");
        assert!(
            !crate::acp_memory_containment::release_pid_from_cgroup(pid, bogus_parent.path(),),
            "fixture requires release_pid_from_cgroup to fail like verify-failure cleanup"
        );
        crate::acp_memory_containment::discard_prepared_cgroup_after_failed_join(pid, &cgroup_dir);
        assert!(
            child_still_running(&mut child),
            "failed join cleanup must not cgroup.kill a child still in the leaf"
        );
        let _ = child.kill().await;
        let _ = child.wait().await;
        crate::acp_memory_containment::discard_prepared_cgroup_after_failed_join(pid, &cgroup_dir);
    }

    #[tokio::test]
    async fn complete_containment_verify_failure_path_must_not_kill_spawned_child() {
        use crate::acp_memory_containment::{ContainmentHandle, complete_containment_after_spawn};

        let Some((mut child, pid, cgroup_dir, plan)) =
            spawn_sleep_in_prepared_cgroup(&format!("regression-complete-{}", std::process::id()))
                .await
        else {
            crate::acp_memory_containment::test_support::require_cgroup_integration_test();
            panic!("spawn_sleep_in_prepared_cgroup failed on host with writable cgroups");
        };
        let handle = ContainmentHandle::Linux {
            cgroup_dir: cgroup_dir.clone(),
            memory_max_bytes: plan.memory_max_bytes / 2,
        };
        let containment = complete_containment_after_spawn(Some(pid), handle).await;
        assert!(!containment.active());
        assert!(
            child_still_running(&mut child),
            "verify-failure cleanup must not kill the spawned agent child"
        );
        let _ = child.kill().await;
        let _ = child.wait().await;
        crate::acp_memory_containment::discard_prepared_cgroup_after_failed_join(pid, &cgroup_dir);
    }
}
