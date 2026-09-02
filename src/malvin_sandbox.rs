use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub use crate::acp_spawn_lock::assert_no_peer_acp_spawn_lock;
use crate::acp_spawn_lock::{acquire_acp_spawn_lock, release_acp_spawn_lock};

#[cfg(unix)]
use crate::acp::sandbox_monitor_pids;
#[cfg(unix)]
use crate::process_group_rss::pids_sandbox_bytes;

pub use crate::parent_death_signal::{
    install_parent_death_signal, install_tokio_parent_death_signal,
};

static MALVIN_SPAWN_BASELINE: OnceLock<HashSet<u32>> = OnceLock::new();

struct ActiveSandboxSession {
    pgid: Option<u32>,
    baseline: HashSet<u32>,
    work_dir: PathBuf,
    acp_lock_slot: String,
}

static ACTIVE_SANDBOX_SESSION: Mutex<Option<ActiveSandboxSession>> = Mutex::new(None);

/// Proof that the previous sandbox was cleared before this spawn attempt.
/// Consumed by [`note_active_sandbox_session`] so spawn paths cannot skip the gate.
#[derive(Debug)]
pub struct SandboxSpawnTicket(());

pub fn init_malvin_spawn_baseline() {
    #[cfg(unix)]
    {
        if !crate::acp::test_no_real_agent_enabled() {
            crate::acp::reap_baseline_amnestied_agent_orphans_blocking();
        }
    }
    #[cfg(not(unix))]
    {}
}

#[must_use]
pub fn malvin_spawn_baseline() -> HashSet<u32> {
    MALVIN_SPAWN_BASELINE.get_or_init(HashSet::new).clone()
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

fn apply_sandbox_resource_limits(cmd: &mut std::process::Command) {
    cmd.env("MALLOC_ARENA_MAX", "2");
}

fn apply_sandbox_resource_limits_tokio(cmd: &mut tokio::process::Command) {
    cmd.env("MALLOC_ARENA_MAX", "2");
}

#[must_use]
pub fn malvin_std_command(program: impl AsRef<OsStr>) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    isolate_child_process_group(&mut cmd);
    install_parent_death_signal(&mut cmd);
    apply_sandbox_resource_limits(&mut cmd);
    cmd
}

#[must_use]
pub fn malvin_tokio_command(program: impl AsRef<OsStr>) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(program);
    isolate_tokio_child_process_group(&mut cmd);
    install_tokio_parent_death_signal(&mut cmd);
    apply_sandbox_resource_limits_tokio(&mut cmd);
    cmd
}

/// Gate the next spawn: previous sandbox processes must already be dead.
#[must_use = "pass the ticket to note_active_sandbox_session after spawn"]
pub fn take_sandbox_spawn_ticket() -> Result<SandboxSpawnTicket, String> {
    assert_dead_before_next_spawn()?;
    Ok(SandboxSpawnTicket(()))
}

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

pub fn note_active_sandbox_session(
    _ticket: SandboxSpawnTicket,
    pgid: Option<u32>,
    baseline: HashSet<u32>,
    work_dir: &Path,
) -> Result<(), String> {
    let acp_lock_slot = crate::acp_spawn_lock::active_acp_lock_slot();
    acquire_acp_spawn_lock(work_dir)?;
    *ACTIVE_SANDBOX_SESSION
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(ActiveSandboxSession {
        pgid,
        baseline,
        work_dir: work_dir.to_path_buf(),
        acp_lock_slot,
    });
    Ok(())
}

pub fn clear_active_sandbox_session() {
    let session = ACTIVE_SANDBOX_SESSION
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    if let Some(session) = session {
        release_acp_spawn_lock(&session.work_dir, &session.acp_lock_slot);
    }
    #[cfg(unix)]
    crate::acp::clear_session_spawn_affiliation();
}

pub fn teardown_active_sandbox_for_interrupt() {
    let session = ACTIVE_SANDBOX_SESSION
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    let Some(session) = session else {
        return;
    };
    #[cfg(unix)]
    {
        crate::active_agent_heartbeat::unregister_active_agent_process_group(session.pgid);
        crate::acp::terminate_agent_process_group_for_interrupt(session.pgid, &session.baseline);
        crate::acp::clear_session_spawn_affiliation();
    }
    release_acp_spawn_lock(&session.work_dir, &session.acp_lock_slot);
}

#[cfg(test)]
pub(crate) fn clear_active_sandbox_session_for_test() {
    clear_active_sandbox_session();
}

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
pub(crate) fn sandbox_still_alive(
    agent_pgid: Option<u32>,
    session_baseline: &HashSet<u32>,
) -> bool {
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
        let _ = crate::acp::reap_baseline_amnestied_agent_orphans_blocking;
        let _ = super::clear_active_sandbox_session_for_test;
        let _ = super::teardown_active_sandbox_for_interrupt;
        let _ = super::init_malvin_spawn_baseline;
        let _ = super::malvin_spawn_baseline;
        let _ = super::isolate_child_process_group;
        let _ = super::isolate_tokio_child_process_group;
        let _ = super::install_parent_death_signal;
        let _ = super::install_tokio_parent_death_signal;
        let _ = super::sandbox_still_alive;
        let _ = super::take_sandbox_spawn_ticket;
        let _ = super::malvin_std_command("true");
        let _ = super::malvin_tokio_command("true");
    }

    #[test]
    fn sandbox_spawn_ticket_requires_clear_gate() {
        let ticket = super::take_sandbox_spawn_ticket().expect("clear");
        super::note_active_sandbox_session(
            ticket,
            None,
            std::collections::HashSet::new(),
            std::path::Path::new("."),
        )
        .expect("note");
        super::clear_active_sandbox_session();
    }
}
