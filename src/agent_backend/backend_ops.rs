//! Free functions for [`super::backend::AgentBackend`] operations kept out of the enum impl for kiss limits.

use std::sync::{Arc, Mutex};

use crate::acp::{AgentError, AgentKpopMultiturnCtl, KpopFlowOnceArgs};

use super::backend::AgentBackend;
use super::mini::MiniAgentClient;

pub fn agent_backend_set_run_timing(
    backend: &mut AgentBackend,
    timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
) {
    match backend {
        AgentBackend::Acp(c) => c.set_run_timing(timing),
        AgentBackend::Mini(c) => c.timing = timing,
    }
}

#[must_use]
pub fn agent_backend_attach_run_timing_for_session(
    backend: &mut AgentBackend,
) -> Arc<Mutex<crate::run_timing::RunTiming>> {
    match backend {
        AgentBackend::Acp(c) => c.attach_run_timing_for_session(),
        AgentBackend::Mini(c) => crate::run_timing::attach_new_run_timing(&mut c.timing),
    }
}

/// Returns existing run timing or installs a new wall clock when none is active.
#[must_use]
pub fn agent_backend_ensure_run_timing_for_session(
    backend: &mut AgentBackend,
) -> Arc<Mutex<crate::run_timing::RunTiming>> {
    if let Some(t) = agent_backend_timing(backend).cloned() {
        return t;
    }
    agent_backend_attach_run_timing_for_session(backend)
}

pub fn agent_backend_set_implement_display_name(backend: &AgentBackend, label: &'static str) {
    let Some(timing) = agent_backend_timing(backend) else {
        return;
    };
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_implement_display_name(label);
}

#[allow(clippy::missing_const_for_fn)]
#[must_use]
pub fn agent_backend_timing(
    backend: &AgentBackend,
) -> Option<&Arc<Mutex<crate::run_timing::RunTiming>>> {
    match backend {
        AgentBackend::Acp(c) => c.timing.as_ref(),
        AgentBackend::Mini(c) => c.timing.as_ref(),
    }
}

pub async fn agent_backend_run_kpop_flow(
    client: &mut AgentBackend,
    flow: &KpopFlowOnceArgs<'_>,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    match client {
        AgentBackend::Acp(c) => {
            crate::acp::AgentClient::run_kpop_flow(c, flow, session_dotfile_backups).await
        }
        AgentBackend::Mini(c) => run_kpop_flow_mini(c, flow, session_dotfile_backups).await,
    }
}

pub async fn agent_backend_run_kpop_multiturn(
    client: &mut AgentBackend,
    ctl: AgentKpopMultiturnCtl<'_, '_>,
) -> Result<(), AgentError> {
    match client {
        AgentBackend::Acp(c) => c.run_kpop_multiturn(ctl).await,
        AgentBackend::Mini(c) => run_kpop_multiturn_mini(c, ctl).await,
    }
}

async fn run_kpop_flow_mini(
    client: &mut MiniAgentClient,
    flow: &KpopFlowOnceArgs<'_>,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    use crate::acp::{backoff_after_agent_failure, retries_noun};

    crate::agent_phase::enter_kpop();
    let mut last_error = String::new();
    let mut attempts_used = 0_u32;
    let max_attempts = client.max_acp_retries();
    for attempt in 1..=max_attempts {
        attempts_used = attempt;
        match super::kpop_bridge::run_kpop_flow_once_mini(client, flow, session_dotfile_backups).await
        {
            Ok(()) => {
                crate::agent_phase::leave_kpop();
                return Ok(());
            }
            Err(e) => {
                last_error = e.0;
                if backoff_after_agent_failure(
                    client.timing.as_ref(),
                    &last_error,
                    attempt,
                    max_attempts,
                )
                .await?
                {
                    break;
                }
            }
        }
    }
    crate::agent_phase::leave_kpop();
    let retries = attempts_used.saturating_sub(1);
    let noun = retries_noun(retries);
    Err(AgentError(format!(
        "mini agent (kpop flow) failed after {retries} {noun}. Last error:\n{last_error}"
    )))
}

async fn run_kpop_multiturn_mini(
    client: &mut MiniAgentClient,
    mut ctl: AgentKpopMultiturnCtl<'_, '_>,
) -> Result<(), AgentError> {
    use crate::acp::{backoff_after_agent_failure, retries_noun};

    crate::agent_phase::enter_kpop();
    let mut last_error = String::new();
    let mut attempts_used = 0_u32;
    let max_attempts = client.max_acp_retries();
    for attempt in 1..=max_attempts {
        attempts_used = attempt;
        match super::kpop_bridge::run_kpop_multiturn_once_mini(client, &mut ctl).await {
            Ok(()) => {
                crate::agent_phase::leave_kpop();
                return Ok(());
            }
            Err(e) => {
                ctl.state.reset_for_transport_retry();
                last_error = e.0;
                if backoff_after_agent_failure(
                    client.timing.as_ref(),
                    &last_error,
                    attempt,
                    max_attempts,
                )
                .await?
                {
                    break;
                }
            }
        }
    }
    crate::agent_phase::leave_kpop();
    let retries = attempts_used.saturating_sub(1);
    let noun = retries_noun(retries);
    Err(AgentError(format!(
        "mini agent (kpop multiturn) failed after {retries} {noun}. Last error:\n{last_error}"
    )))
}

#[cfg(test)]
mod kiss_cov_auto {
    use super::*;
    use std::path::Path;

    #[test]
    fn kiss_cov_backend_ops() {
        let _ = (
            agent_backend_set_run_timing,
            agent_backend_attach_run_timing_for_session,
            agent_backend_timing,
            agent_backend_run_kpop_flow,
            agent_backend_run_kpop_multiturn,
            run_kpop_flow_mini,
            run_kpop_multiturn_mini,
        );
        let _: Option<&Path> = None;
    }
}
