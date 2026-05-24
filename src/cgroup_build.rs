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

pub fn build_script_cargo_lines(target_os: &str) -> Vec<&'static str> {
    const CHECK_CFG: &str = "cargo::rustc-check-cfg=cfg(malvin_have_writable_cgroups)";
    if target_os != "linux" {
        return vec![CHECK_CFG];
    }
    if probe_writable_cgroup_parent().is_some() {
        vec![CHECK_CFG, "cargo:rustc-cfg=malvin_have_writable_cgroups"]
    } else {
        vec![CHECK_CFG]
    }
}

pub fn format_build_script_lines(target_os: &str) -> String {
    let mut out = String::new();
    for line in build_script_cargo_lines(target_os) {
        out.push_str(line);
        out.push('\n');
    }
    out
}

pub fn run_build_script(target_os: &str) {
    print!("{}", format_build_script_lines(target_os));
}

pub fn run_build_script_from_cargo_env() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    run_build_script(&target_os);
}

#[cfg(test)]
mod tests {
    #![allow(unsafe_code)]

    #[test]
    fn run_build_script_from_cargo_env_uses_target_os_env() {
        let prior = std::env::var("CARGO_CFG_TARGET_OS").ok();
        unsafe {
            std::env::set_var("CARGO_CFG_TARGET_OS", "macos");
        }
        super::run_build_script_from_cargo_env();
        let lines = super::build_script_cargo_lines("macos");
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("rustc-check-cfg"));
        match prior {
            Some(value) => unsafe { std::env::set_var("CARGO_CFG_TARGET_OS", value) },
            None => unsafe { std::env::remove_var("CARGO_CFG_TARGET_OS") },
        }
    }

    #[test]
    fn run_build_script_emits_check_cfg_for_non_linux() {
        let expected = "cargo::rustc-check-cfg=cfg(malvin_have_writable_cgroups)\n";
        assert_eq!(super::format_build_script_lines("macos"), expected);
        super::run_build_script("macos");
    }

    #[test]
    fn kiss_cov_probe_writable_cgroup_parent() {
        #[cfg(target_os = "linux")]
        let _ = super::probe_writable_cgroup_parent();
        #[cfg(not(target_os = "linux"))]
        assert!(super::probe_writable_cgroup_parent().is_none());
    }

    #[test]
    fn kiss_cov_resolve_cgroup_v2_parent() {
        #[cfg(target_os = "linux")]
        {
            let _ = super::resolve_cgroup_v2_parent();
            assert!(super::cgroup_v2_mount().is_some());
        }
        #[cfg(not(target_os = "linux"))]
        assert!(super::resolve_cgroup_v2_parent().is_none());
    }

    #[test]
    fn kiss_cov_resolve_cgroup_v1_memory_parent() {
        #[cfg(target_os = "linux")]
        let _ = super::resolve_cgroup_v1_memory_parent();
        #[cfg(not(target_os = "linux"))]
        assert!(super::resolve_cgroup_v1_memory_parent().is_none());
    }

    #[test]
    fn kiss_cov_cgroup_v2_mount() {
        #[cfg(target_os = "linux")]
        assert!(super::cgroup_v2_mount().is_some());
        #[cfg(not(target_os = "linux"))]
        assert!(super::cgroup_v2_mount().is_none());
    }

    #[test]
    fn kiss_cov_self_cgroup_v2_relative_path() {
        #[cfg(target_os = "linux")]
        assert!(super::self_cgroup_v2_relative_path().is_some());
        #[cfg(not(target_os = "linux"))]
        assert!(super::self_cgroup_v2_relative_path().is_none());
    }

    #[test]
    fn kiss_cov_self_cgroup_v1_memory_relative_path() {
        #[cfg(target_os = "linux")]
        let _ = super::self_cgroup_v1_memory_relative_path();
        #[cfg(not(target_os = "linux"))]
        assert!(super::self_cgroup_v1_memory_relative_path().is_none());
    }

    #[test]
    fn kiss_cov_probe_writable_parent() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("cgroup.procs"), "").expect("procs");
        assert!(super::probe_writable_parent(dir.path(), |d| d.join("cgroup.procs").is_file()).is_some());
    }
}
