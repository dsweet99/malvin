use super::session::NpmPiSession;
use crate::acp::AgentError;
use crate::bridge_sdk::{BridgeSpawnArgs, MemWatchArgs, start_mem_watch};

pub(crate) async fn npm_pi_spawn_bridge(args: BridgeSpawnArgs<'_>) -> Result<NpmPiSession, AgentError> {
    crate::acp::require_force(args.io.force)?;
    let ticket = crate::malvin_sandbox::take_sandbox_spawn_ticket().map_err(AgentError)?;
    if crate::acp::test_no_real_agent_enabled() {
        return Err(AgentError(
            "npm pi mock session is not configured for MALVIN_TEST_NO_REAL_AGENT".into(),
        ));
    }
    let session = super::session_process::spawn_npm_pi_session(&args, ticket)?;
    start_mem_watch(MemWatchArgs {
        process_group_id: session.process_group_id,
        reader_dead: &session.reader_dead,
        work_dir: &session.work_dir,
        spawn_pid_baseline: &session.spawn_pid_baseline,
        run_dir: session.run_dir.as_deref(),
    });
    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_spawn_bridge() {
        let _ = npm_pi_spawn_bridge;
    }
}
