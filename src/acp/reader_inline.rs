// Stdout JSON-RPC line processing (inlined into `acp` for `kiss` dependency depth).
use tokio::sync::Notify;

/// Clears busy / trace when the JSON-RPC response for a `session/prompt` request is processed.
pub(crate) struct PromptRpcCleanup {
    pub busy: Arc<AtomicBool>,
    pub trace_writer: Arc<Mutex<Option<PromptTraceWriter>>>,
    pub prompt_rpc_id: Arc<AtomicU64>,
    /// When set (UI lane only), [`Self::clear_if_prompt_response`] notifies waiters when `busy` clears.
    pub idle_notify: Option<Arc<Notify>>,
}

impl PromptRpcCleanup {
    pub async fn clear_if_prompt_response(&self, id: u64) {
        let expected = self.prompt_rpc_id.load(Ordering::SeqCst);
        if expected != 0 && expected == id {
            self.prompt_rpc_id.store(0, Ordering::SeqCst);
            self.busy.store(false, Ordering::SeqCst);
            *self.trace_writer.lock().await = None;
            if let Some(n) = &self.idle_notify {
                n.notify_waiters();
            }
        }
    }
}

// Single stdout include (no `reader::stdout` submodule) for `kiss` dependency depth.
include!("reader_stdout_body.inc");

#[cfg(test)]
mod reader_inline_unit_tests {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Arc;

    use super::PromptRpcCleanup;

    #[tokio::test]
    async fn clear_if_prompt_response_clears_busy() {
        let busy = Arc::new(AtomicBool::new(true));
        let trace_writer: Arc<tokio::sync::Mutex<Option<super::PromptTraceWriter>>> =
            Arc::new(tokio::sync::Mutex::new(None));
        let prompt_rpc_id = Arc::new(AtomicU64::new(9));
        let cleanup = PromptRpcCleanup {
            busy: busy.clone(),
            trace_writer,
            prompt_rpc_id: prompt_rpc_id.clone(),
            idle_notify: None,
        };
        cleanup.clear_if_prompt_response(9).await;
        assert!(!busy.load(Ordering::SeqCst));
        assert_eq!(prompt_rpc_id.load(Ordering::SeqCst), 0);
    }
}
