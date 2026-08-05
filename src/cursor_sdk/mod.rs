//! Cursor TypeScript SDK backend (`cursor:` models via Node JSONL bridge).
#![cfg_attr(test, allow(unsafe_code))]

mod auth;
pub(crate) mod bridge_path;
mod client;
pub(super) mod client_prompt;
mod client_session;
mod log_adapter;
pub(crate) mod node_resolve;
mod protocol;
mod session;
mod session_io;
mod session_spawn;
mod timing;

pub use client::CursorSdkClient;

#[cfg(test)]
mod kiss_coverage;
#[cfg(test)]
mod bridge_path_tests;
#[cfg(test)]
mod protocol_tests;
#[cfg(test)]
mod session_mock_tests;
#[cfg(test)]
mod client_mock_tests;
