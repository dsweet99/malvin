use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo::rustc-check-cfg=cfg(malvin_have_writable_cgroups)");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("linux") {
        return;
    }
    if probe_writable_cgroup_parent().is_some() {
        println!("cargo:rustc-cfg=malvin_have_writable_cgroups");
    }
}

fn probe_writable_cgroup_parent() -> Option<PathBuf> {
    if let Some(parent) = resolve_cgroup_v2_parent() {
        return Some(parent);
    }
    resolve_cgroup_v1_memory_parent()
}

fn resolve_cgroup_v2_parent() -> Option<PathBuf> {
    let mount = cgroup_v2_mount()?;
    let rel = self_cgroup_v2_relative_path()?;
    let parent = mount.join(rel.trim_start_matches('/'));
    probe_writable_parent(&parent, |dir| dir.join("cgroup.procs").is_file())
}

fn resolve_cgroup_v1_memory_parent() -> Option<PathBuf> {
    let rel = self_cgroup_v1_memory_relative_path()?;
    let parent = PathBuf::from("/sys/fs/cgroup/memory").join(rel.trim_start_matches('/'));
    probe_writable_parent(&parent, |dir| dir.join("tasks").is_file())
}

fn cgroup_v2_mount() -> Option<PathBuf> {
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

fn self_cgroup_v2_relative_path() -> Option<String> {
    let text = fs::read_to_string("/proc/self/cgroup").ok()?;
    text.lines()
        .find_map(|line| line.strip_prefix("0::").map(str::to_string))
}

fn self_cgroup_v1_memory_relative_path() -> Option<String> {
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

fn probe_writable_parent(start: &Path, marker: impl Fn(&Path) -> bool) -> Option<PathBuf> {
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
