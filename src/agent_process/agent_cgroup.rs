use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tracing::warn;

pub(crate) const CGROUP_ROOT: &str = "/sys/fs/cgroup";
const RELEASE_WAIT: Duration = Duration::from_secs(2);
const RELEASE_POLL: Duration = Duration::from_millis(10);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCgroup {
    leader_pid: u32,
    dir: PathBuf,
}

impl AgentCgroup {
    #[must_use]
    pub const fn new(leader_pid: u32, dir: PathBuf) -> Self {
        Self { leader_pid, dir }
    }

    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    #[must_use]
    pub fn member_pids(&self) -> HashSet<u32> {
        std::fs::read_to_string(self.dir.join("cgroup.procs"))
            .map(|text| parse_cgroup_procs(&text))
            .unwrap_or_default()
    }

    #[must_use]
    pub fn is_populated(&self) -> bool {
        std::fs::read_to_string(self.dir.join("cgroup.events"))
            .ok()
            .and_then(|text| parse_flat_keyed(&text, "populated"))
            .is_some_and(|n| n > 0)
    }

    #[must_use]
    pub fn oom_kill_count(&self) -> u64 {
        std::fs::read_to_string(self.dir.join("memory.events"))
            .ok()
            .and_then(|text| parse_flat_keyed(&text, "oom_kill"))
            .unwrap_or(0)
    }

    pub fn kill(&self) {
        match std::fs::write(self.dir.join("cgroup.kill"), "1") {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => warn!(error = %e, dir = %self.dir.display(), "cannot kill agent cgroup"),
        }
    }

    pub fn kill_and_wait(&self, max_wait: Duration) -> bool {
        if !self.is_populated() {
            return true;
        }
        self.kill();
        let start = Instant::now();
        while self.is_populated() {
            if start.elapsed() >= max_wait {
                return false;
            }
            std::thread::sleep(RELEASE_POLL);
        }
        true
    }
}

#[must_use]
pub(crate) fn parse_cgroup_procs(text: &str) -> HashSet<u32> {
    text.lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect()
}

#[must_use]
pub(crate) fn parse_flat_keyed(text: &str, key: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let (name, value) = line.split_once(' ')?;
        (name == key).then(|| value.trim().parse().ok()).flatten()
    })
}

#[must_use]
pub(crate) fn parse_unified_cgroup_path(proc_pid_cgroup: &str) -> Option<&str> {
    proc_pid_cgroup
        .lines()
        .find_map(|line| line.strip_prefix("0::"))
        .map(str::trim)
}

#[must_use]
pub(crate) fn pid_cgroup_dir(pid: u32) -> Option<PathBuf> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/cgroup")).ok()?;
    let rel = parse_unified_cgroup_path(&text)?;
    Some(Path::new(CGROUP_ROOT).join(rel.trim_start_matches('/')))
}

static ACTIVE_AGENT_CGROUP: Mutex<Option<AgentCgroup>> = Mutex::new(None);

fn active_slot() -> std::sync::MutexGuard<'static, Option<AgentCgroup>> {
    ACTIVE_AGENT_CGROUP
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn set_active_agent_cgroup(cgroup: Option<AgentCgroup>) {
    *active_slot() = cgroup;
}

#[must_use]
pub fn active_agent_cgroup_for(process_group_id: Option<u32>) -> Option<AgentCgroup> {
    let pgid = process_group_id?;
    active_slot()
        .as_ref()
        .filter(|cgroup| cgroup.leader_pid == pgid)
        .cloned()
}

#[must_use]
pub fn active_agent_cgroup_member_pids(process_group_id: Option<u32>) -> HashSet<u32> {
    active_agent_cgroup_for(process_group_id)
        .map(|cgroup| cgroup.member_pids())
        .unwrap_or_default()
}

pub fn kill_active_agent_cgroup(process_group_id: Option<u32>) {
    if let Some(cgroup) = active_agent_cgroup_for(process_group_id) {
        cgroup.kill();
    }
}

pub fn release_active_agent_cgroup() {
    let Some(cgroup) = active_slot().take() else {
        return;
    };
    if !cgroup.kill_and_wait(RELEASE_WAIT) {
        warn!(dir = %cgroup.dir.display(), "agent cgroup still populated after kill");
    }
}

#[cfg(test)]
#[path = "agent_cgroup_tests.rs"]
mod agent_cgroup_tests;
