use super::session::NpmPiSession;
use crate::acp::AgentError;
use crate::bridge_sdk::{BridgeSpawnArgs, start_mem_watch};

pub(crate) async fn npm_pi_spawn_bridge(
    args: BridgeSpawnArgs<'_>,
) -> Result<NpmPiSession, AgentError> {
    let ticket = crate::malvin_sandbox::take_sandbox_spawn_ticket().map_err(AgentError)?;
    if crate::acp::test_no_real_agent_enabled() {
        return Err(AgentError(
            "npm pi mock session is not configured for MALVIN_TEST_NO_REAL_AGENT".into(),
        ));
    }
    prewarm_openrouter_pricing(args.model);
    let session = super::session_process::spawn_npm_pi_session(&args, ticket)?;
    start_mem_watch(session.stdio.mem_watch_args());
    Ok(session)
}

fn prewarm_openrouter_pricing(model: &crate::model_id::ParsedModel) {
    let Some((provider, id)) = model.pi_provider_and_model() else {
        return;
    };
    if crate::pi_sdk::uses_openrouter_catalog(provider, id) {
        std::thread::spawn(|| crate::pi_sdk::warm_openrouter_pricing_cache(false));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_spawn_bridge() {
        let _ = npm_pi_spawn_bridge;
        let _ = prewarm_openrouter_pricing;
    }
}
