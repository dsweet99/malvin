use super::*;

const DEAD_PID: u32 = 999_999_999;

fn tree() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tmp");
    std::fs::create_dir_all(tmp.path().join("g/agent")).expect("leaf");
    std::fs::create_dir_all(tmp.path().join("g/n/agent")).expect("nested");
    tmp
}

#[test]
fn cgroup_tree_listing_and_removal() {
    let tmp = tree();
    let g = tmp.path().join("g");
    let mut children = child_cgroup_dirs(&g);
    children.sort();
    assert_eq!(children, vec![g.join("agent"), g.join("n")]);
    assert!(child_cgroup_dirs(&g.join("missing")).is_empty());
    remove_cgroup_tree(&g);
    assert!(!g.exists());
    remove_cgroup_tree(&g);
    std::fs::create_dir(&g).expect("g");
    std::fs::write(g.join("file"), "x").expect("file");
    remove_cgroup_tree(&g);
    assert!(g.exists());
}

#[test]
fn group_names_owners_and_memory_delegation() {
    assert!(is_malvin_group_name("malvin-agent-12-0.scope"));
    assert!(is_malvin_group_name("malvin-nested-12-3"));
    assert!(!is_malvin_group_name("malvin-agent-12-0"));
    assert!(!is_malvin_group_name("app.slice"));
    assert_eq!(nested_group_name(12, 3), "malvin-nested-12-3");
    assert_eq!(group_owner_pid("malvin-nested-12-3"), Some(12));
    assert_eq!(group_owner_pid("malvin-agent-34-0.scope"), Some(34));
    assert_eq!(group_owner_pid("malvin-agent-34-0"), None);
    assert_eq!(group_owner_pid("agent"), None);
    let tmp = tempfile::tempdir().expect("tmp");
    assert!(!delegates_memory(tmp.path()));
    std::fs::write(tmp.path().join("cgroup.subtree_control"), "cpu memory pids\n").expect("w");
    assert!(delegates_memory(tmp.path()));
    std::fs::write(tmp.path().join("cgroup.subtree_control"), "memoryx\n").expect("w");
    assert!(!delegates_memory(tmp.path()));
}

#[test]
fn stale_groups_need_a_dead_owner() {
    let tmp = tempfile::tempdir().expect("tmp");
    let own = std::process::id();
    let mk = |name: &str| {
        let dir = tmp.path().join(name);
        std::fs::create_dir(&dir).expect("dir");
        dir
    };
    let dead_nested = mk(&nested_group_name(DEAD_PID, 0));
    let live_nested = mk(&nested_group_name(own, 0));
    let dead_empty_scope = mk(&format!("malvin-agent-{DEAD_PID}-0.scope"));
    let dead_busy_scope = mk(&format!("malvin-agent-{DEAD_PID}-1.scope"));
    std::fs::write(dead_busy_scope.join("cgroup.procs"), format!("{own}\n")).expect("procs");
    let other = mk("other");
    assert!(group_is_stale(&dead_nested));
    assert!(!group_is_stale(&live_nested));
    assert!(!group_is_stale(&dead_empty_scope));
    assert!(group_is_stale(&dead_busy_scope));
    assert!(!group_is_stale(&other));
    sweep_stale_groups(tmp.path());
    assert!(!dead_nested.exists());
    assert!(live_nested.exists() && dead_busy_scope.exists() && other.exists());
}

#[test]
fn nested_group_creation_and_leaf_entry() {
    let tmp = tempfile::tempdir().expect("tmp");
    let group = create_nested_group(tmp.path(), "malvin-nested-1-0", 4096).expect("group");
    assert_eq!(std::fs::read_to_string(group.join("memory.max")).expect("max"), "4096");
    assert!(create_nested_group(tmp.path(), "malvin-nested-1-0", 1).is_err());
    assert!(create_nested_group(&tmp.path().join("missing"), "x", 1).is_err());
    enter_leaf(&group).expect("leaf");
    enter_leaf(&group).expect("leaf again");
    assert_eq!(
        std::fs::read_to_string(group.join("agent/cgroup.procs")).expect("procs"),
        "0"
    );
    assert_eq!(
        std::fs::read_to_string(group.join("cgroup.subtree_control")).expect("ctl"),
        "+memory"
    );
    assert!(enter_leaf(&tmp.path().join("missing")).is_err());
}

#[test]
fn leaf_shim_arguments_and_group_paths() {
    let args = |v: &[&str]| v.iter().map(OsString::from).collect::<Vec<_>>().into_iter();
    let parsed = leaf_shim_invocation(args(&[LEAF_SHIM_MARKER, ".", "node", "a", "b"]));
    assert_eq!(
        parsed,
        Some((".".into(), "node".into(), vec!["a".into(), "b".into()]))
    );
    assert!(leaf_shim_invocation(args(&[LEAF_SHIM_MARKER, "."])).is_none());
    assert!(leaf_shim_invocation(args(&["__summary-worker", ".", "x"])).is_none());
    assert!(leaf_shim_invocation(args(&[])).is_none());
    assert!(shim_group(OsStr::new("/tmp")).is_none());
    assert_eq!(
        shim_group(OsStr::new("/sys/fs/cgroup/x")),
        Some(PathBuf::from("/sys/fs/cgroup/x"))
    );
    let own = shim_group(OsStr::new(OWN_GROUP_ARG));
    assert_eq!(own, pid_cgroup_dir(std::process::id()));
    assert!(run_if_leaf_shim().is_none());
}
