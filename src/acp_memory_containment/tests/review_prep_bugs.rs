//! Regression tests for bugs flagged in `_malvin/20260517_231715_e38xr15g/review_prep.md`.
//! These assert correct behavior; they fail until the underlying bug is fixed.

#[cfg(target_os = "linux")]
mod linux {
    use crate::acp_memory_containment::acp_memory_containment_unit_tests::cgroup_helpers::{child_still_running, spawn_sleep_in_prepared_cgroup};
    use crate::acp_memory_containment::{
        ContainmentHandle, begin_containment_for_command, complete_containment_after_spawn,
        finalize_containment_cgroup,
    };

    #[test]
    fn shutdown_remove_must_not_leave_orphan_cgroup_while_containment_state_is_cloned() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cgroup_dir = dir.path().join("malvin-acp-shutdown-leak");
        std::fs::create_dir(&cgroup_dir).expect("cgroup dir");
        std::fs::write(cgroup_dir.join("cgroup.procs"), "").expect("procs");

        let containment =
            crate::acp_memory_containment::test_support::active_with_cgroup_dir(cgroup_dir.clone());
        let reader_clone = containment.clone();

        finalize_containment_cgroup(&containment);

        assert!(
            !cgroup_dir.exists(),
            "shutdown must remove cgroup even when stdout reader still holds a containment clone"
        );
        drop(reader_clone);
    }

    #[test]
    fn stdio_failure_remove_must_finalize_cgroup_even_when_containment_is_cloned() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cgroup_dir = dir.path().join("malvin-acp-stdio-leak");
        std::fs::create_dir(&cgroup_dir).expect("cgroup dir");
        std::fs::write(cgroup_dir.join("cgroup.procs"), "").expect("procs");

        let containment =
            crate::acp_memory_containment::test_support::active_with_cgroup_dir(cgroup_dir.clone());
        let _extra_handle = containment.clone();

        finalize_containment_cgroup(&containment);

        assert!(
            !cgroup_dir.exists(),
            "stdio failure path must finalize cgroup even when another handle still clones containment state"
        );
    }

    #[test]
    fn remove_on_clone_must_not_leave_sibling_active_without_cgroup_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.events"), "oom_kill 0\n").expect("events");
        let retained = crate::acp_memory_containment::test_support::active_with_cgroup_dir(
            dir.path().to_path_buf(),
        );
        let prompt_local = retained.clone();
        finalize_containment_cgroup(&prompt_local);
        assert!(
            !retained.active(),
            "remove on a clone must clear active on all handles sharing containment state"
        );
        assert!(
            retained.cgroup_leaf_snapshot_for_tests().is_none(),
            "shared cgroup_dir must be None after remove on a clone"
        );
    }

    #[test]
    fn remove_with_clone_must_teardown_cgroup_not_only_finalize() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cgroup_dir = dir.path().join("malvin-acp-remove-clone-leak");
        std::fs::create_dir(&cgroup_dir).expect("cgroup dir");
        std::fs::write(cgroup_dir.join("cgroup.procs"), "").expect("procs");

        let containment =
            crate::acp_memory_containment::test_support::active_with_cgroup_dir(cgroup_dir.clone());
        let _reader_clone = containment.clone();

        finalize_containment_cgroup(&containment);

        assert!(
            !cgroup_dir.exists(),
            "finalize_containment_cgroup must remove leaf cgroup even when another handle clones containment state"
        );
    }

    #[tokio::test]
    async fn complete_containment_verify_failure_leaves_child_alive_until_session_abort() {
        let Some((mut child, pid, cgroup_dir, plan)) =
            spawn_sleep_in_prepared_cgroup(&format!("review-prep-verify-{}", std::process::id()))
                .await
        else {
            assert!(
                !crate::acp_memory_containment::test_support::writable_cgroups_on_host(),
                "expected cgroup-backed sleep child on host with writable cgroups",
            );
            return;
        };
        let bad_handle = ContainmentHandle::Linux {
            cgroup_dir: cgroup_dir.clone(),
            memory_max_bytes: plan.memory_max_bytes / 2,
        };
        let containment = complete_containment_after_spawn(Some(pid), bad_handle).await;
        assert!(!containment.active());
        assert!(
            child_still_running(&mut child),
            "complete_containment_after_spawn must not kill the child on verify failure"
        );
        let _ = child.kill().await;
        let _ = child.wait().await;
        finalize_containment_cgroup(&containment);
    }

    #[tokio::test]
    async fn session_spawn_gate_aborts_when_cgroup_verify_fails() {
        let Some((mut child, pid, cgroup_dir, plan)) =
            spawn_sleep_in_prepared_cgroup(&format!("review-prep-gate-{}", std::process::id())).await
        else {
            assert!(
                !crate::acp_memory_containment::test_support::writable_cgroups_on_host(),
                "expected cgroup-backed sleep child on host with writable cgroups",
            );
            return;
        };
        let err = crate::acp_memory_containment::complete_and_require_linux_containment_after_spawn(
            Some(pid),
            ContainmentHandle::Linux {
                cgroup_dir,
                memory_max_bytes: plan.memory_max_bytes / 2,
            },
            true,
            &mut child,
        )
        .await
        .expect_err("session_spawn gate must abort when cgroup verify failed after a Linux plan");
        assert!(
            err.contains("verify failed"),
            "unexpected abort message: {err}"
        );
        assert!(
            !child_still_running(&mut child),
            "session_spawn gate must terminate the child without manual kill"
        );
    }

    #[tokio::test]
    async fn containment_must_be_active_before_remove_on_writable_cgroup_host() {
        let mut cmd = tokio::process::Command::new("true");
        let handle = begin_containment_for_command(&mut cmd);
        let mut child = cmd.spawn().expect("spawn true");
        let c = complete_containment_after_spawn(child.id(), handle).await;
        if !crate::acp_memory_containment::test_support::writable_cgroups_on_host() {
            eprintln!(
                "SKIP containment_must_be_active_before_remove_on_writable_cgroup_host: no writable cgroups"
            );
            let _ = child.wait().await;
            return;
        }
        assert!(
            c.active(),
            "containment must be active after successful join before teardown"
        );
        finalize_containment_cgroup(&c);
        let _ = child.wait().await;
    }
}
