use super::*;

#[test]
fn agent_scope_args_env_switch_and_unit_matching() {
    let args = scope_args("malvin-agent-1-0", 4096);
    assert_eq!(args.first().map(String::as_str), Some("--user"));
    assert!(args.contains(&"--scope".to_string()));
    assert!(args.contains(&"--unit=malvin-agent-1-0".to_string()));
    assert!(args.contains(&"--property=MemoryMax=4096".to_string()));
    assert_eq!(args.last().map(String::as_str), Some("--"));
    assert!(cgroup_disabled_by_env(Some(OsStr::new("1"))));
    assert!(!cgroup_disabled_by_env(Some(OsStr::new("0"))));
    assert!(!cgroup_disabled_by_env(None));
    let dir = Path::new("/sys/fs/cgroup/u/app.slice/malvin-agent-1-0.scope");
    assert!(cgroup_dir_is_unit_scope(dir, "malvin-agent-1-0"));
    assert!(!cgroup_dir_is_unit_scope(dir, "malvin-agent-1-1"));
    assert!(!agent_cgroups_enabled());
    let cmd = malvin_agent_tokio_command("node", Path::new("."));
    assert_eq!(cmd.as_std().get_program(), "node");
    assert!(next_scope_unit().starts_with("malvin-agent-"));
    assert!(wait_for_scope_cgroup(u32::MAX, "x").is_none());
    let _ = probe_systemd_scope;
    *pending_slot() = Some("malvin-agent-never".into());
    attach_pending_agent_scope(None);
    assert!(pending_slot().is_none());
}
