use crate::agent_process::AgentError;

use super::bridge_sdk::TurnWait;
use super::bridge_sdk::session::BridgeSession;
use super::bridge_sdk::session_handshake::wait_for_ok;
use super::bridge_sdk::timing::{note_sdk_step, record_sdk_usage};
use crate::backends::cursor_sdk::protocol::{
    BridgeEvent, BridgeRequest, decode_event, encode_request,
};
use crate::config::model_id::ModelBackend;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

const BRIDGE: ModelBackend = ModelBackend::Cursor;

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

pub async fn write_request(session: &BridgeSession, req: &BridgeRequest) -> Result<(), AgentError> {
    let line = encode_request(req).map_err(AgentError)?;
    let label = BRIDGE.wire_label();
    let mut stdin = session.stdio.stdin.lock().await;
    stdin
        .write_all(format!("{line}\n").as_bytes())
        .await
        .map_err(|e| AgentError::session_dead(format!("{label} write: {e}")))?;
    stdin
        .flush()
        .await
        .map_err(|e| AgentError::session_dead(format!("{label} flush: {e}")))?;
    drop(stdin);
    Ok(())
}

pub(crate) async fn read_event(session: &BridgeSession) -> Result<BridgeEvent, AgentError> {
    let label = BRIDGE.wire_label();
    let mut line = String::new();
    let n = {
        let mut stdout = session.stdio.stdout.lock().await;
        stdout
            .read_line(&mut line)
            .await
            .map_err(|e| AgentError::session_dead(format!("{label} read: {e}")))?
    };
    if n == 0 {
        return Err(AgentError::session_dead(format!("{label} stdout closed")));
    }
    decode_event(line.trim()).map_err(AgentError)
}

pub(crate) async fn drain_until_run_done(session: &BridgeSession) -> Result<(), AgentError> {
    use super::bridge_sdk::log_adapter::handle_stream_event;
    let mut turn = super::bridge_sdk::DrainIdleTurn::new();
    let mut last_usage: Option<serde_json::Value> = None;
    loop {
        let ev = read_event_with_idle_timeout(session, "run_done", &mut turn).await?;
        match &ev {
            BridgeEvent::Step { .. } => note_sdk_step(session.timing.as_ref()),
            BridgeEvent::Usage { usage } => {
                last_usage = Some(usage.clone());
                handle_stream_event(session, &ev);
            }
            BridgeEvent::RunDone { .. } => {
                return finish_run_done(session, &ev, last_usage.as_ref());
            }
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
    turn: &mut super::bridge_sdk::DrainIdleTurn,
) -> Result<BridgeEvent, AgentError> {
    let labels = super::bridge_sdk::DrainIdleLabels {
        prefix: crate::config::model_id::ModelBackend::Cursor.drain_idle_prefix(),
        waiting_for,
    };
    super::bridge_sdk::await_turn_event(TurnWait::of(session), labels, read_event(session), turn)
        .await
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
pub(crate) const fn run_done_status_is_failure(
    status: crate::backends::cursor_sdk::protocol::RunDoneStatus,
) -> bool {
    status.is_failure()
}

fn finish_run_done(
    session: &BridgeSession,
    ev: &BridgeEvent,
    last_usage: Option<&serde_json::Value>,
) -> Result<(), AgentError> {
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
    let effective_usage = match usage {
        Some(v) if crate::run_timing::acp_usage_payload_is_observable(v) => Some(v),
        _ => last_usage,
    };
    if let Some(u) = effective_usage {
        record_sdk_usage(session.timing.as_ref(), u);
    }
    *session
        .last_response
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = result.clone().unwrap_or_default();
    if let Some(text) = result {
        super::bridge_sdk::log_adapter::feed_do_dm_run_result(text);
    }
    super::bridge_sdk::log_adapter::handle_stream_event(session, ev);
    if *status == crate::backends::cursor_sdk::protocol::RunDoneStatus::Unknown {
        tracing::warn!(
            result = result.as_deref(),
            error = error.as_deref(),
            "run_done unknown status; surfacing result/error"
        );
    }
    if run_done_status_is_failure(*status) {
        return Err(AgentError(error.clone().unwrap_or_else(|| match *status {
            crate::backends::cursor_sdk::protocol::RunDoneStatus::Cancelled => {
                "run cancelled".into()
            }
            crate::backends::cursor_sdk::protocol::RunDoneStatus::Unknown => {
                "run finished with unknown status".into()
            }
            _ => "run error".into(),
        })));
    }
    Ok(())
}

pub(crate) struct MemWatchArgs<'a> {
    pub process_group_id: Option<u32>,
    pub reader_dead: &'a std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub work_dir: &'a std::path::Path,
    pub spawn_pid_baseline: &'a std::collections::HashSet<u32>,
    pub run_dir: Option<&'a std::path::Path>,
}

pub(crate) fn start_mem_watch(args: MemWatchArgs<'_>) {
    #[cfg(unix)]
    {
        if crate::agent_process::test_no_real_agent_enabled() {
            return;
        }
        let Some(pgid) = args.process_group_id else {
            return;
        };
        let handles = crate::agent_process::MemWatchHandles {
            reader_dead: std::sync::Arc::clone(args.reader_dead),
            pgid: Some(pgid),
            limit_bytes: crate::config::mem_limit_config::load_mem_limit_bytes(args.work_dir),
            spawn_pid_baseline: args.spawn_pid_baseline.clone(),
            run_dir: args.run_dir.map(std::path::Path::to_path_buf),
        };
        tokio::spawn(async move {
            crate::agent_process::watch_process_group_memory(handles).await;
        });
    }
    #[cfg(not(unix))]
    {
        let _ = args;
    }
}

#[cfg(test)]
mod tests {
    use super::run_done_status_is_failure;

    #[test]
    fn cancelled_and_error_are_failures() {
        use crate::backends::cursor_sdk::protocol::RunDoneStatus;
        assert!(run_done_status_is_failure(RunDoneStatus::Error));
        assert!(run_done_status_is_failure(RunDoneStatus::Cancelled));
        assert!(run_done_status_is_failure(RunDoneStatus::Unknown));
        assert!(!run_done_status_is_failure(RunDoneStatus::Finished));
    }
}
