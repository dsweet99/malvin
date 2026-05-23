#![allow(unused_imports, clippy::wildcard_imports)]
mod inline {
    #![allow(clippy::wildcard_imports)]
    use super::super::jsonl_trace::AcpJsonlTrace;
    use super::super::session_types::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU64};
    use tokio::process::{Child, ChildStdin, ChildStdout};
    use tokio::sync::{Mutex, Notify};
    include!("session_channels.inc");
}

pub(crate) use inline::*;
