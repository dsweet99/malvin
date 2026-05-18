#[cfg(target_os = "linux")]
mod linux {
    use std::process::Stdio;
    use std::time::Duration;

    use tokio::process::Command;

    fn spawn_over_allocator_in_plan(
        plan: &crate::acp_memory_containment::CgroupSpawnPlan,
    ) -> tokio::process::Child {
        let mut cmd = Command::new("bash");
        cmd.arg("-c")
            .arg("exec tail -c 64M /dev/zero")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        crate::acp_memory_containment::apply_linux_child_pre_exec(
            &mut cmd,
            Some(plan.cgroup_dir.clone()),
        );
        cmd.spawn().expect("spawn bash")
    }

    #[tokio::test]
    async fn tiny_cgroup_memory_limit_kills_allocating_child() {
        let Some(mut plan) =
            crate::acp_memory_containment::try_prepare_cgroup_spawn_plan(&format!(
                "enforce-{}",
                std::process::id()
            ))
        else {
            eprintln!(
                "SKIP tiny_cgroup_memory_limit_kills_allocating_child: cgroup spawn plan unavailable"
            );
            return;
        };
        let limit = plan.memory_max_bytes.min(8 * 1024 * 1024);
        assert!(
            crate::acp_memory_containment::write_memory_limit(&plan.cgroup_dir, limit),
            "fixture requires writable memory limit"
        );
        plan.memory_max_bytes = limit;
        let mut child = spawn_over_allocator_in_plan(&plan);
        let pid = child.id().expect("pid");
        assert!(
            crate::acp_memory_containment::wait_for_cgroup_join(pid, &plan).await,
            "child must join cgroup"
        );
        let status = tokio::time::timeout(Duration::from_secs(10), child.wait())
            .await
            .expect("wait timeout")
            .expect("wait status");
        assert!(
            !status.success(),
            "over-allocating child must be killed by cgroup memory limit"
        );
        crate::acp_memory_containment::remove_cgroup_dir(&plan.cgroup_dir);
    }
}
