#![allow(unused_imports, clippy::wildcard_imports)]
mod inline {
    use std::path::PathBuf;
    use std::process::Command as StdCommand;
    use std::time::{Duration, Instant};
    include!("agent_bundle.rs");
}

pub(crate) use inline::*;
pub use inline::{AgentError, AgentIoOptions, AuthError};
