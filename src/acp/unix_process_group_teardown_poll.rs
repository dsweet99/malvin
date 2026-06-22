//! Liveness-driven sandbox teardown: poll, re-snapshot kill targets, TERM→KILL escalation.

use std::collections::HashSet;

#[path = "unix_process_group_teardown_timing.rs"]
mod unix_process_group_teardown_timing;

use super::unix_process_group_kill_targets::kill_targets_for_teardown;
use super::unix_process_group_ps::{signal_pid, signal_process_group};
use unix_process_group_teardown_timing::{
    teardown_kill_after_polls, teardown_poll_interval, teardown_total_cap,
    test_fast_acp_teardown_enabled,
};

pub(crate) use unix_process_group_teardown_timing::shutdown_cancel_timeout;

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
    } else if (force_kill || state.polls >= teardown_kill_after_polls())
        && state.sigkill_pids.insert(pid)
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
    if (force_kill || state.polls >= teardown_kill_after_polls()) && !state.pg_sigkill {
        signal_process_group(pgid, 9);
        state.pg_sigkill = true;
    }
}

fn teardown_agent_sandbox_fast_tick(
    process_group_id: Option<u32>,
    baseline_opt: Option<&HashSet<u32>>,
) {
    let mut state = TeardownPollState::default();
    teardown_poll_tick(process_group_id, baseline_opt, &mut state, true);
}

fn teardown_agent_sandbox_slow_blocking(
    process_group_id: Option<u32>,
    baseline_opt: Option<&HashSet<u32>>,
    baseline_for_alive: &HashSet<u32>,
) {
    let mut state = TeardownPollState::default();
    let start = std::time::Instant::now();
    while crate::malvin_sandbox::sandbox_still_alive(process_group_id, baseline_for_alive)
        && start.elapsed() < teardown_total_cap()
    {
        teardown_poll_tick(process_group_id, baseline_opt, &mut state, false);
        if !crate::malvin_sandbox::sandbox_still_alive(process_group_id, baseline_for_alive) {
            break;
        }
        std::thread::sleep(teardown_poll_interval());
        state.polls = state.polls.saturating_add(1);
    }
    if crate::malvin_sandbox::sandbox_still_alive(process_group_id, baseline_for_alive) {
        teardown_poll_tick(process_group_id, baseline_opt, &mut state, true);
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
    if test_fast_acp_teardown_enabled() {
        teardown_agent_sandbox_fast_tick(process_group_id, baseline_opt);
        return;
    }
    teardown_agent_sandbox_slow_blocking(process_group_id, baseline_opt, spawn_baseline);
}

async fn teardown_agent_sandbox_slow_async(
    process_group_id: Option<u32>,
    baseline_opt: Option<&HashSet<u32>>,
    baseline_for_alive: &HashSet<u32>,
) {
    let baseline_for_alive = baseline_for_alive.clone();
    let baseline_owned = baseline_opt.cloned();
    tokio::task::spawn_blocking(move || {
        teardown_agent_sandbox_slow_blocking(
            process_group_id,
            baseline_owned.as_ref(),
            &baseline_for_alive,
        );
    })
    .await
    .ok();
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
    if test_fast_acp_teardown_enabled() {
        teardown_agent_sandbox_fast_tick(process_group_id, baseline_opt);
        return;
    }
    let empty_baseline = HashSet::new();
    let baseline_for_alive = spawn_baseline.unwrap_or(&empty_baseline);
    teardown_agent_sandbox_slow_async(process_group_id, baseline_opt, baseline_for_alive).await;
}

pub(crate) fn reap_fixed_pid_targets_blocking(targets: &HashSet<u32>) {
    if targets.is_empty() {
        return;
    }
    let mut state = TeardownPollState::default();
    let start = std::time::Instant::now();
    let any_alive = || targets.iter().any(|pid| crate::acp::pid_alive(*pid));
    while any_alive() && start.elapsed() < teardown_total_cap() {
        for pid in targets {
            escalate_pid(*pid, &mut state, false);
        }
        if !any_alive() {
            break;
        }
        std::thread::sleep(teardown_poll_interval());
        state.polls = state.polls.saturating_add(1);
    }
    for pid in targets {
        if crate::acp::pid_alive(*pid) {
            signal_pid(*pid, 9);
        }
    }
}
