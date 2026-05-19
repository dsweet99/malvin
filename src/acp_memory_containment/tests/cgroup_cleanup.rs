#[cfg(all(target_os = "linux", malvin_have_writable_cgroups))]
mod linux {
    use crate::acp_memory_containment::acp_memory_containment_unit_tests::cgroup_helpers::spawn_sleep_in_prepared_cgroup;

    fn process_exists(pid: u32) -> bool {
        std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    #[tokio::test]
    async fn remove_cgroup_dir_kills_process_still_in_leaf() {
        let Some((mut child, pid, cgroup_dir, plan)) =
            spawn_sleep_in_prepared_cgroup(&format!("kill-leaf-{}", std::process::id())).await
        else {
            crate::acp_memory_containment::test_support::require_cgroup_integration_test();
            panic!("spawn_sleep_in_prepared_cgroup failed on host with writable cgroups");
        };
        assert!(process_exists(pid));
        crate::acp_memory_containment::remove_cgroup_dir(&cgroup_dir);
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert!(
            !process_exists(pid),
            "cgroup cleanup must kill processes still in the leaf cgroup"
        );
        let _ = child.kill().await;
        let _ = child.wait().await;
        let _ = plan;
    }
}
