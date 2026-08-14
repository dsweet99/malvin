
use crate::acp::AgentError;

use crate::bridge_protocol::{decode_event, encode_request, BridgeEvent, BridgeRequest};
use super::session::BridgeSession;
use super::timing::{note_sdk_step, record_sdk_usage};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

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
    let no_force = (!session.io.force).then_some("fail_fast");
    let req = BridgeRequest::Create {
        cwd: args.cwd.display().to_string(),
        model: args.model.to_string(),
        api_key: args.api_key,
        no_force_policy: no_force,
        models_json_path: args.models_json_path.map(str::to_string),
    };
    write_request(session, &req).await?;
    wait_for_ok(session).await
}

pub(crate) async fn send_resume(
    session: &BridgeSession,
    args: ResumeArgs<'_>,
) -> Result<(), AgentError> {
    let no_force = (!session.io.force).then_some("fail_fast");
    let req = BridgeRequest::Resume {
        agent_id: args.agent_id.to_string(),
        cwd: args.cwd.display().to_string(),
        model: args.model.to_string(),
        api_key: args.api_key,
        no_force_policy: no_force,
    };
    write_request(session, &req).await?;
    wait_for_ok(session).await
}

pub async fn write_request(
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

pub(crate) async fn read_event(session: &BridgeSession) -> Result<BridgeEvent, AgentError> {
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

async fn wait_for_ok(session: &BridgeSession) -> Result<(), AgentError> {
    loop {
        match read_event_with_idle_timeout(session, "ok").await? {
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

pub(crate) async fn drain_until_run_done(session: &BridgeSession) -> Result<(), AgentError> {
    use super::log_adapter::handle_stream_event;
    loop {
        let ev = read_event_with_idle_timeout(session, "run_done").await?;
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

async fn read_event_with_idle_timeout(
    session: &BridgeSession,
    waiting_for: &str,
) -> Result<BridgeEvent, AgentError> {
    let idle = crate::sdk_drain_timeout::sdk_drain_idle_timeout_from_env();
    tokio::time::timeout(idle, read_event(session))
        .await
        .unwrap_or_else(|_| {
            Err(AgentError(format!(
                "bridge timed out waiting for {waiting_for} after {idle:?} of silence"
            )))
        })
}

async fn discard_optional_trailing_run_done(session: &BridgeSession) {
    let read = read_event(session);
    let timed = tokio::time::timeout(std::time::Duration::from_millis(50), read).await;
    match timed {
        Ok(Ok(BridgeEvent::RunDone { .. })) => {}
        _ => {}
    }
}

#[must_use]
pub(crate) fn run_done_status_is_failure(status: &str) -> bool {
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
        record_sdk_usage(session.timing.as_ref(), u, session.normalize_pi_usage);
    }
    *session
        .last_response
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) =
        result.clone().unwrap_or_default();
    if let Some(text) = result {
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

pub(crate) fn start_mem_watch(session: &BridgeSession) {
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
