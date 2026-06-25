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
mod plan_chain_tests;
#[cfg(test)]
mod trace_tests;
pub use trace::MiniTraceSink;

pub use client::{MiniAgentClient, MiniLoopConfig};
pub use loop_driver::{LlmBackend, MockScript, MockStep};
