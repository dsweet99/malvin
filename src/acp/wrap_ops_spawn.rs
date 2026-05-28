#![allow(unused_imports, clippy::wildcard_imports)]
mod inline {
    use std::path::{Path, PathBuf};
    use std::process::Command as StdCommand;
    use std::time::Duration;
    use super::super::import_prelude::*;
    use super::super::session_types::*;
    use super::super::{AgentClient, AgentError, AcpSession};
    include!("ops_body_spawn.inc");
}

pub(crate) use inline::*;
