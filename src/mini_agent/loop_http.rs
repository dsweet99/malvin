//! HTTP completion retries for the mini loop driver.

use std::sync::{Arc, Mutex};

use crate::llm_transport::TransportError;

use crate::mini_agent::trace::MiniTraceSink;

pub enum HttpCompletionError {
    Exhausted(String),
    ContextOverflow,
}

impl std::fmt::Debug for HttpCompletionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exhausted(msg) => f.debug_tuple("Exhausted").field(msg).finish(),
            Self::ContextOverflow => f.write_str("ContextOverflow"),
        }
    }
}

pub struct HttpRetryRequest<'a> {
    pub llm: &'a super::loop_mock::LlmBackend,
    pub messages: &'a [crate::openrouter_transport::ChatMessage],
    pub max_transport_retries: u32,
    pub single_attempt: bool,
    pub timing: Option<&'a Arc<Mutex<crate::run_timing::RunTiming>>>,
    pub trace: Option<&'a MiniTraceSink>,
}

pub(crate) async fn backoff_before_http_retry(
    timing: Option<&Arc<Mutex<crate::run_timing::RunTiming>>>,
    failures: u32,
    err: &TransportError,
) {
    crate::output::print_log_error(&format!(
        "mini HTTP attempt {failures} failed (transport): {err}"
    ));
    let sleep = if failures == 1 {
        std::time::Duration::from_secs(1)
    } else {
        std::time::Duration::from_secs(3)
    };
    crate::run_timing::record_backoff(timing, sleep);
    crate::acp::agent_backoff_sleep(sleep).await;
}

pub(crate) fn exhaustion_message(failures: u32, limit: u32, detail: &str) -> String {
    format!(
        "mini HTTP failed after {failures} transport attempts (limit {limit}): {detail}"
    )
}

pub use crate::openrouter_transport::complete_transport_with_retries;
