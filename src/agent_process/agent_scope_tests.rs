use super::*;

#[test]
fn agent_scope_args_env_switch_and_unit_matching() {
    let args = scope_args("malvin-agent-1-0", 4096);
    assert_eq!(args.first().map(String::as_str), Some("--user"));
    assert!(args.contains(&"--scope".to_string()));
    assert!(args.contains(&"--unit=malvin-agent-1-0".to_string()));
    assert!(args.contains(&"--property=MemoryMax=4096".to_string()));
    assert_eq!(args.last().map(String::as_str), Some("--"));
    crate::env_flag_tests::flags_are_on_only_for_non_empty_values_other_than_zero();
    let dir = Path::new("/sys/fs/cgroup/u/app.slice/malvin-agent-1-0.scope");
    assert!(cgroup_dir_is_unit_scope(dir, "malvin-agent-1-0"));
    assert!(!cgroup_dir_is_unit_scope(dir, "malvin-agent-1-1"));
    assert!(!agent_cgroups_enabled());
    let mut ticket = SandboxSpawnTicket::for_test();
    let cmd = malvin_agent_tokio_command("node", Path::new("."), &mut ticket);
    assert_eq!(cmd.as_std().get_program(), "node");
    assert!(next_scope_unit().starts_with("malvin-agent-"));
    assert!(wait_for_scope_cgroup(u32::MAX, "x").is_none());
    let _ = probe_systemd_scope;
    attach_agent_scope(Some("malvin-agent-never".into()), None);
    assert_eq!(without_stalling_runtime(|| 7), 7);
}
