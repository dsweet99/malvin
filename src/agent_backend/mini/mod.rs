mod acp_trace_shim;
mod bash_adapter;
mod client;
mod client_prompt_log;
mod fence_parser;
mod loop_driver;
mod model_resolve;
mod trace;

#[cfg(test)]
mod kiss_coverage;
#[cfg(test)]
mod client_retry_tests;
#[cfg(test)]
mod trace_tests;
#[cfg(test)]
#[path = "trace_comment_tests.rs"]
mod trace_comment_tests;
pub use trace::MiniTraceSink;

pub use client::{MiniAgentClient, MiniLoopConfig};
pub use loop_driver::{
    run_inner_loop, LoopDriverConfig, LoopDriverOutcome, LoopDriverRun, LoopDriverSession,
    LlmBackend, MockScript, MockStep,
};
