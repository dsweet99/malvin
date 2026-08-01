mod acp_trace_shim;
mod bash_adapter;
mod client;
pub mod protocol;
mod client_prompt_log;
mod context_recovery;
mod fence_parser;
pub(crate) mod loop_driver;
mod memory_assemble;
mod model_resolve;
pub(crate) mod retry_fork;
mod terminal;
mod trace;
pub(crate) mod trace_audit;

#[cfg(test)]
mod kiss_coverage;
#[cfg(test)]
#[path = "loop_mock_kiss_cov.rs"]
mod loop_mock_kiss_cov;
#[cfg(test)]
mod client_retry_tests;
#[cfg(test)]
mod trace_tests;
#[cfg(test)]
#[path = "trace_stdout_tee_tests.rs"]
mod trace_stdout_tee_tests;
#[cfg(test)]
#[path = "trace_do_plain_tests.rs"]
mod trace_do_plain_tests;
#[cfg(test)]
#[path = "trace_comment_tests.rs"]
mod trace_comment_tests;
#[cfg(test)]
#[path = "trace_http_exchange_tests.rs"]
mod trace_http_exchange_tests;
pub use trace::{record_http_exchange, MiniTraceSink};
pub use acp_trace_shim::MiniHttpExchangeRecord;

pub use client::{MiniAgentClient, MiniLoopConfig};
pub use model_resolve::resolve_mini_model;
pub use retry_fork::MiniRetryStrategy;
pub use terminal::{MiniPhase, MiniTerminalReason, MiniTerminalRecord};
pub use loop_driver::{
    run_inner_loop, LoopDriverConfig, LoopDriverOutcome, LoopDriverRun, LoopDriverSession,
    LlmBackend, MockScript, MockStep,
};
pub(crate) use loop_driver::{
    backoff_before_http_retry, exhaustion_message,
    HttpCompletionError, HttpRetryRequest, LlmCompletionOutcome,
};
