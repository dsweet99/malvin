use crate::acp_memory_containment::finalize_containment_cgroup;

#[test]
fn remove_containment_cgroup_clears_active_flag() {
    let dir = tempfile::tempdir().expect("tempdir");
    let c = crate::acp_memory_containment::test_support::active_with_cgroup_dir(dir.path().to_path_buf());
    assert!(c.active());
    finalize_containment_cgroup(&c);
    assert!(!c.active());
}

#[test]
fn shared_containment_deactivates_when_cgroup_removed_on_sibling_handle() {
    let dir = tempfile::tempdir().expect("tempdir");
    let retained = crate::acp_memory_containment::test_support::active_with_cgroup_dir(dir.path().to_path_buf());
    assert!(retained.active());
    let prompt_local = retained.clone();
    finalize_containment_cgroup(&prompt_local);
    assert!(
        !retained.active(),
        "tearing down the cgroup must clear active on all clones sharing the Arc state"
    );
}

#[test]
fn session_spawn_must_emit_containment_unavailable_warn_when_inactive() {
    use crate::test_stderr_capture::capture_stderr_output;

    assert!(
        crate::acp_memory_containment::spawn_should_warn_containment_unavailable(false),
        "inactive containment must trigger warn policy"
    );
    assert!(
        !crate::acp_memory_containment::spawn_should_warn_containment_unavailable(true),
        "active containment must not trigger warn policy"
    );
    let stderr = capture_stderr_output(crate::acp_memory_containment::emit_containment_unavailable_warn);
    assert!(stderr.contains(crate::acp_memory_containment::CONTAINMENT_UNAVAILABLE_WARN));
    assert!(stderr.contains("malvin"));
}

#[cfg(target_os = "linux")]
mod linux_regression {
    use super::{AGENT_EXCEEDED_MEMORY_LIMIT_MSG, finalize_containment_cgroup, map_acp_child_exit_message};

    #[test]
    fn v1_failcnt_without_oom_kill_is_not_memory_limit_exceeded() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.limit_in_bytes"), "1048576").expect("limit");
        std::fs::write(dir.path().join("memory.failcnt"), "1").expect("failcnt");
        assert!(
            !dir.path().join("memory.events").exists(),
            "fixture must be v1-only (no memory.events)"
        );
        assert!(
            !crate::acp_memory_containment::memory_limit_exceeded_at(dir.path()),
            "failcnt alone must not map to agent exceeded memory limit"
        );
    }

    #[test]
    fn memory_limit_exceeded_detects_v1_under_oom_without_memory_events() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.limit_in_bytes"), "1048576").expect("limit");
        std::fs::write(
            dir.path().join("memory.oom_control"),
            "oom_kill_disable 0\nunder_oom 1\n",
        )
        .expect("oom_control");
        assert!(
            !dir.path().join("memory.events").exists(),
            "fixture must be v1-only (no memory.events)"
        );
        assert!(crate::acp_memory_containment::memory_limit_exceeded_at(dir.path()));
    }

    #[test]
    fn remove_cgroup_dir_removes_populated_cgroup_after_failed_verify() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("cgroup.procs"), "1\n").expect("procs");
        crate::acp_memory_containment::remove_cgroup_dir(dir.path());
        assert!(!dir.path().exists());
    }

    #[test]
    fn oom_message_still_mapped_after_containment_cgroup_removed() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.events"), "oom_kill 0\n").expect("events");
        let c = crate::acp_memory_containment::test_support::active_with_cgroup_dir(dir.path().to_path_buf());
        std::fs::write(dir.path().join("memory.events"), "oom_kill 1\n").expect("events");
        assert_eq!(
            map_acp_child_exit_message(&c, "acp stdout closed"),
            AGENT_EXCEEDED_MEMORY_LIMIT_MSG
        );
        finalize_containment_cgroup(&c);
        assert_eq!(
            map_acp_child_exit_message(&c, "acp stdout closed"),
            AGENT_EXCEEDED_MEMORY_LIMIT_MSG
        );
    }

    #[test]
    fn discard_must_not_leave_leaf_cgroup_dir_when_release_fails() {
        let root = tempfile::tempdir().expect("tempdir");
        let leaf = root.path().join("malvin-acp-regression-leaf");
        std::fs::create_dir(&leaf).expect("leaf cgroup dir");
        crate::acp_memory_containment::discard_prepared_cgroup_after_failed_join(1, &leaf);
        assert!(
            !leaf.exists(),
            "discard must remove or relocate leaf cgroup when release_pid_from_cgroup fails"
        );
    }

    #[test]
    fn memory_limit_exceeded_since_baseline_detects_v2_oom_after_counter_increments() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.events"), "oom_kill 0\n").expect("events");
        let baseline = crate::acp_memory_containment::memory_limit_oom_baseline_at(dir.path());
        assert!(!crate::acp_memory_containment::memory_limit_exceeded_since_baseline(
            dir.path(),
            baseline
        ));
        std::fs::write(dir.path().join("memory.events"), "oom_kill 1\n").expect("events");
        assert!(crate::acp_memory_containment::memory_limit_exceeded_since_baseline(
            dir.path(),
            baseline
        ));
    }

    #[test]
    fn memory_limit_exceeded_at_must_not_treat_stale_v2_oom_kill_as_exceeded() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.events"), "oom_kill 99\n").expect("events");
        assert!(
            !crate::acp_memory_containment::memory_limit_exceeded_at(dir.path()),
            "memory_limit_exceeded_at must not treat pre-existing oom_kill as exceeded without activation baseline"
        );
    }

    #[test]
    fn memory_limit_exceeded_must_not_use_stale_cgroup_event_counters() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.events"), "oom_kill 99\n").expect("events");
        let containment =
            crate::acp_memory_containment::test_support::active_with_cgroup_dir(dir.path().to_path_buf());
        assert!(
            !containment.memory_limit_exceeded(),
            "stale memory.events must not alone trigger memory_limit_exceeded"
        );
    }

    #[test]
    fn map_acp_child_exit_message_must_not_use_stale_cgroup_oom_counters() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.events"), "oom_kill 99\n").expect("events");
        let containment =
            crate::acp_memory_containment::test_support::active_with_cgroup_dir(dir.path().to_path_buf());
        assert_eq!(
            map_acp_child_exit_message(&containment, "acp child process is not running"),
            "acp child process is not running"
        );
    }

    #[test]
    fn cgroup_join_wait_ms_is_at_least_two_seconds() {
        let join_wait_ms = crate::acp_memory_containment::CGROUP_JOIN_WAIT_MS;
        assert!(
            join_wait_ms >= 2_000,
            "cgroup join poll must allow slow agent startup"
        );
    }

    #[test]
    fn map_acp_child_exit_message_surfaces_oom_when_active() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.events"), "oom_kill 0\n").expect("events");
        let containment =
            crate::acp_memory_containment::test_support::active_with_cgroup_dir(dir.path().to_path_buf());
        std::fs::write(dir.path().join("memory.events"), "oom_kill 1\n").expect("events");
        assert_eq!(
            map_acp_child_exit_message(&containment, "acp stdout closed"),
            AGENT_EXCEEDED_MEMORY_LIMIT_MSG
        );
    }
}

#[cfg(target_os = "linux")]
mod complete_inactive_when_verify_fails {
    use std::path::PathBuf;

    use crate::acp_memory_containment::{
        ContainmentHandle, complete_containment_after_spawn,
    };

    #[tokio::test]
    async fn complete_containment_inactive_when_verify_fails() {
        let handle = ContainmentHandle::Linux {
            cgroup_dir: PathBuf::from("/nonexistent-malvin-acp-cgroup"),
            memory_max_bytes: 1,
        };
        let containment = complete_containment_after_spawn(Some(std::process::id()), handle).await;
        assert!(!containment.active());
    }
}
