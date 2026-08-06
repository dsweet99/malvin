//! Prime SDK backend (`prime:` models via Node JSONL bridge to `prime-agent`).
#![cfg_attr(test, allow(unsafe_code))]

mod auth;
pub(crate) mod bridge_path;
mod client;
mod client_prompt;
mod client_session;
mod log_adapter;
mod log_adapter_tool;
mod models_list;
pub(crate) mod node_resolve;
mod protocol;
mod session;
mod session_io;
mod session_spawn;
mod timing;

pub use client::PrimeSdkClient;
pub use models_list::{list_prime_models_sync, PrimeModelListing};

#[cfg(test)]
mod client_mock_tests;
#[cfg(test)]
mod kiss_coverage_tests;
#[cfg(test)]
mod protocol_tests;
#[cfg(test)]
mod timing_tests;
#[cfg(test)]
mod session_io_tests;
#[cfg(test)]
mod session_spawn_tests;
#[cfg(test)]
mod log_adapter_tests;
#[cfg(test)]
mod node_resolve_tests;
