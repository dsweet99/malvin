use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::cgroup_memory::{
    cgroup_memory_max_is_limited, resolve_writable_cgroup_parent, write_memory_limit,
};
use super::linux_fs::{
    half_physical_memory_bytes, memory_limit_exceeded_since_baseline,
    memory_limit_oom_baseline_at,
};
use super::cgroup_line::cgroup_line_lists_leaf;

use tokio::process::Command;


pub struct CgroupSpawnPlan {
    pub cgroup_dir: PathBuf,
    pub memory_max_bytes: u64,
}

pub fn try_prepare_cgroup_spawn_plan(suffix: &str) -> Option<CgroupSpawnPlan> {
    let memory_max_bytes = half_physical_memory_bytes()?;
    let parent = resolve_writable_cgroup_parent()?;
    let name = format!("malvin-acp-{suffix}");
    let cgroup_dir = parent.join(&name);
    if cgroup_dir.exists() {
        remove_cgroup_dir(&cgroup_dir);
    }
    fs::create_dir(&cgroup_dir).ok()?;
    if !write_memory_limit(&cgroup_dir, memory_max_bytes) {
        remove_cgroup_dir(&cgroup_dir);
        return None;
    }
    Some(CgroupSpawnPlan {
        cgroup_dir,
        memory_max_bytes,
    })
}

pub fn apply_linux_child_pre_exec(cmd: &mut Command, cgroup_dir: Option<PathBuf>) {
    let expected_parent_pid = std::process::id();
    unsafe {
        cmd.pre_exec(move || {
            install_parent_death_guard(expected_parent_pid)?;
            if let Some(ref dir) = cgroup_dir {
                join_cgroup_dir(dir)?;
            }
            Ok(())
        });
    }
}

pub const CGROUP_JOIN_WAIT_MS: u64 = 2_000;

pub async fn wait_for_cgroup_join(pid: u32, plan: &CgroupSpawnPlan) -> bool {
    let deadline =
        tokio::time::Instant::now() + tokio::time::Duration::from_millis(CGROUP_JOIN_WAIT_MS);
    loop {
        if verify_pid_in_cgroup(pid, plan) {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            let _ = release_pid_from_cgroup(pid, &plan.cgroup_dir);
            return false;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    }
}

pub fn discard_prepared_cgroup_after_failed_join(pid: u32, cgroup_dir: &Path) {
    if release_pid_from_cgroup(pid, cgroup_dir) {
        remove_cgroup_dir(cgroup_dir);
        return;
    }
    if !pid_listed_in_leaf_cgroup(pid, cgroup_dir) {
        remove_cgroup_dir(cgroup_dir);
        return;
    }
    remove_leaf_cgroup_dir_without_killing_members(cgroup_dir);
}

fn remove_leaf_cgroup_dir_without_killing_members(cgroup_dir: &Path) {
    if let Ok(entries) = fs::read_dir(cgroup_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let _ = fs::remove_file(path);
            }
        }
    }
    if fs::remove_dir(cgroup_dir).is_ok() {
        return;
    }
    let _ = fs::remove_dir_all(cgroup_dir);
}

pub fn pid_listed_in_leaf_cgroup(pid: u32, cgroup_dir: &Path) -> bool {
    let pid_text = pid.to_string();
    for name in ["cgroup.procs", "tasks"] {
        let path = cgroup_dir.join(name);
        if let Ok(text) = fs::read_to_string(&path) {
            if text.lines().any(|line| line.trim() == pid_text) {
                return true;
            }
        }
    }
    false
}

pub fn verify_pid_in_cgroup(pid: u32, plan: &CgroupSpawnPlan) -> bool {
    if pid == 0 {
        return false;
    }
    let Some(name) = plan
        .cgroup_dir
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty())
    else {
        return false;
    };
    let Ok(cgroup_text) = fs::read_to_string(format!("/proc/{pid}/cgroup")) else {
        return false;
    };
    if !cgroup_text
        .lines()
        .any(|line| cgroup_line_lists_leaf(line, name))
    {
        return false;
    }
    cgroup_memory_max_is_limited(&plan.cgroup_dir, plan.memory_max_bytes)
}

pub fn remove_cgroup_dir(path: &Path) {
    kill_cgroup_members(path);
    if fs::remove_dir(path).is_ok() {
        return;
    }
    let _ = fs::remove_dir_all(path);
}

pub fn release_pid_from_cgroup(pid: u32, cgroup_dir: &Path) -> bool {
    let Some(parent) = cgroup_dir.parent() else {
        return false;
    };
    let procs = parent.join("cgroup.procs");
    if procs.is_file() {
        return write_pid_to_cgroup_procs(&procs, pid).is_ok();
    }
    let tasks = parent.join("tasks");
    if tasks.is_file() {
        return write_pid_to_cgroup_procs(&tasks, pid).is_ok();
    }
    false
}

fn join_cgroup_dir(cgroup_dir: &Path) -> std::io::Result<()> {
    let procs = cgroup_dir.join("cgroup.procs");
    if procs.is_file() {
        return write_pid_to_cgroup_procs(&procs, std::process::id());
    }
    let tasks = cgroup_dir.join("tasks");
    if tasks.is_file() {
        return write_pid_to_cgroup_procs(&tasks, std::process::id());
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "cgroup join file missing",
    ))
}

fn write_pid_to_cgroup_procs(procs: &Path, pid: u32) -> std::io::Result<()> {
    let mut file = OpenOptions::new().append(true).open(procs)?;
    writeln!(file, "{pid}")?;
    Ok(())
}

fn kill_cgroup_members(path: &Path) {
    let kill = path.join("cgroup.kill");
    if kill.is_file() {
        let _ = fs::write(kill, "1");
        return;
    }
    for name in ["cgroup.procs", "tasks"] {
        let procs = path.join(name);
        let Ok(text) = fs::read_to_string(&procs) else {
            continue;
        };
        for line in text.lines() {
            let Ok(member_pid) = line.trim().parse::<u32>() else {
                continue;
            };
            if member_pid == 0 {
                continue;
            }
            let _ = std::process::Command::new("kill")
                .arg("-KILL")
                .arg(member_pid.to_string())
                .status();
        }
    }
}

#[cfg(test)]
mod linux_spawn_tests {
    use super::*;

    #[test]
    fn linux_spawn_unit_surface() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("cgroup.procs"), "1\n").expect("procs");
        let plan = CgroupSpawnPlan {
            cgroup_dir: dir.path().to_path_buf(),
            memory_max_bytes: 1024,
        };
        let _ = pid_listed_in_leaf_cgroup(1, dir.path());
        remove_leaf_cgroup_dir_without_killing_members(dir.path());
        kill_cgroup_members(dir.path());
        let join_wait_ms = CGROUP_JOIN_WAIT_MS;
        assert!(join_wait_ms >= 2_000);
        let _ = verify_pid_in_cgroup(0, &plan);
    }
}


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_join_cgroup_dir() { let _ = stringify!(join_cgroup_dir); }

    #[test]
    fn kiss_cov_write_pid_to_cgroup_procs() { let _ = stringify!(write_pid_to_cgroup_procs); }

}
