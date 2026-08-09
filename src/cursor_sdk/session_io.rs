//! Bridge stdin/stdout protocol helpers.

use crate::acp::AgentError;

use super::auth::effective_sdk_api_key;
use super::protocol::{decode_event, encode_request, BridgeEvent, BridgeRequest};
use super::session::BridgeSession;
use super::timing::{note_sdk_step, record_sdk_usage};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

pub(super) async fn send_create(
    session: &BridgeSession,
    cwd: &std::path::Path,
    model: &str,
) -> Result<(), AgentError> {
    let no_force = (!session.io.force).then_some("fail_fast");
    let req = BridgeRequest::Create {
        cwd: cwd.display().to_string(),
        model: model.to_string(),
        api_key: effective_sdk_api_key(),
        no_force_policy: no_force,
    };
    write_request(session, &req).await?;
    wait_for_ok(session).await
}

pub(super) async fn send_resume(
    session: &BridgeSession,
    agent_id: &str,
    cwd: &std::path::Path,
    model: &str,
) -> Result<(), AgentError> {
    let no_force = (!session.io.force).then_some("fail_fast");
    let req = BridgeRequest::Resume {
        agent_id: agent_id.to_string(),
        cwd: cwd.display().to_string(),
        model: model.to_string(),
        api_key: effective_sdk_api_key(),
        no_force_policy: no_force,
    };
    write_request(session, &req).await?;
    wait_for_ok(session).await
}

pub(super) async fn write_request(
    session: &BridgeSession,
    req: &BridgeRequest,
) -> Result<(), AgentError> {
    let line = encode_request(req).map_err(AgentError)?;
    let mut stdin = session.stdin.lock().await;
    stdin
        .write_all(format!("{line}\n").as_bytes())
        .await
        .map_err(|e| AgentError(format!("bridge write: {e}")))?;
    stdin
        .flush()
        .await
        .map_err(|e| AgentError(format!("bridge flush: {e}")))?;
    drop(stdin);
    Ok(())
}

pub(super) async fn read_event(session: &BridgeSession) -> Result<BridgeEvent, AgentError> {
    let mut line = String::new();
    let n = {
        let mut stdout = session.stdout.lock().await;
        stdout
            .read_line(&mut line)
            .await
            .map_err(|e| AgentError(format!("bridge read: {e}")))?
    };
    if n == 0 {
        return Err(AgentError("bridge stdout closed".into()));
    }
    decode_event(line.trim()).map_err(AgentError)
}

pub(super) async fn wait_for_ok(session: &BridgeSession) -> Result<(), AgentError> {
    loop {
        match read_event(session).await? {
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

pub(super) async fn drain_until_run_done(session: &BridgeSession) -> Result<(), AgentError> {
    use super::log_adapter::handle_stream_event;
    loop {
        let ev = read_event_with_drain_idle_timeout(session).await?;
        match &ev {
            BridgeEvent::Step { .. } => note_sdk_step(session.timing.as_ref()),
            BridgeEvent::RunDone { .. } => return finish_run_done(session, &ev),
            BridgeEvent::Fatal { message, .. } => {
                discard_optional_trailing_run_done(session).await;
                return Err(AgentError(message.clone()));
            }
            _ => handle_stream_event(session, &ev),
        }
    }
}

/// Fail the turn if the bridge stays silent too long (never emits `run_done` / `fatal`).
async fn read_event_with_drain_idle_timeout(
    session: &BridgeSession,
) -> Result<BridgeEvent, AgentError> {
    let idle = crate::sdk_drain_timeout::sdk_drain_idle_timeout_from_env();
    tokio::time::timeout(idle, read_event(session))
        .await
        .unwrap_or_else(|_| {
            Err(AgentError(format!(
                "bridge drain timed out waiting for run_done after {idle:?} of silence"
            )))
        })
}

/// Legacy bridges sometimes emitted `fatal` then `run_done`. Consume a trailing
/// `run_done` if it is already buffered so the next prompt does not see it.
async fn discard_optional_trailing_run_done(session: &BridgeSession) {
    let read = read_event(session);
    let timed = tokio::time::timeout(std::time::Duration::from_millis(50), read).await;
    match timed {
        Ok(Ok(BridgeEvent::RunDone { .. })) => {}
        _ => {}
    }
}

/// `RunResultStatus` values that must not be treated as a successful turn.
#[must_use]
pub(super) fn run_done_status_is_failure(status: &str) -> bool {
    status == "error" || status == "cancelled"
}

fn finish_run_done(session: &BridgeSession, ev: &BridgeEvent) -> Result<(), AgentError> {
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
        record_sdk_usage(session.timing.as_ref(), u);
    }
    if let Some(text) = result {
        *session
            .last_response
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = text.clone();
        // Authoritative final text: SDK often places DM fences only here.
        super::log_adapter::feed_do_dm_run_result(text);
    }
    super::log_adapter::handle_stream_event(session, ev);
    if run_done_status_is_failure(status) {
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

pub(super) fn start_mem_watch(session: &BridgeSession) {
    #[cfg(unix)]
    {
        if crate::acp::test_no_real_agent_enabled() {
            return;
        }
        let Some(pgid) = session.process_group_id else {
            return;
        };
        let handles = crate::acp::MemWatchHandles {
            reader_dead: std::sync::Arc::clone(&session.reader_dead),
            pgid,
            limit_bytes: crate::mem_limit_config::load_mem_limit_bytes(&session.work_dir),
            spawn_pid_baseline: session.spawn_pid_baseline.clone(),
            run_dir: session.run_dir.clone(),
        };
        tokio::spawn(async move {
            crate::acp::watch_process_group_memory(handles).await;
        });
    }
    #[cfg(not(unix))]
    {
        let _ = session;
    }
}

#[cfg(test)]
mod tests {
    use super::run_done_status_is_failure;

    #[test]
    fn cancelled_and_error_are_failures() {
        assert!(run_done_status_is_failure("error"));
        assert!(run_done_status_is_failure("cancelled"));
        assert!(!run_done_status_is_failure("finished"));
    }
}
