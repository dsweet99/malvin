use std::ffi::OsStr;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use tracing::{info, warn};

use super::agent_cgroup::{AgentCgroup, pid_cgroup_dir, set_active_agent_cgroup};

pub const NO_CGROUP_ENV: &str = "MALVIN_NO_CGROUP";
const SYSTEMD_RUN: &str = "systemd-run";
const ATTACH_WAIT: Duration = Duration::from_secs(5);
const ATTACH_POLL: Duration = Duration::from_millis(10);

static SCOPE_PROBE: OnceLock<bool> = OnceLock::new();
static NEXT_SCOPE_SERIAL: AtomicU64 = AtomicU64::new(0);
static PENDING_SCOPE_UNIT: Mutex<Option<String>> = Mutex::new(None);

#[must_use]
pub(crate) fn scope_args(unit: &str, mem_limit_bytes: u64) -> Vec<String> {
    vec![
        "--user".into(),
        "--scope".into(),
        "--quiet".into(),
        "--collect".into(),
        format!("--unit={unit}"),
        format!("--property=MemoryMax={mem_limit_bytes}"),
        "--property=OOMPolicy=continue".into(),
        "--".into(),
    ]
}

#[must_use]
pub(crate) fn cgroup_disabled_by_env(value: Option<&OsStr>) -> bool {
    value.is_some_and(|v| !v.is_empty() && v != "0")
}

fn probe_systemd_scope() -> bool {
    let unit = next_scope_unit();
    let ok = std::process::Command::new(SYSTEMD_RUN)
        .args(scope_args(&unit, 1 << 40))
        .arg("true")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if ok {
        info!("malvin agent sessions run in per-session systemd scopes (cgroup v2)");
    } else {
        info!("systemd user scopes unavailable; agent sessions use process-group containment");
    }
    ok
}

#[must_use]
pub fn agent_cgroups_enabled() -> bool {
    if cfg!(test)
        || !cfg!(target_os = "linux")
        || cgroup_disabled_by_env(std::env::var_os(NO_CGROUP_ENV).as_deref())
        || crate::agent_process::test_no_real_agent_enabled()
    {
        return false;
    }
    *SCOPE_PROBE.get_or_init(probe_systemd_scope)
}

fn next_scope_unit() -> String {
    let serial = NEXT_SCOPE_SERIAL.fetch_add(1, Ordering::Relaxed);
    format!("malvin-agent-{}-{serial}", std::process::id())
}

fn pending_slot() -> std::sync::MutexGuard<'static, Option<String>> {
    PENDING_SCOPE_UNIT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[must_use]
pub fn malvin_agent_tokio_command(
    program: impl AsRef<OsStr>,
    work_dir: &Path,
) -> tokio::process::Command {
    if !agent_cgroups_enabled() {
        *pending_slot() = None;
        return super::malvin_sandbox::malvin_tokio_command(program);
    }
    let unit = next_scope_unit();
    let limit = crate::config::mem_limit_config::load_mem_limit_bytes(work_dir);
    let mut cmd = super::malvin_sandbox::malvin_std_command(SYSTEMD_RUN);
    cmd.args(scope_args(&unit, limit)).arg(program);
    *pending_slot() = Some(unit);
    tokio::process::Command::from(cmd)
}

#[must_use]
pub(crate) fn cgroup_dir_is_unit_scope(dir: &Path, unit: &str) -> bool {
    dir.file_name()
        .and_then(OsStr::to_str)
        .and_then(|name| name.strip_suffix(".scope"))
        .is_some_and(|name| name == unit)
}

fn wait_for_scope_cgroup(pid: u32, unit: &str) -> Option<AgentCgroup> {
    let start = Instant::now();
    loop {
        let dir = pid_cgroup_dir(pid)?;
        if cgroup_dir_is_unit_scope(&dir, unit) {
            return Some(AgentCgroup::new(pid, dir));
        }
        if start.elapsed() >= ATTACH_WAIT {
            return None;
        }
        std::thread::sleep(ATTACH_POLL);
    }
}

pub fn attach_pending_agent_scope(agent_pid: Option<u32>) {
    let unit = pending_slot().take();
    let (Some(unit), Some(pid)) = (unit, agent_pid) else {
        set_active_agent_cgroup(None);
        return;
    };
    let cgroup = wait_for_scope_cgroup(pid, &unit);
    if cgroup.is_none() {
        warn!(
            unit,
            pid, "agent did not enter its systemd scope; using process-group containment"
        );
    }
    set_active_agent_cgroup(cgroup);
}

#[cfg(test)]
#[path = "agent_scope_tests.rs"]
mod agent_scope_tests;
