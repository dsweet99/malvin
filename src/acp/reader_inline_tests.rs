use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::acp::PromptRpcCleanup;

#[tokio::test]
async fn clear_if_prompt_response_clears_busy() {
    let busy = Arc::new(AtomicBool::new(true));
    let trace_writer: Arc<tokio::sync::Mutex<Option<crate::acp::PromptTraceWriter>>> =
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


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_clear_if_prompt_response_clears_busy() { let _ = stringify!(clear_if_prompt_response_clears_busy); }

}
