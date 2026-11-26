use super::*;

#[test]
fn agent_scope_args_env_switch_and_unit_matching() {
    let args = scope_args("malvin-agent-1-0", 4096);
    assert_eq!(args.first().map(String::as_str), Some("--user"));
    assert!(args.contains(&"--scope".to_string()));
    assert!(args.contains(&"--unit=malvin-agent-1-0".to_string()));
    assert!(args.contains(&"--property=MemoryMax=4096".to_string()));
    assert!(args.contains(&"--property=Delegate=yes".to_string()));
    assert_eq!(args.last().map(String::as_str), Some("--"));
    crate::env_flag_tests::flags_are_on_only_for_non_empty_values_other_than_zero();
    assert!(!agent_cgroups_enabled());
    let mut ticket = SandboxSpawnTicket::for_test();
    let cmd = malvin_agent_tokio_command("node", Path::new("."), &mut ticket);
    assert_eq!(cmd.as_std().get_program(), "node");
    assert!(next_scope_unit().starts_with("malvin-agent-"));
    let _ = probe_systemd_scope;
}

#[test]
fn enclosure_detects_malvin_leaves_and_ancestors() {
    let tmp = tempfile::tempdir().expect("tmp");
    let scope = tmp.path().join("malvin-agent-1-0.scope");
    let leaf = scope.join(LEAF_NAME);
    std::fs::create_dir_all(&leaf).expect("leaf");
    assert_eq!(enclosure_of(&leaf), Enclosure::InsideMalvin);
    std::fs::write(scope.join("cgroup.subtree_control"), "memory\n").expect("ctl");
    assert_eq!(enclosure_of(&leaf), Enclosure::MalvinLeaf(scope.clone()));
    assert_eq!(enclosure_of(&scope), Enclosure::InsideMalvin);
    assert_eq!(enclosure_of(&scope.join("other")), Enclosure::InsideMalvin);
    assert_eq!(
        enclosure_of(&tmp.path().join("x.scope/agent")),
        Enclosure::Outside
    );
    assert_eq!(enclosure_of(Path::new("/")), Enclosure::Outside);
}

#[test]
fn nested_and_scope_commands_route_through_the_leaf_shim() {
    let tmp = tempfile::tempdir().expect("tmp");
    let mut ticket = SandboxSpawnTicket::for_test();
    let cmd = nested_command(OsStr::new("node"), tmp.path(), 4096, &mut ticket).expect("cmd");
    let args: Vec<_> = cmd.get_args().collect();
    assert_eq!(args.first().copied(), Some(OsStr::new(LEAF_SHIM_MARKER)));
    assert_eq!(args.last().copied(), Some(OsStr::new("node")));
    let group = PathBuf::from(args[1]);
    assert_eq!(
        std::fs::read_to_string(group.join("memory.max")).expect("max"),
        "4096"
    );
    assert!(
        nested_command(
            OsStr::new("node"),
            &tmp.path().join("missing"),
            1,
            &mut ticket
        )
        .is_none()
    );
    std::fs::remove_file(group.join("memory.max")).expect("rm");
    super::super::agent_scope_attach::attach_agent_scope(
        Some(ScopeTarget::Nested(group.clone())),
        Some(u32::MAX),
    );
    assert!(!group.exists());
    let scope = scope_command(OsStr::new("node"), 4096, &mut ticket);
    assert_eq!(scope.get_program(), SYSTEMD_RUN);
    let args: Vec<_> = scope.get_args().collect();
    assert!(args.contains(&OsStr::new(LEAF_SHIM_MARKER)));
    assert_eq!(args.last().copied(), Some(OsStr::new("node")));
}
