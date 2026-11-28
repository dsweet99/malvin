use tokio::time::Instant;

use crate::agent_process::AgentError;
use crate::backends::cursor_sdk::protocol::{BridgeEvent, BridgeRequest};
use crate::sdk_drain_timeout::sdk_bridge_startup_timeout;

use super::bridge_sdk::session::BridgeSession;
use super::bridge_sdk::session_io::{read_event, write_request};

async fn read_event_with_timeout(
    session: &BridgeSession,
    waiting_for: &str,
    timeout: std::time::Duration,
) -> Result<BridgeEvent, AgentError> {
    tokio::time::timeout(timeout, read_event(session))
        .await
        .unwrap_or_else(|_| {
            Err(AgentError(format!(
                "{} waiting for {waiting_for} after {timeout:?} of silence",
                crate::config::model_id::ModelBackend::Cursor.drain_idle_prefix()
            )))
        })
}

async fn wait_for_ok(session: &BridgeSession) -> Result<(), AgentError> {
    let deadline = Instant::now() + sdk_bridge_startup_timeout();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(AgentError(format!(
                "{} waiting for ok after {:?} (startup handshake deadline)",
                crate::config::model_id::ModelBackend::Cursor.drain_idle_prefix(),
                sdk_bridge_startup_timeout()
            )));
        }
        match read_event_with_timeout(session, "ok", remaining).await? {
            BridgeEvent::Ok { agent_id } => {
                if let Some(id) = agent_id {
                    *session
                        .agent_id
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(id);
                }
                return Ok(());
            }
            BridgeEvent::Fatal { message, .. } => return Err(AgentError(message)),
            _ => {}
        }
    }
}

pub(crate) struct CreateArgs<'a> {
    pub cwd: &'a std::path::Path,
    pub model: &'a str,
    pub api_key: Option<String>,
    pub models_json_path: Option<&'a str>,
}

pub(crate) struct ResumeArgs<'a> {
    pub agent_id: &'a str,
    pub cwd: &'a std::path::Path,
    pub model: &'a str,
    pub api_key: Option<String>,
}

pub(crate) async fn send_create(
    session: &BridgeSession,
    args: CreateArgs<'_>,
) -> Result<(), AgentError> {
    let req = BridgeRequest::Create {
        cwd: args.cwd.display().to_string(),
        model: args.model.to_string(),
        api_key: args.api_key,
        models_json_path: args.models_json_path.map(str::to_string),
    };
    write_request(session, &req).await?;
    wait_for_ok(session).await
}

pub(crate) async fn send_resume(
    session: &BridgeSession,
    args: ResumeArgs<'_>,
) -> Result<(), AgentError> {
    let req = BridgeRequest::Resume {
        agent_id: args.agent_id.to_string(),
        cwd: args.cwd.display().to_string(),
        model: args.model.to_string(),
        api_key: args.api_key,
    };
    write_request(session, &req).await?;
    wait_for_ok(session).await
}
