use std::collections::HashSet;
use std::time::Duration;

use crate::child_health::{
    SilenceHealthOutcome, evaluate_after_acp_silence, silence_grace_for_rpc_timeout,
};

use super::{DrainHealthVerdict, DrainIdleHealthCtx};

pub(crate) async fn sample_drain_health(
    ctx: DrainIdleHealthCtx<'_>,
    slice: Duration,
) -> DrainHealthVerdict {
    let pids = drain_sample_pids(ctx.process_group_id, ctx.spawn_pid_baseline).await;
    if pids.is_empty() {
        return DrainHealthVerdict::DeadOrZombie;
    }
    aggregate_pid_health(&pids, silence_grace_for_rpc_timeout(slice)).await
}

pub(crate) async fn drain_sample_pids(
    pgid: Option<u32>,
    spawn_pid_baseline: &HashSet<u32>,
) -> Vec<u32> {
    let baseline = spawn_pid_baseline.clone();
    tokio::task::spawn_blocking(move || drain_sample_pids_blocking(pgid, &baseline))
        .await
        .unwrap_or_else(|_| pgid.map_or_else(Vec::new, |id| vec![id]))
}

fn drain_sample_pids_blocking(pgid: Option<u32>, spawn_pid_baseline: &HashSet<u32>) -> Vec<u32> {
    #[cfg(unix)]
    {
        let mut pids: Vec<u32> = crate::acp::sandbox_monitor_pids(pgid, spawn_pid_baseline)
            .into_iter()
            .collect();
        if pids.is_empty()
            && let Some(id) = pgid
        {
            pids.push(id);
        }
        pids.sort_unstable();
        pids.dedup();
        pids
    }
    #[cfg(not(unix))]
    {
        let _ = spawn_pid_baseline;
        pgid.map_or_else(Vec::new, |id| vec![id])
    }
}

pub(crate) async fn aggregate_pid_health(pids: &[u32], grace: Duration) -> DrainHealthVerdict {
    let mut outcomes = Vec::with_capacity(pids.len());
    for &pid in pids {
        outcomes.push(evaluate_after_acp_silence(pid, grace).await);
    }
    aggregate_health_outcomes(&outcomes)
}

pub(crate) fn aggregate_health_outcomes(outcomes: &[SilenceHealthOutcome]) -> DrainHealthVerdict {
    let mut saw_busy = false;
    let mut saw_alive = false;
    for outcome in outcomes {
        match outcome {
            SilenceHealthOutcome::StillBusyExtendWait => saw_busy = true,
            SilenceHealthOutcome::AppearsHung => saw_alive = true,
            SilenceHealthOutcome::ChildNotRunning | SilenceHealthOutcome::ChildZombie => {}
        }
    }
    if saw_busy {
        DrainHealthVerdict::StillBusy
    } else if saw_alive {
        DrainHealthVerdict::AppearsHung
    } else {
        DrainHealthVerdict::DeadOrZombie
    }
}
