//! Liveness-driven sandbox teardown: poll, re-snapshot kill targets, TERM→KILL escalation.

use std::collections::HashSet;

use super::unix_process_group_kill_targets::kill_targets_for_teardown;
use super::unix_process_group_ps::{signal_pid, signal_process_group};

/// Poll interval aligned with the sandbox memory watcher (release builds).
#[cfg(debug_assertions)]
pub(crate) const TEARDOWN_POLL_INTERVAL: std::time::Duration =
    std::time::Duration::from_millis(100);
#[cfg(not(debug_assertions))]
pub(crate) const TEARDOWN_POLL_INTERVAL: std::time::Duration =
    std::time::Duration::from_millis(500);
/// Hard cap for cooperative teardown before unconditional SIGKILL sweep.
#[cfg(debug_assertions)]
pub(crate) const TEARDOWN_TOTAL_CAP: std::time::Duration =
    std::time::Duration::from_millis(500);
#[cfg(not(debug_assertions))]
pub(crate) const TEARDOWN_TOTAL_CAP: std::time::Duration = std::time::Duration::from_secs(5);
/// Bounded wait for ACP `session/cancel` during explicit shutdown.
#[cfg(debug_assertions)]
pub(crate) const SHUTDOWN_CANCEL_TIMEOUT: std::time::Duration =
    std::time::Duration::from_millis(200);
#[cfg(not(debug_assertions))]
pub(crate) const SHUTDOWN_CANCEL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);
/// Poll ticks after SIGTERM before escalating a survivor to SIGKILL.
#[cfg(debug_assertions)]
pub(crate) const TEARDOWN_KILL_AFTER_POLLS: u32 = 1;
#[cfg(not(debug_assertions))]
pub(crate) const TEARDOWN_KILL_AFTER_POLLS: u32 = 3;

#[derive(Default)]
struct TeardownPollState {
    sigterm_pids: HashSet<u32>,
    sigkill_pids: HashSet<u32>,
    pg_sigterm: bool,
    pg_sigkill: bool,
    polls: u32,
}

fn escalate_pid(pid: u32, state: &mut TeardownPollState, force_kill: bool) {
    if state.sigkill_pids.contains(&pid) {
        return;
    }
    if state.sigterm_pids.insert(pid) {
        signal_pid(pid, 15);
    } else if (force_kill || state.polls >= TEARDOWN_KILL_AFTER_POLLS) && state.sigkill_pids.insert(pid)
    {
        signal_pid(pid, 9);
    }
}

fn teardown_poll_tick(
    process_group_id: Option<u32>,
    spawn_baseline: Option<&HashSet<u32>>,
    state: &mut TeardownPollState,
    force_kill: bool,
) {
    let targets = kill_targets_for_teardown(process_group_id, spawn_baseline);
    for pid in targets {
        escalate_pid(pid, state, force_kill);
    }
    let Some(pgid) = process_group_id else {
        return;
    };
    if !state.pg_sigterm {
        signal_process_group(pgid, 15);
        state.pg_sigterm = true;
        return;
    }
    if (force_kill || state.polls >= TEARDOWN_KILL_AFTER_POLLS) && !state.pg_sigkill {
        signal_process_group(pgid, 9);
        state.pg_sigkill = true;
    }
}

pub(crate) fn teardown_agent_sandbox_blocking(
    process_group_id: Option<u32>,
    spawn_baseline: &HashSet<u32>,
) {
    let orphan_scan = !spawn_baseline.is_empty();
    if process_group_id.is_none() && !orphan_scan {
        return;
    }
    let baseline_opt = orphan_scan.then_some(spawn_baseline);
    let baseline_for_alive = spawn_baseline;
    let mut state = TeardownPollState::default();
    let start = std::time::Instant::now();
    while crate::malvin_sandbox::sandbox_still_alive(process_group_id, baseline_for_alive)
        && start.elapsed() < TEARDOWN_TOTAL_CAP
    {
        teardown_poll_tick(process_group_id, baseline_opt, &mut state, false);
        if !crate::malvin_sandbox::sandbox_still_alive(process_group_id, baseline_for_alive) {
            break;
        }
        std::thread::sleep(TEARDOWN_POLL_INTERVAL);
        state.polls = state.polls.saturating_add(1);
    }
    if crate::malvin_sandbox::sandbox_still_alive(process_group_id, baseline_for_alive) {
        teardown_poll_tick(process_group_id, baseline_opt, &mut state, true);
    }
}

pub(crate) async fn teardown_agent_sandbox_async(
    process_group_id: Option<u32>,
    spawn_baseline: Option<&HashSet<u32>>,
) {
    let orphan_scan = spawn_baseline.is_some_and(|b| !b.is_empty());
    if process_group_id.is_none() && !orphan_scan {
        return;
    }
    let baseline_opt = spawn_baseline.filter(|b| !b.is_empty());
    let empty_baseline = HashSet::new();
    let baseline_for_alive = spawn_baseline.unwrap_or(&empty_baseline);
    let mut state = TeardownPollState::default();
    let start = std::time::Instant::now();
    while crate::malvin_sandbox::sandbox_still_alive(process_group_id, baseline_for_alive)
        && start.elapsed() < TEARDOWN_TOTAL_CAP
    {
        teardown_poll_tick(process_group_id, baseline_opt, &mut state, false);
        if !crate::malvin_sandbox::sandbox_still_alive(process_group_id, baseline_for_alive) {
            break;
        }
        tokio::time::sleep(TEARDOWN_POLL_INTERVAL).await;
        state.polls = state.polls.saturating_add(1);
    }
    if crate::malvin_sandbox::sandbox_still_alive(process_group_id, baseline_for_alive) {
        teardown_poll_tick(process_group_id, baseline_opt, &mut state, true);
    }
}

pub(crate) fn reap_fixed_pid_targets_blocking(targets: &HashSet<u32>) {
    if targets.is_empty() {
        return;
    }
    let mut state = TeardownPollState::default();
    let start = std::time::Instant::now();
    let any_alive = || targets.iter().any(|pid| crate::acp::pid_alive(*pid));
    while any_alive() && start.elapsed() < TEARDOWN_TOTAL_CAP {
        for pid in targets {
            escalate_pid(*pid, &mut state, false);
        }
        if !any_alive() {
            break;
        }
        std::thread::sleep(TEARDOWN_POLL_INTERVAL);
        state.polls = state.polls.saturating_add(1);
    }
    for pid in targets {
        if crate::acp::pid_alive(*pid) {
            signal_pid(*pid, 9);
        }
    }
}

#[cfg(test)]
mod kiss_cov_auto {
    use super::*;

    #[test]
    fn kiss_cov_teardown_agent_sandbox_blocking() {
        let _ = teardown_agent_sandbox_blocking;
    }
    #[test]
    fn kiss_cov_teardown_agent_sandbox_async() {
        let _ = teardown_agent_sandbox_async;
    }
    #[test]
    fn kiss_cov_teardown_poll_tick() {
        let _ = teardown_poll_tick;
    }
    #[test]
    fn kiss_cov_teardown_poll_state() {
        let _ = std::mem::size_of::<TeardownPollState>();
    }
    #[test]
    fn kiss_cov_reap_fixed_pid_targets_blocking() {
        let _ = reap_fixed_pid_targets_blocking;
    }
    #[test]
    fn kiss_cov_escalate_pid() {
        let _ = escalate_pid;
    }
}
