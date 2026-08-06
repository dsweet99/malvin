//! Bridge stdin/stdout protocol helpers (create/send/cancel/close; no resume).

use crate::acp::AgentError;

use super::protocol::{prime_decode_event, prime_encode_request, PrimeBridgeEvent, PrimeBridgeRequest};
use super::session::PrimeBridgeSession;
use super::timing::{prime_note_sdk_step, prime_record_sdk_usage};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

pub(super) async fn prime_send_create(
    session: &PrimeBridgeSession,
    cwd: &std::path::Path,
    model: &str,
) -> Result<(), AgentError> {
    let no_force = (!session.io.force).then_some("fail_fast");
    let req = PrimeBridgeRequest::Create {
        cwd: cwd.display().to_string(),
        model: model.to_string(),
        // Never forward Cursor credentials; bridge uses Prime AuthStorage + provider env.
        api_key: None,
        no_force_policy: no_force,
    };
    prime_write_request(session, &req).await?;
    prime_wait_for_ok(session).await
}

pub(super) async fn prime_write_request(
    session: &PrimeBridgeSession,
    req: &PrimeBridgeRequest,
) -> Result<(), AgentError> {
    let line = prime_encode_request(req).map_err(AgentError)?;
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

pub(super) async fn prime_read_event(session: &PrimeBridgeSession) -> Result<PrimeBridgeEvent, AgentError> {
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
    prime_decode_event(line.trim()).map_err(AgentError)
}

pub(super) async fn prime_wait_for_ok(session: &PrimeBridgeSession) -> Result<(), AgentError> {
    loop {
        match prime_read_event(session).await? {
            PrimeBridgeEvent::Ok { agent_id } => {
                if let Some(id) = agent_id {
                    *session
                        .agent_id
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(id);
                }
                return Ok(());
            }
            PrimeBridgeEvent::Fatal { message, .. } => return Err(AgentError(message)),
            _ => {}
        }
    }
}

pub(super) async fn prime_drain_until_run_done(session: &PrimeBridgeSession) -> Result<(), AgentError> {
    use super::log_adapter::prime_handle_stream_event;
    loop {
        let ev = prime_read_event(session).await?;
        match &ev {
            PrimeBridgeEvent::Step { .. } => prime_note_sdk_step(session.timing.as_ref()),
            PrimeBridgeEvent::RunDone { .. } => return prime_finish_run_done(session, &ev),
            PrimeBridgeEvent::Fatal { message, .. } => {
                prime_discard_optional_trailing_run_done(session).await;
                return Err(AgentError(message.clone()));
            }
            _ => prime_handle_stream_event(session, &ev),
        }
    }
}

async fn prime_discard_optional_trailing_run_done(session: &PrimeBridgeSession) {
    let read = prime_read_event(session);
    let timed = tokio::time::timeout(std::time::Duration::from_millis(50), read).await;
    if let Ok(Ok(PrimeBridgeEvent::RunDone { .. })) = timed {
        // discarded
    }
}

#[must_use]
pub(super) fn prime_run_done_status_is_failure(status: &str) -> bool {
    status == "error" || status == "cancelled"
}

fn prime_finish_run_done(session: &PrimeBridgeSession, ev: &PrimeBridgeEvent) -> Result<(), AgentError> {
    let PrimeBridgeEvent::RunDone {
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
        prime_record_sdk_usage(session.timing.as_ref(), u);
    }
    if let Some(text) = result {
        *session
            .last_response
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = text.clone();
        super::log_adapter::prime_feed_do_dm_run_result(text);
    }
    super::log_adapter::prime_handle_stream_event(session, ev);
    if prime_run_done_status_is_failure(status) {
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

pub(super) fn prime_start_mem_watch(session: &PrimeBridgeSession) {
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
