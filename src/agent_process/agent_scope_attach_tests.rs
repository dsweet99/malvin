use super::*;

#[test]
fn attach_waits_for_the_agent_and_matches_units() {
    let dir = Path::new("/sys/fs/cgroup/u/app.slice/malvin-agent-1-0.scope");
    assert!(cgroup_dir_is_unit_scope(dir, "malvin-agent-1-0"));
    assert!(!cgroup_dir_is_unit_scope(dir, "malvin-agent-1-1"));
    assert!(wait_for_agent_cgroup(u32::MAX, &ScopeTarget::Unit("x".into())).is_none());
    attach_agent_scope(Some(ScopeTarget::Unit("malvin-agent-never".into())), None);
    assert_eq!(without_stalling_runtime(|| 7), 7);
    let tmp = tempfile::tempdir().expect("tmp");
    let leaf = AgentCgroup::with_leaf(1, tmp.path().join("malvin-agent-1-0.scope"), false);
    sweep_stale_sibling_scopes(&leaf, &ScopeTarget::Unit("malvin-agent-1-0".into()));
    sweep_stale_sibling_scopes(&leaf, &ScopeTarget::Nested(tmp.path().to_path_buf()));
}

#[test]
fn leaf_and_root_cgroups_match_their_targets() {
    let scope = PathBuf::from("/sys/fs/cgroup/a/malvin-agent-1-0.scope");
    let unit = ScopeTarget::Unit("malvin-agent-1-0".into());
    let leaf = leaf_cgroup(5, &scope.join(LEAF_NAME), &unit).expect("leaf");
    assert_eq!(leaf.dir(), scope);
    assert!(leaf_cgroup(5, &scope, &unit).is_none());
    assert!(leaf_cgroup(5, &scope.join("other"), &unit).is_none());
    let nested = ScopeTarget::Nested(scope.join("malvin-nested-2-0"));
    let inner = leaf_cgroup(5, &scope.join("malvin-nested-2-0/agent"), &nested).expect("inner");
    assert_eq!(inner.dir(), scope.join("malvin-nested-2-0"));
    assert!(leaf_cgroup(5, &scope.join(LEAF_NAME), &nested).is_none());
    assert_eq!(root_cgroup(5, scope.clone(), &unit).map(|c| c.dir().to_path_buf()), Some(scope.clone()));
    assert!(root_cgroup(5, scope.join(LEAF_NAME), &unit).is_none());
    assert!(root_cgroup(5, scope, &nested).is_none());
}

