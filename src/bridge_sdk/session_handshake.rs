use tokio::time::Instant;

use crate::acp::AgentError;
use crate::bridge_protocol::BridgeEvent;
use crate::sdk_drain_timeout::sdk_bridge_startup_timeout;

use super::session::BridgeSession;
use super::session_io::read_event;

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
                crate::acp::DRAIN_IDLE_PREFIX_BRIDGE
            )))
        })
}

pub(super) async fn wait_for_ok(session: &BridgeSession) -> Result<(), AgentError> {
    let deadline = Instant::now() + sdk_bridge_startup_timeout();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(AgentError(format!(
                "{} waiting for ok after {:?} (startup handshake deadline)",
                crate::acp::DRAIN_IDLE_PREFIX_BRIDGE,
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
