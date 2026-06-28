mod acp_trace_shim;
mod bash_adapter;
mod client;
mod client_gate_retry;
#[path = "client_gate_retry_attempt.rs"]
mod client_gate_retry_attempt;
mod client_prompt_log;
mod context_recovery;
mod fence_parser;
mod loop_driver;
mod model_resolve;
mod retry_fork;
mod terminal;
mod trace;
mod trace_audit;

#[cfg(test)]
mod kiss_coverage;
#[cfg(test)]
mod client_retry_tests;
#[cfg(test)]
mod trace_tests;
#[cfg(test)]
#[path = "trace_do_plain_tests.rs"]
mod trace_do_plain_tests;
#[cfg(test)]
#[path = "trace_comment_tests.rs"]
mod trace_comment_tests;
#[cfg(test)]
#[path = "trace_stdout_tee_tests.rs"]
mod trace_stdout_tee_tests;
pub use trace::MiniTraceSink;

pub use client::{MiniAgentClient, MiniLoopConfig};
pub use retry_fork::MiniRetryStrategy;
pub use terminal::{MiniPhase, MiniTerminalReason, MiniTerminalRecord};
pub use loop_driver::{
    run_inner_loop, LoopDriverConfig, LoopDriverOutcome, LoopDriverRun, LoopDriverSession,
    LlmBackend, MockScript, MockStep,
};
