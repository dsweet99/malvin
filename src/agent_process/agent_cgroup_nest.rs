use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use tracing::warn;

use super::agent_cgroup::pid_cgroup_dir;

pub const LEAF_NAME: &str = "agent";
pub const LEAF_SHIM_MARKER: &str = "__agent-cgroup-leaf";
pub const OWN_GROUP_ARG: &str = ".";
const SCOPE_PREFIX: &str = "malvin-agent-";
const NESTED_PREFIX: &str = "malvin-nested-";
const STALE_KILL_WAIT: std::time::Duration = std::time::Duration::from_secs(2);

#[must_use]
pub fn child_cgroup_dirs(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter(is_dir_entry)
        .map(|entry| entry.path())
        .collect()
}

fn is_dir_entry(entry: &std::fs::DirEntry) -> bool {
    entry.file_type().is_ok_and(|t| t.is_dir())
}

pub fn remove_cgroup_tree(dir: &Path) {
    for child in child_cgroup_dirs(dir) {
        remove_cgroup_tree(&child);
    }
    match std::fs::remove_dir(dir) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => warn!(error = %e, dir = %dir.display(), "cannot remove nested agent cgroup"),
    }
}

#[must_use]
pub fn is_malvin_group_name(name: &str) -> bool {
    name.strip_prefix(SCOPE_PREFIX)
        .and_then(|rest| rest.strip_suffix(".scope"))
        .is_some()
        || name.starts_with(NESTED_PREFIX)
}

fn file_name(dir: &Path) -> Option<&str> {
    dir.file_name().and_then(OsStr::to_str)
}

#[must_use]
pub fn delegates_memory(group: &Path) -> bool {
    std::fs::read_to_string(group.join("cgroup.subtree_control"))
        .is_ok_and(|text| text.split_whitespace().any(|c| c == "memory"))
}

#[must_use]
pub fn nested_group_name(pid: u32, serial: u64) -> String {
    format!("{NESTED_PREFIX}{pid}-{serial}")
}

pub fn create_nested_group(parent: &Path, name: &str, limit_bytes: u64) -> std::io::Result<PathBuf> {
    sweep_stale_groups(parent);
    let group = parent.join(name);
    std::fs::create_dir(&group)?;
    if let Err(e) = std::fs::write(group.join("memory.max"), limit_bytes.to_string()) {
        remove_cgroup_tree(&group);
        return Err(e);
    }
    Ok(group)
}

#[must_use]
pub fn group_owner_pid(name: &str) -> Option<u32> {
    let rest = name
        .strip_prefix(NESTED_PREFIX)
        .or_else(|| name.strip_prefix(SCOPE_PREFIX)?.strip_suffix(".scope"))?;
    rest.split('-').next()?.parse().ok()
}

fn pid_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

#[must_use]
pub fn group_is_stale(dir: &Path) -> bool {
    let Some(owner) = file_name(dir).and_then(group_owner_pid) else {
        return false;
    };
    if pid_alive(owner) {
        return false;
    }
    if file_name(dir).is_some_and(|name| name.starts_with(NESTED_PREFIX)) {
        return true;
    }
    let mut members = std::collections::HashSet::new();
    super::agent_cgroup::collect_member_pids(dir, &mut members);
    members.into_iter().any(pid_alive)
}

pub fn sweep_stale_groups(parent: &Path) {
    for dir in child_cgroup_dirs(parent) {
        if !group_is_stale(&dir) {
            continue;
        }
        warn!(dir = %dir.display(), "killing agent cgroup left behind by a malvin that is gone");
        super::agent_cgroup::AgentCgroup::new(0, dir.clone()).kill_and_wait(STALE_KILL_WAIT);
        if file_name(&dir).is_some_and(|name| name.starts_with(NESTED_PREFIX)) {
            remove_cgroup_tree(&dir);
        }
    }
}

pub fn enter_leaf(group: &Path) -> std::io::Result<()> {
    let leaf = group.join(LEAF_NAME);
    match std::fs::create_dir(&leaf) {
        Err(e) if e.kind() != std::io::ErrorKind::AlreadyExists => return Err(e),
        _ => {}
    }
    std::fs::write(leaf.join("cgroup.procs"), "0")?;
    let _ = std::fs::write(group.join("cgroup.subtree_control"), "+memory");
    Ok(())
}

fn shim_group(arg: &OsStr) -> Option<PathBuf> {
    let dir = if arg == OWN_GROUP_ARG {
        pid_cgroup_dir(std::process::id())?
    } else {
        PathBuf::from(arg)
    };
    dir.starts_with(super::agent_cgroup::CGROUP_ROOT).then_some(dir)
}

#[must_use]
pub fn leaf_shim_invocation(
    mut args: impl Iterator<Item = OsString>,
) -> Option<(OsString, OsString, Vec<OsString>)> {
    if args.next()? != LEAF_SHIM_MARKER {
        return None;
    }
    let group = args.next()?;
    let program = args.next()?;
    Some((group, program, args.collect()))
}

#[must_use]
pub fn run_if_leaf_shim() -> Option<crate::Exit> {
    use std::os::unix::process::CommandExt;
    let (group, program, args) = leaf_shim_invocation(std::env::args_os().skip(1))?;
    let entered = shim_group(&group).map_or_else(
        || Err(std::io::Error::other("no cgroup v2 directory to enter")),
        |dir| enter_leaf(&dir),
    );
    if let Err(e) = entered {
        eprintln!("malvin: agent stays outside its cgroup leaf ({e}); using process-group containment");
    }
    let error = std::process::Command::new(&program).args(args).exec();
    eprintln!("malvin: cannot start {}: {error}", program.to_string_lossy());
    Some(crate::Exit::Failure)
}

#[cfg(test)]
#[path = "agent_cgroup_nest_tests.rs"]
mod agent_cgroup_nest_tests;
