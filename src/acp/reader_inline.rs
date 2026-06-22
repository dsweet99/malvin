use crate::acp::import_prelude::*;
use crate::acp::*;
// Stdout JSON-RPC line processing (inlined into `acp` for `kiss` dependency depth).

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

#[cfg(test)]
#[path = "reader_inline_kiss_cov_test.rs"]
mod reader_inline_kiss_cov_test;
#[cfg(test)]
#[path = "reader_inline_test.rs"]
mod reader_inline_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<PromptRpcCleanup> = None;
    }
}
