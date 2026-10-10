use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::agent_process::AgentIoOptions;

use super::bridge_sdk::spawn_args::ToolCallStart;
use super::bridge_sdk::stdout_coalesce_buf::StdoutCoalesceBuf;

pub struct StreamLog {
    pub io: AgentIoOptions,
    pub last_response: Arc<Mutex<String>>,
    pub timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
    pub run_dir: Option<PathBuf>,
    pub started_at: Instant,
    pub(crate) stdout_coalesce: StdoutCoalesceBuf,
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
            stdout_coalesce: StdoutCoalesceBuf::new(io),
            tool_starts: Mutex::new(HashMap::new()),
            thinking: None,
        }
    }

    #[must_use]
    pub fn from_spawn(args: &super::bridge_sdk::BridgeSpawnArgs<'_>) -> Self {
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
