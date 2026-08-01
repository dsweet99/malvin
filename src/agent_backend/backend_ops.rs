//! Free functions for [`super::backend::AgentBackend`] operations kept out of the enum impl for kiss limits.

use std::sync::{Arc, Mutex};

use crate::acp::{AgentError, AgentKpopMultiturnCtl, KpopFlowOnceArgs};

use super::backend::AgentBackend;

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

pub(crate) const fn agent_max_retries(backend: &AgentBackend) -> u32 {
    backend.max_acp_retries()
}

pub(crate) fn agent_timing_opt(
    backend: &AgentBackend,
) -> Option<&Arc<Mutex<crate::run_timing::RunTiming>>> {
    agent_backend_timing(backend)
}

/// Generic K-Pop flow over the shared [`AgentBackend`] surface (no Mini-only branches).
pub async fn agent_backend_run_kpop_flow(
    client: &mut AgentBackend,
    flow: &KpopFlowOnceArgs<'_>,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    // ACP retains its specialized spawn-per-attempt path for cursor-agent parity.
    if let AgentBackend::Acp(c) = client {
        return crate::acp::AgentClient::run_kpop_flow(c, flow, session_dotfile_backups).await;
    }
    super::backend_kpop_ops::run_kpop_flow_via_agent(client, flow, session_dotfile_backups).await
}

pub async fn agent_backend_run_kpop_multiturn(
    client: &mut AgentBackend,
    ctl: AgentKpopMultiturnCtl<'_, '_>,
) -> Result<(), AgentError> {
    if let AgentBackend::Acp(c) = client {
        return c.run_kpop_multiturn(ctl).await;
    }
    super::backend_kpop_ops::run_kpop_multiturn_via_agent(client, ctl).await
}

