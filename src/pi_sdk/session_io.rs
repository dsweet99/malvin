//! Pi RPC stdin/stdout against a shared [`BridgeSession`].

use std::sync::atomic::{AtomicU64, Ordering};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::acp::AgentError;
use crate::bridge_protocol::BridgeEvent;
use crate::bridge_sdk::{note_sdk_step, record_sdk_usage, BridgeSession};

use super::map_event::map_pi_event;
use super::protocol::{
    abort_request, pi_decode_line, pi_encode_request, new_session_request, prompt_request, PiLine,
};

static PI_REQ_SEQ: AtomicU64 = AtomicU64::new(1);

fn pi_next_req_id() -> String {
    format!("malvin-{}", PI_REQ_SEQ.fetch_add(1, Ordering::Relaxed))
}

pub(crate) async fn pi_write_line(session: &BridgeSession, line: &str) -> Result<(), AgentError> {
    let mut stdin = session.stdin.lock().await;
    stdin
        .write_all(format!("{line}\n").as_bytes())
        .await
        .map_err(|e| AgentError(format!("pi rpc write: {e}")))?;
    stdin
        .flush()
        .await
        .map_err(|e| AgentError(format!("pi rpc flush: {e}")))?;
    drop(stdin);
    Ok(())
}

pub(crate) async fn pi_write_abort(session: &BridgeSession) -> Result<(), AgentError> {
    let req = abort_request(pi_next_req_id());
    pi_write_line(session, &pi_encode_request(&req)).await
}

pub(crate) async fn pi_send_new_session(session: &BridgeSession) -> Result<(), AgentError> {
    let id = pi_next_req_id();
    let req = new_session_request(&id);
    pi_write_line(session, &pi_encode_request(&req)).await?;
    pi_wait_for_response(session, &id).await
}

pub(crate) async fn pi_send_prompt(session: &BridgeSession, prompt: &str) -> Result<(), AgentError> {
    let id = pi_next_req_id();
    let req = prompt_request(&id, prompt);
    pi_write_line(session, &pi_encode_request(&req)).await?;
    pi_wait_for_response(session, &id).await?;
    pi_drain_until_run_done(session).await
}

async fn pi_wait_for_response(session: &BridgeSession, id: &str) -> Result<(), AgentError> {
    loop {
        match pi_read_line(session).await? {
            PiLine::Response {
                id: rid,
                success,
                error,
                ..
            } if rid == id => {
                if success {
                    return Ok(());
                }
                return Err(AgentError(
                    error.unwrap_or_else(|| "pi rpc command failed".into()),
                ));
            }
            PiLine::Response { .. } => {}
            PiLine::Event { type_name, raw } => {
                for ev in map_pi_event(&type_name, &raw) {
                    if let BridgeEvent::Fatal { message, .. } = ev {
                        return Err(AgentError(message));
                    }
                }
            }
        }
    }
}

async fn pi_read_line(session: &BridgeSession) -> Result<PiLine, AgentError> {
    let mut line = String::new();
    let n = {
        let mut stdout = session.stdout.lock().await;
        stdout
            .read_line(&mut line)
            .await
            .map_err(|e| AgentError(format!("pi rpc read: {e}")))?
    };
    if n == 0 {
        return Err(AgentError("pi rpc stdout closed".into()));
    }
    pi_decode_line(line.trim()).map_err(AgentError)
}

async fn pi_drain_until_run_done(session: &BridgeSession) -> Result<(), AgentError> {
    loop {
        let line = pi_read_line_with_idle_timeout(session).await?;
        match line {
            PiLine::Response { success, error, .. } => {
                if !success {
                    return Err(AgentError(
                        error.unwrap_or_else(|| "pi rpc command failed".into()),
                    ));
                }
            }
            PiLine::Event { type_name, raw } => {
                for ev in map_pi_event(&type_name, &raw) {
                    match &ev {
                        BridgeEvent::Step { .. } => note_sdk_step(session.timing.as_ref()),
                        BridgeEvent::RunDone { .. } => return pi_finish_run_done(session, &ev),
                        BridgeEvent::Fatal { message, .. } => {
                            return Err(AgentError(message.clone()));
                        }
                        _ => crate::bridge_sdk::handle_stream_event(session, &ev),
                    }
                }
            }
        }
    }
}

async fn pi_read_line_with_idle_timeout(session: &BridgeSession) -> Result<PiLine, AgentError> {
    let idle = crate::sdk_drain_timeout::sdk_drain_idle_timeout_from_env();
    tokio::time::timeout(idle, pi_read_line(session))
        .await
        .unwrap_or_else(|_| {
            Err(AgentError(format!(
                "pi rpc drain timed out waiting for agent_end after {idle:?} of silence"
            )))
        })
}

fn pi_finish_run_done(session: &BridgeSession, ev: &BridgeEvent) -> Result<(), AgentError> {
    let BridgeEvent::RunDone {
        status,
        result,
        usage,
        error,
        ..
    } = ev
    else {
        return Ok(());
    };
    if let Some(u) = usage {
        record_sdk_usage(session.timing.as_ref(), u, session.normalize_prime_usage);
    }
    if let Some(text) = result {
        *session
            .last_response
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = text.clone();
        crate::bridge_sdk::feed_do_dm_run_result(text);
    }
    crate::bridge_sdk::handle_stream_event(session, ev);
    if crate::bridge_sdk::run_done_status_is_failure(status) {
        return Err(AgentError(error.clone().unwrap_or_else(|| {
            if status == "cancelled" {
                "run cancelled".into()
            } else {
                "run error".into()
            }
        })));
    }
    Ok(())
}
