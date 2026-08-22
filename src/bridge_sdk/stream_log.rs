use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::acp::AgentIoOptions;

use super::session::ToolCallStart;

pub struct StreamLog {
    pub io: AgentIoOptions,
    pub last_response: Arc<Mutex<String>>,
    pub timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
    pub run_dir: Option<PathBuf>,
    pub started_at: Instant,
    pub(crate) stdout_coalesce: Mutex<crate::acp::TraceChunkCoalescer>,
    pub tool_starts: Mutex<HashMap<String, ToolCallStart>>,
    pub thinking: Option<String>,
}

impl StreamLog {
    #[must_use]
    pub fn new(io: AgentIoOptions) -> Self {
        Self {
            io,
            last_response: Arc::new(Mutex::new(String::new())),
            timing: None,
            run_dir: None,
            started_at: Instant::now(),
            stdout_coalesce: Mutex::new(crate::acp::TraceChunkCoalescer::default()),
            tool_starts: Mutex::new(HashMap::new()),
            thinking: None,
        }
    }

    #[must_use]
    pub fn from_spawn(args: &super::BridgeSpawnArgs<'_>) -> Self {
        let mut log = Self::new(args.io);
        log.timing = args.timing.clone();
        log.run_dir = args.run_dir.clone();
        log.thinking = args.thinking.map(str::to_string);
        log
    }

    #[must_use]
    pub fn last_text(&self) -> String {
        self.last_response
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}
