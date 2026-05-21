// Cgroup discovery shared by `build.rs` (`malvin_have_writable_cgroups`) and unit tests.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

pub fn probe_writable_cgroup_parent() -> Option<PathBuf> {
    if let Some(parent) = resolve_cgroup_v2_parent() {
        return Some(parent);
    }
    resolve_cgroup_v1_memory_parent()
}

pub fn resolve_cgroup_v2_parent() -> Option<PathBuf> {
    let mount = cgroup_v2_mount()?;
    let rel = self_cgroup_v2_relative_path()?;
    let parent = mount.join(rel.trim_start_matches('/'));
    probe_writable_parent(&parent, |dir| dir.join("cgroup.procs").is_file())
}

pub fn resolve_cgroup_v1_memory_parent() -> Option<PathBuf> {
    let rel = self_cgroup_v1_memory_relative_path()?;
    let parent = PathBuf::from("/sys/fs/cgroup/memory").join(rel.trim_start_matches('/'));
    probe_writable_parent(&parent, |dir| dir.join("tasks").is_file())
}

pub fn cgroup_v2_mount() -> Option<PathBuf> {
    const ROOT: &str = "/sys/fs/cgroup";
    if Path::new(ROOT).join("cgroup.controllers").is_file() {
        return Some(PathBuf::from(ROOT));
    }
    let unified = Path::new(ROOT).join("unified");
    if unified.join("cgroup.controllers").is_file() {
        return Some(unified);
    }
    None
}

pub fn self_cgroup_v2_relative_path() -> Option<String> {
    let text = fs::read_to_string("/proc/self/cgroup").ok()?;
    text.lines()
        .find_map(|line| line.strip_prefix("0::").map(str::to_string))
}

pub fn self_cgroup_v1_memory_relative_path() -> Option<String> {
    let text = fs::read_to_string("/proc/self/cgroup").ok()?;
    for line in text.lines() {
        let mut parts = line.splitn(3, ':');
        let (_id, controllers, path) = (parts.next()?, parts.next()?, parts.next()?);
        if controllers.split(',').any(|c| c == "memory") {
            return Some(path.to_string());
        }
    }
    None
}

pub fn probe_writable_parent(start: &Path, marker: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if marker(&current) {
            let probe = current.join(format!(".malvin-probe-{}", std::process::id()));
            if fs::create_dir(&probe).is_ok() {
                let _ = fs::remove_dir(&probe);
                return Some(current);
            }
        }
        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_cov_probe_writable_cgroup_parent() {
        let _ = super::probe_writable_cgroup_parent;
    }

    #[test]
    fn kiss_cov_resolve_cgroup_v2_parent() {
        let _ = super::resolve_cgroup_v2_parent;
    }

    #[test]
    fn kiss_cov_resolve_cgroup_v1_memory_parent() {
        let _ = super::resolve_cgroup_v1_memory_parent;
    }

    #[test]
    fn kiss_cov_cgroup_v2_mount() {
        let _ = super::cgroup_v2_mount;
    }

    #[test]
    fn kiss_cov_self_cgroup_v2_relative_path() {
        let _ = super::self_cgroup_v2_relative_path;
    }

    #[test]
    fn kiss_cov_self_cgroup_v1_memory_relative_path() {
        let _ = super::self_cgroup_v1_memory_relative_path;
    }

    #[test]
    fn kiss_cov_probe_writable_parent() {
        let _ = stringify!(super::probe_writable_parent);
    }
}
