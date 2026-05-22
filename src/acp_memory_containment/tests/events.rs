use crate::acp_memory_containment::{AcpMemoryContainment, map_acp_child_exit_message};

#[test]
fn containment_maps_exit_message_when_inactive() {
    let c = AcpMemoryContainment::inactive();
    assert_eq!(
        map_acp_child_exit_message(&c, "acp child process is not running"),
        "acp child process is not running"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn containment_maps_exit_message_when_oom_events_present() {
    use crate::acp_memory_containment::AGENT_EXCEEDED_MEMORY_LIMIT_MSG;
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("memory.events"), "oom_kill 0\n").expect("events");
    let c = crate::acp_memory_containment::test_support::active_with_cgroup_dir(
        dir.path().to_path_buf(),
    );
    std::fs::write(dir.path().join("memory.events"), "oom_kill 1\n").expect("events");
    assert!(c.memory_limit_exceeded());
    assert_eq!(
        map_acp_child_exit_message(&c, "acp child process is not running"),
        AGENT_EXCEEDED_MEMORY_LIMIT_MSG
    );
}
