//! Free functions for [`super::backend::AgentBackend`] operations.

use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::acp::AgentError;

use super::backend::AgentBackend;

pub async fn agent_backend_ensure_coder_session(
    backend: &mut AgentBackend,
    cwd: &Path,
) -> Result<(), AgentError> {
    match backend {
        AgentBackend::CursorSdk(c) => c.ensure_coder_session(cwd).await,
        AgentBackend::PrimeSdk(c) => c.ensure_coder_session(cwd).await,
        AgentBackend::Acp(c) => {
            if c.has_open_coder_session() {
                Ok(())
            } else {
                c.begin_coder_session(cwd).await
            }
        }
    }
}

pub fn agent_backend_set_run_timing(
    backend: &mut AgentBackend,
    timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
) {
    match backend {
        AgentBackend::Acp(c) => c.set_run_timing(timing),
        AgentBackend::CursorSdk(c) => c.set_run_timing(timing),
        AgentBackend::PrimeSdk(c) => c.set_run_timing(timing),
    }
}

#[must_use]
pub fn agent_backend_attach_run_timing_for_session(
    backend: &mut AgentBackend,
) -> Arc<Mutex<crate::run_timing::RunTiming>> {
    match backend {
        AgentBackend::Acp(c) => c.attach_run_timing_for_session(),
        AgentBackend::CursorSdk(c) => c.attach_run_timing_for_session(),
        AgentBackend::PrimeSdk(c) => c.attach_run_timing_for_session(),
    }
}

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
        AgentBackend::CursorSdk(c) => c.timing.as_ref(),
        AgentBackend::PrimeSdk(c) => c.timing.as_ref(),
    }
}
