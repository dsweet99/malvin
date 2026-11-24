use super::*;

fn fake_cgroup(procs: &str, events: &str, memory_events: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tmp");
    std::fs::write(tmp.path().join("cgroup.procs"), procs).expect("procs");
    std::fs::write(tmp.path().join("cgroup.events"), events).expect("events");
    std::fs::write(tmp.path().join("memory.events"), memory_events).expect("memory.events");
    tmp
}

#[test]
fn agent_cgroup_reads_members_population_oom_and_kills_via_registry() {
    assert_eq!(
        parse_unified_cgroup_path("0::/a/b.scope\n"),
        Some("/a/b.scope")
    );
    assert_eq!(parse_unified_cgroup_path("1:name=x:/y\n"), None);
    assert_eq!(parse_flat_keyed("low 0\noom_kill 3\n", "oom_kill"), Some(3));
    assert_eq!(parse_flat_keyed("oom_kill_x 3\n", "oom_kill"), None);
    let tmp = fake_cgroup(
        "10\n11\n\n",
        "populated 1\nfrozen 0\n",
        "oom 1\noom_kill 2\n",
    );
    let cgroup = AgentCgroup::new(10, tmp.path().to_path_buf());
    assert_eq!(cgroup.dir(), tmp.path());
    assert_eq!(cgroup.member_pids(), HashSet::from([10, 11]));
    assert!(cgroup.is_populated());
    assert_eq!(cgroup.oom_kill_count(), 2);
    set_active_agent_cgroup(Some(cgroup));
    assert!(active_agent_cgroup_member_pids(Some(99)).is_empty());
    assert_eq!(active_agent_cgroup_member_pids(Some(10)).len(), 2);
    kill_active_agent_cgroup(Some(10));
    assert_eq!(
        std::fs::read_to_string(tmp.path().join("cgroup.kill")).expect("kill"),
        "1"
    );
    std::fs::write(tmp.path().join("cgroup.events"), "populated 0\n").expect("events");
    release_active_agent_cgroup();
    assert!(active_agent_cgroup_for(Some(10)).is_none());
    let gone = AgentCgroup::new(1, tmp.path().join("missing"));
    gone.kill();
    assert!(gone.kill_and_wait(Duration::ZERO));
    assert!(!gone.is_populated() && gone.member_pids().is_empty() && gone.oom_kill_count() == 0);
}

#[test]
fn pid_cgroup_dir_handles_host_without_unified_cgroups() {
    assert!(pid_cgroup_dir(u32::MAX).is_none());
    #[cfg(target_os = "linux")]
    if let Some(dir) = pid_cgroup_dir(std::process::id()) {
        assert!(dir.starts_with(CGROUP_ROOT));
    }
    #[cfg(not(target_os = "linux"))]
    assert!(pid_cgroup_dir(std::process::id()).is_none());
}

#[test]
fn leaf_layout_counts_nested_members_and_attributes_oom_locally() {
    let tmp = tempfile::tempdir().expect("tmp");
    let group = tmp.path().join("malvin-nested-1-0");
    std::fs::create_dir_all(group.join("agent")).expect("leaf");
    std::fs::create_dir_all(group.join("malvin-nested-2-0/agent")).expect("child");
    std::fs::write(group.join("agent/cgroup.procs"), "10\n").expect("procs");
    std::fs::write(group.join("malvin-nested-2-0/agent/cgroup.procs"), "11\n").expect("procs");
    std::fs::write(group.join("memory.events"), "oom 9\noom_kill 9\n").expect("events");
    let cgroup = AgentCgroup::with_leaf(10, group.clone(), true);
    assert_eq!(cgroup.member_pids(), HashSet::from([10, 11]));
    assert_eq!(cgroup.oom_kill_count(), 0);
    std::fs::write(group.join("memory.events.local"), "oom 1\noom_kill 0\n").expect("local");
    std::fs::write(group.join("agent/memory.events.local"), "oom 0\noom_kill 2\n").expect("leaf");
    assert_eq!(cgroup.oom_kill_count(), 3);
    assert_eq!(AgentCgroup::new(10, group.clone()).oom_kill_count(), 9);
    set_active_agent_cgroup(Some(cgroup));
    release_active_agent_cgroup();
    assert!(group.exists());
}
