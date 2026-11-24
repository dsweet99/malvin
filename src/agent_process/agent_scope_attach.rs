use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tracing::warn;

use super::agent_cgroup::{
    AgentCgroup, pid_cgroup_dir, set_active_agent_cgroup, without_stalling_runtime,
};
use super::agent_cgroup_nest::{LEAF_NAME, remove_cgroup_tree};
use super::agent_scope::ScopeTarget;

const ATTACH_WAIT: Duration = Duration::from_secs(5);
const ATTACH_POLL: Duration = Duration::from_millis(10);

#[must_use]
pub(crate) fn cgroup_dir_is_unit_scope(dir: &Path, unit: &str) -> bool {
    dir.file_name()
        .and_then(OsStr::to_str)
        .and_then(|name| name.strip_suffix(".scope"))
        .is_some_and(|name| name == unit)
}

#[must_use]
pub(crate) fn leaf_cgroup(pid: u32, dir: &Path, target: &ScopeTarget) -> Option<AgentCgroup> {
    if dir.file_name() != Some(OsStr::new(LEAF_NAME)) {
        return None;
    }
    let group = dir.parent()?;
    match target {
        ScopeTarget::Unit(unit) => cgroup_dir_is_unit_scope(group, unit)
            .then(|| AgentCgroup::with_leaf(pid, group.to_path_buf(), false)),
        ScopeTarget::Nested(nested) => {
            (group == nested).then(|| AgentCgroup::with_leaf(pid, nested.clone(), true))
        }
    }
}

#[must_use]
pub(crate) fn root_cgroup(pid: u32, dir: PathBuf, target: &ScopeTarget) -> Option<AgentCgroup> {
    match target {
        ScopeTarget::Unit(unit) if cgroup_dir_is_unit_scope(&dir, unit) => {
            Some(AgentCgroup::new(pid, dir))
        }
        _ => None,
    }
}

fn sweep_stale_sibling_scopes(cgroup: &AgentCgroup, target: &ScopeTarget) {
    if let (ScopeTarget::Unit(_), Some(parent)) = (target, cgroup.dir().parent()) {
        super::agent_cgroup_nest::sweep_stale_groups(parent);
    }
}

fn wait_for_agent_cgroup(pid: u32, target: &ScopeTarget) -> Option<AgentCgroup> {
    let start = Instant::now();
    loop {
        let dir = pid_cgroup_dir(pid)?;
        if let Some(cgroup) = leaf_cgroup(pid, &dir, target) {
            sweep_stale_sibling_scopes(&cgroup, target);
            return Some(cgroup);
        }
        if start.elapsed() >= ATTACH_WAIT {
            return root_cgroup(pid, dir, target);
        }
        std::thread::sleep(ATTACH_POLL);
    }
}

pub(crate) fn attach_agent_scope(target: Option<ScopeTarget>, agent_pid: Option<u32>) {
    let (Some(target), Some(pid)) = (target, agent_pid) else {
        set_active_agent_cgroup(None);
        return;
    };
    let cgroup = without_stalling_runtime(|| wait_for_agent_cgroup(pid, &target));
    if cgroup.is_none() {
        warn!(
            ?target,
            pid, "agent did not enter its cgroup; using process-group containment"
        );
        if let ScopeTarget::Nested(group) = &target {
            remove_cgroup_tree(group);
        }
    }
    set_active_agent_cgroup(cgroup);
}

#[cfg(test)]
#[path = "agent_scope_attach_tests.rs"]
mod agent_scope_attach_tests;
