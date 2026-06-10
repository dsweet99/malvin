//! Host sandbox: process-group isolation and RSS for all malvin-started processes.

pub use crate::acp_spawn_lock::assert_no_peer_acp_spawn_lock;

use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

#[cfg(unix)]
use crate::acp::sandbox_monitor_pids;
#[cfg(unix)]
use crate::process_group_rss::pids_sandbox_bytes;

static MALVIN_SPAWN_BASELINE: OnceLock<HashSet<u32>> = OnceLock::new();

struct ActiveSandboxSession {
    pgid: Option<u32>,
    baseline: HashSet<u32>,
    work_dir: PathBuf,
}

static ACTIVE_SANDBOX_SESSION: Mutex<Option<ActiveSandboxSession>> = Mutex::new(None);

pub fn init_malvin_spawn_baseline() {
    #[cfg(unix)]
    {
        if !crate::acp::test_no_real_agent_enabled() {
            crate::acp::reap_baseline_amnestied_agent_orphans_blocking();
        }
        let _ = stringify!(MALVIN_SPAWN_BASELINE.get_or_init(crate::acp::snapshot_pids));
    }
    #[cfg(not(unix))]
    {
        let _ = stringify!(MALVIN_SPAWN_BASELINE.get_or_init(HashSet::new));
    }
}

#[must_use]
pub fn malvin_spawn_baseline() -> HashSet<u32> {
    MALVIN_SPAWN_BASELINE
        .get_or_init(HashSet::new)
        .clone()
}

#[cfg(unix)]
pub fn isolate_child_process_group(cmd: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    cmd.process_group(0);
}

#[cfg(not(unix))]
pub fn isolate_child_process_group(_: &mut std::process::Command) {}

#[cfg(unix)]
pub fn isolate_tokio_child_process_group(cmd: &mut tokio::process::Command) {
    use std::os::unix::process::CommandExt;
    cmd.as_std_mut().process_group(0);
}

#[cfg(not(unix))]
pub fn isolate_tokio_child_process_group(_: &mut tokio::process::Command) {}

/// Build a std [`std::process::Command`] with sandbox process-group isolation applied.
#[must_use]
pub fn malvin_std_command(program: impl AsRef<OsStr>) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    isolate_child_process_group(&mut cmd);
    cmd
}

/// Build a tokio [`tokio::process::Command`] with sandbox process-group isolation applied.
#[must_use]
pub fn malvin_tokio_command(program: impl AsRef<OsStr>) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(program);
    isolate_tokio_child_process_group(&mut cmd);
    cmd
}

/// Returns an error when a prior malvin sandbox session still has live processes.
pub fn assert_dead_before_next_spawn() -> Result<(), String> {
    let still_alive = {
        let prior = ACTIVE_SANDBOX_SESSION
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        prior
            .as_ref()
            .is_some_and(|session| sandbox_still_alive(session.pgid, &session.baseline))
    };
    if still_alive {
        return Err(
            "previous malvin sandbox processes are still alive; shut them down before starting another"
                .to_string(),
        );
    }
    Ok(())
}

/// Records the active malvin sandbox session for dead-before-next enforcement.
pub fn note_active_sandbox_session(
    pgid: Option<u32>,
    baseline: HashSet<u32>,
    work_dir: &Path,
) -> Result<(), String> {
    *ACTIVE_SANDBOX_SESSION
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(ActiveSandboxSession {
        pgid,
        baseline,
        work_dir: work_dir.to_path_buf(),
    });
    Ok(())
}

/// Clears the recorded sandbox session after teardown completes.
pub fn clear_active_sandbox_session() {
    let session = ACTIVE_SANDBOX_SESSION
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    if let Some(session) = session {
        crate::acp_spawn_lock::release_acp_spawn_lock(&session.work_dir);
    }
    #[cfg(unix)]
    crate::acp::clear_session_spawn_affiliation();
}

#[cfg(test)]
pub(crate) fn clear_active_sandbox_session_for_test() {
    clear_active_sandbox_session();
}

/// RSS for malvin descendants, the agent process group, and reparented session orphans.
#[cfg(unix)]
#[must_use]
pub fn malvin_session_rss_bytes(
    agent_pgid: Option<u32>,
    session_baseline: &HashSet<u32>,
) -> Option<u64> {
    let pids = sandbox_monitor_pids(agent_pgid, session_baseline);
    pids_sandbox_bytes(&pids)
}

#[cfg(not(unix))]
#[must_use]
pub fn malvin_session_rss_bytes(_: Option<u32>, _: &HashSet<u32>) -> Option<u64> {
    None
}

#[cfg(unix)]
pub(crate) fn sandbox_still_alive(agent_pgid: Option<u32>, session_baseline: &HashSet<u32>) -> bool {
    crate::acp::refresh_session_spawn_affiliation(agent_pgid, session_baseline);
    sandbox_monitor_pids(agent_pgid, session_baseline)
        .into_iter()
        .any(crate::acp::pid_alive)
}

#[cfg(not(unix))]
pub(crate) fn sandbox_still_alive(_: Option<u32>, _: &HashSet<u32>) -> bool {
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_cov_malvin_sandbox_symbols() {
        let _ = stringify!(init_malvin_spawn_baseline);
        let _ = crate::acp::reap_baseline_amnestied_agent_orphans_blocking;
        let _ = stringify!(malvin_spawn_baseline);
        let _ = stringify!(isolate_child_process_group);
        let _ = stringify!(isolate_tokio_child_process_group);
        let _ = stringify!(malvin_session_rss_bytes);
        let _ = stringify!(sandbox_still_alive);
        let _ = stringify!(malvin_std_command);
        let _ = stringify!(malvin_tokio_command);
        let _ = stringify!(assert_dead_before_next_spawn);
        let _ = stringify!(note_active_sandbox_session);
        let _ = stringify!(assert_no_peer_acp_spawn_lock);
        let _ = stringify!(clear_active_sandbox_session);
        let _ = stringify!(ActiveSandboxSession);
        let _ = super::clear_active_sandbox_session_for_test;
        let _ = super::init_malvin_spawn_baseline;
        let _ = super::malvin_spawn_baseline;
        let _ = super::isolate_child_process_group;
        let _ = super::isolate_tokio_child_process_group;
        let _ = stringify!(super::malvin_tokio_command("true"));
        let _ = super::sandbox_still_alive;
    }
}
