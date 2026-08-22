use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use pi::sdk::{AbortHandle, AgentEvent, AgentSessionHandle, SessionOptions};

enum PiCmd {
    Prompt(PromptCmd),
    Shutdown,
}

struct PromptCmd {
    text: String,
    events: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
    reply: tokio::sync::oneshot::Sender<Result<(), String>>,
}

pub(crate) struct PiRuntime {
    cmd_tx: std::sync::mpsc::Sender<PiCmd>,
    abort: Arc<Mutex<Option<AbortHandle>>>,
    thread: Option<JoinHandle<()>>,
}

impl PiRuntime {
    pub(crate) fn start(options: SessionOptions) -> Result<Self, String> {
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let abort = Arc::new(Mutex::new(None));
        let abort_for_thread = Arc::clone(&abort);
        let thread = std::thread::Builder::new()
            .name("malvin-pi-sdk".into())
            .spawn(move || run_pi_thread(options, cmd_rx, ready_tx, abort_for_thread))
            .map_err(|e| format!("pi sdk thread: {e}"))?;
        ready_rx
            .recv()
            .map_err(|_| "pi sdk thread ended before ready".to_string())??;
        Ok(Self {
            cmd_tx,
            abort,
            thread: Some(thread),
        })
    }

    pub(crate) fn prompt(
        &self,
        text: String,
        events: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
    ) -> Result<tokio::sync::oneshot::Receiver<Result<(), String>>, String> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.cmd_tx
            .send(PiCmd::Prompt(PromptCmd {
                text,
                events,
                reply: reply_tx,
            }))
            .map_err(|_| "pi sdk runtime stopped".to_string())?;
        Ok(reply_rx)
    }

    pub(crate) fn abort(&self) {
        if let Some(handle) = self
            .abort
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            handle.abort();
        }
    }

    pub(crate) fn shutdown(&mut self) {
        let _ = self.cmd_tx.send(PiCmd::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for PiRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn run_pi_thread(
    options: SessionOptions,
    cmd_rx: std::sync::mpsc::Receiver<PiCmd>,
    ready_tx: std::sync::mpsc::Sender<Result<(), String>>,
    abort: Arc<Mutex<Option<AbortHandle>>>,
) {
    let runtime = match asupersync::runtime::RuntimeBuilder::current_thread().build() {
        Ok(runtime) => runtime,
        Err(e) => {
            let _ = ready_tx.send(Err(format!("asupersync runtime: {e}")));
            return;
        }
    };
    match runtime.block_on(pi::sdk::create_agent_session(options)) {
        Ok(handle) => {
            let _ = ready_tx.send(Ok(()));
            serve_session(runtime, handle, cmd_rx, abort);
        }
        Err(e) => {
            let _ = ready_tx.send(Err(format!("pi create_agent_session: {e}")));
        }
    }
}

fn serve_session(
    runtime: asupersync::runtime::Runtime,
    mut handle: AgentSessionHandle,
    cmd_rx: std::sync::mpsc::Receiver<PiCmd>,
    abort: Arc<Mutex<Option<AbortHandle>>>,
) {
    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            PiCmd::Shutdown => break,
            PiCmd::Prompt(prompt) => run_prompt(&runtime, &mut handle, prompt, &abort),
        }
    }
}

fn run_prompt(
    runtime: &asupersync::runtime::Runtime,
    handle: &mut AgentSessionHandle,
    prompt: PromptCmd,
    abort: &Arc<Mutex<Option<AbortHandle>>>,
) {
    let (abort_handle, signal) = AgentSessionHandle::new_abort_handle();
    *abort
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(abort_handle);
    let events = prompt.events;
    let result = runtime.block_on(handle.prompt_with_abort(prompt.text, signal, move |event| {
        let _ = events.send(event);
    }));
    *abort
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    let _ = prompt
        .reply
        .send(result.map(|_| ()).map_err(|e| e.to_string()));
}
