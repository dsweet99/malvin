use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pi::sdk::AgentEvent;
use tokio::sync::mpsc;

use crate::acp::AgentError;
use crate::bridge_protocol::BridgeEvent;
use crate::bridge_sdk::{
    DrainIdleHealthCtx, DrainIdleLabels, StreamLog, note_sdk_step, record_sdk_usage,
    run_done_status_is_failure,
};

use super::map_agent_event::map_pi_agent_event;
use super::runtime::PiRuntime;
use super::session_fake::fake_events_for_prompt;

pub(crate) struct PiEmbeddedSession {
    pub(crate) runtime: Option<PiRuntime>,
    pub(crate) log: StreamLog,
    pub(crate) work_dir: PathBuf,
    pub(crate) reader_dead: Arc<AtomicBool>,
    pub(crate) spawn_pid_baseline: HashSet<u32>,
}

impl PiEmbeddedSession {
    pub(crate) async fn send_prompt(&self, prompt: &str) -> Result<(), AgentError> {
        if self.runtime.is_none() {
            return send_fake_prompt(self, prompt).await;
        }
        let (events_tx, events_rx) = mpsc::unbounded_channel();
        let runtime = self
            .runtime
            .as_ref()
            .ok_or_else(|| AgentError("pi session runtime missing".into()))?;
        let reply = runtime
            .prompt(prompt.to_string(), events_tx)
            .map_err(AgentError)?;
        drain_agent_events(self, events_rx, reply).await
    }

    pub(crate) async fn shutdown(mut self) -> Result<(), AgentError> {
        self.reader_dead.store(true, Ordering::SeqCst);
        if let Some(mut runtime) = self.runtime.take() {
            runtime.abort();
            runtime.shutdown();
        }
        #[cfg(unix)]
        {
            crate::acp::terminate_agent_process_group(None, &self.spawn_pid_baseline).await;
        }
        crate::malvin_sandbox::clear_active_sandbox_session();
        Ok(())
    }
}

impl Drop for PiEmbeddedSession {
    fn drop(&mut self) {
        self.reader_dead.store(true, Ordering::SeqCst);
        if let Some(mut runtime) = self.runtime.take() {
            runtime.abort();
            runtime.shutdown();
        }
        #[cfg(unix)]
        {
            crate::acp::terminate_agent_process_group_for_interrupt(
                None,
                &self.spawn_pid_baseline,
            );
        }
        crate::malvin_sandbox::clear_active_sandbox_session();
    }
}

async fn drain_agent_events(
    session: &PiEmbeddedSession,
    mut events_rx: mpsc::UnboundedReceiver<AgentEvent>,
    reply: tokio::sync::oneshot::Receiver<Result<(), String>>,
) -> Result<(), AgentError> {
    tokio::pin!(reply);
    let mut prompt_result: Option<Result<(), String>> = None;
    loop {
        if session.reader_dead.load(Ordering::SeqCst) {
            session.runtime.as_ref().inspect(|runtime| runtime.abort());
            return Err(AgentError(
                "pi session aborted (memory limit or shutdown)".into(),
            ));
        }
        // Invariant: `reply` is polled only through the first select arm. A
        // tokio oneshot receiver is consumed by any winning poll, so
        // re-awaiting it elsewhere would panic ("called after complete");
        // instead the outcome is cached in `prompt_result` and reused by the
        // exit paths. `biased` + reply-first makes that ordering deterministic.
        let next = recv_event_with_idle(session, &mut events_rx);
        tokio::select! {
            biased;
            done = &mut reply, if prompt_result.is_none() => {
                prompt_result = Some(done.unwrap_or_else(|_| Err("pi sdk runtime stopped".into())));
                if events_rx.is_empty() {
                    return finish_after_channel_closed(
                        prompt_result
                            .take()
                            .unwrap_or_else(|| Err("pi sdk runtime stopped".into())),
                    );
                }
            }
            event = next => {
                match event {
                    Ok(Some(event)) => {
                        if handle_mapped_events(session, &event)? {
                            return finish_after_channel_closed(
                                prompt_result.take().unwrap_or_else(|| {
                                    Err("pi sdk runtime stopped".into())
                                }),
                            );
                        }
                    }
                    Ok(None) => {
                        // Reply not yet cached means the first arm never won;
                        // the receiver was therefore never polled, so awaiting
                        // it here is safe (exactly once).
                        let result = match prompt_result.take() {
                            Some(result) => result,
                            None => reply
                                .await
                                .unwrap_or_else(|_| Err("pi sdk runtime stopped".into())),
                        };
                        return finish_after_channel_closed(result);
                    }
                    Err(err) => {
                        session
                            .runtime
                            .as_ref()
                            .inspect(|runtime| runtime.abort());
                        return Err(err);
                    }
                }
            }
        }
    }
}

async fn recv_event_with_idle(
    session: &PiEmbeddedSession,
    events_rx: &mut mpsc::UnboundedReceiver<AgentEvent>,
) -> Result<Option<AgentEvent>, AgentError> {
    let labels = DrainIdleLabels {
        prefix: crate::acp::DRAIN_IDLE_PREFIX_PI,
        waiting_for: "agent_end",
    };
    let health = Some(DrainIdleHealthCtx {
        process_group_id: None,
        spawn_pid_baseline: &session.spawn_pid_baseline,
    });
    crate::bridge_sdk::await_next_with_idle(labels, health, async { Ok(events_rx.recv().await) })
        .await
}

fn handle_mapped_events(
    session: &PiEmbeddedSession,
    event: &AgentEvent,
) -> Result<bool, AgentError> {
    let mut done = false;
    for ev in map_pi_agent_event(event) {
        match &ev {
            BridgeEvent::Step { .. } => note_sdk_step(session.log.timing.as_ref()),
            BridgeEvent::RunDone { .. } => {
                finish_run_done(&session.log, &ev)?;
                done = true;
            }
            BridgeEvent::Fatal { message, .. } => return Err(AgentError(message.clone())),
            _ => crate::bridge_sdk::handle_stream_event(&session.log, &ev),
        }
    }
    Ok(done)
}

pub(crate) fn finish_after_channel_closed(prompt_result: Result<(), String>) -> Result<(), AgentError> {
    prompt_result.map_err(AgentError)
}

pub(crate) fn finish_run_done(log: &StreamLog, ev: &BridgeEvent) -> Result<(), AgentError> {
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
        record_sdk_usage(log.timing.as_ref(), u);
    }
    *log.last_response
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = result.clone().unwrap_or_default();
    if let Some(text) = result {
        crate::bridge_sdk::feed_do_dm_run_result(text);
    }
    crate::bridge_sdk::handle_stream_event(log, ev);
    if run_done_status_is_failure(*status) {
        return Err(AgentError(error.clone().unwrap_or_else(|| {
            if *status == crate::bridge_protocol::RunDoneStatus::Cancelled {
                "run cancelled".into()
            } else {
                "run error".into()
            }
        })));
    }
    Ok(())
}

async fn send_fake_prompt(session: &PiEmbeddedSession, prompt: &str) -> Result<(), AgentError> {
    let (tx, rx) = mpsc::unbounded_channel();
    for event in fake_events_for_prompt(prompt) {
        let _ = tx.send(event);
    }
    drop(tx);
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    let _ = reply_tx.send(Ok(()));
    drain_agent_events(session, rx, reply_rx).await
}
