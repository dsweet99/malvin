fn child_health_timeout_error(
    outcome: crate::child_health::SilenceHealthOutcome,
    containment: &crate::acp_memory_containment::AcpMemoryContainment,
) -> Option<String> {
    use crate::acp_memory_containment::map_acp_child_exit_message;
    match outcome {
        crate::child_health::SilenceHealthOutcome::ChildNotRunning => Some(
            map_acp_child_exit_message(containment, "acp child process is not running"),
        ),
        crate::child_health::SilenceHealthOutcome::ChildZombie => Some(map_acp_child_exit_message(
            containment,
            "acp child process is zombie",
        )),
        crate::child_health::SilenceHealthOutcome::StillBusyExtendWait => None,
        crate::child_health::SilenceHealthOutcome::AppearsHung => Some(map_acp_child_exit_message(
            containment,
            "acp child process appears hung",
        )),
    }
}

#[test]
fn appears_hung_must_surface_oom_when_v1_under_oom_cleared_after_activation() {
    use crate::acp_memory_containment::{
        test_support, AGENT_EXCEEDED_MEMORY_LIMIT_MSG,
    };

    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("memory.limit_in_bytes"), "1048576").expect("limit");
    std::fs::write(
        dir.path().join("memory.oom_control"),
        "oom_kill_disable 0\nunder_oom 1\n",
    )
    .expect("oom_control");
    assert!(
        !dir.path().join("memory.events").exists(),
        "fixture must be v1-only"
    );
    let containment = test_support::active_with_cgroup_dir(dir.path().to_path_buf());
    std::fs::write(
        dir.path().join("memory.oom_control"),
        "oom_kill_disable 0\nunder_oom 0\n",
    )
    .expect("oom_control");
    let err = child_health_timeout_error(
        crate::child_health::SilenceHealthOutcome::AppearsHung,
        &containment,
    );
    assert_eq!(
        err.as_deref(),
        Some(AGENT_EXCEEDED_MEMORY_LIMIT_MSG),
        "AppearsHung must surface agent exceeded memory limit after cgroup OOM even when under_oom clears"
    );
}

#[test]
fn appears_hung_without_memory_exceeded_must_surface_hung_error() {
    use crate::acp_memory_containment::test_support;

    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("memory.events"), "oom_kill 0\n").expect("events");
    let containment = test_support::active_with_cgroup_dir(dir.path().to_path_buf());
    assert!(!containment.memory_limit_exceeded());
    let err = child_health_timeout_error(
        crate::child_health::SilenceHealthOutcome::AppearsHung,
        &containment,
    );
    assert_eq!(
        err.as_deref(),
        Some("acp child process appears hung"),
        "AppearsHung must map to a hung error even when cgroup OOM counters are unchanged"
    );
}

#[test]
fn appears_hung_path_maps_memory_limit_when_exceeded() {
    use crate::acp_memory_containment::{
        test_support, AGENT_EXCEEDED_MEMORY_LIMIT_MSG,
    };

    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("memory.events"), "oom_kill 0\n").expect("events");
    let containment = test_support::active_with_cgroup_dir(dir.path().to_path_buf());
    std::fs::write(dir.path().join("memory.events"), "oom_kill 1\n").expect("events");
    let err = child_health_timeout_error(
        crate::child_health::SilenceHealthOutcome::AppearsHung,
        &containment,
    );
    assert_eq!(err.as_deref(), Some(AGENT_EXCEEDED_MEMORY_LIMIT_MSG));
}

#[test]
fn child_not_running_must_not_use_stale_cgroup_oom_counters() {
    use crate::acp_memory_containment::test_support;

    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("memory.events"), "oom_kill 99\n").expect("events");
    let containment = test_support::active_with_cgroup_dir(dir.path().to_path_buf());
    let err = child_health_timeout_error(
        crate::child_health::SilenceHealthOutcome::ChildNotRunning,
        &containment,
    )
    .expect("child not running should yield an error");
    assert_eq!(err, "acp child process is not running");
}

#[test]
fn child_zombie_must_not_use_stale_cgroup_oom_counters() {
    use crate::acp_memory_containment::test_support;

    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("memory.events"), "oom_kill 99\n").expect("events");
    let containment = test_support::active_with_cgroup_dir(dir.path().to_path_buf());
    let err = child_health_timeout_error(
        crate::child_health::SilenceHealthOutcome::ChildZombie,
        &containment,
    )
    .expect("zombie should yield an error");
    assert_eq!(err, "acp child process is zombie");
}

#[test]
fn child_exit_maps_memory_limit_after_oom_increment() {
    use crate::acp_memory_containment::{
        test_support, AGENT_EXCEEDED_MEMORY_LIMIT_MSG,
    };

    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("memory.events"), "oom_kill 0\n").expect("events");
    let containment = test_support::active_with_cgroup_dir(dir.path().to_path_buf());
    std::fs::write(dir.path().join("memory.events"), "oom_kill 1\n").expect("events");
    let not_running = child_health_timeout_error(
        crate::child_health::SilenceHealthOutcome::ChildNotRunning,
        &containment,
    )
    .expect("not running");
    let zombie = child_health_timeout_error(
        crate::child_health::SilenceHealthOutcome::ChildZombie,
        &containment,
    )
    .expect("zombie");
    assert_eq!(not_running, AGENT_EXCEEDED_MEMORY_LIMIT_MSG);
    assert_eq!(zombie, not_running);
}
