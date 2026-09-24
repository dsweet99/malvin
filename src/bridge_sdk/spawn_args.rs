use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::acp::AgentIoOptions;
use crate::model_id::ParsedModel;

pub const SDK_BRIDGE_MAX_AGE: Duration = Duration::from_mins(10);

#[derive(Debug, Clone)]
pub struct ToolCallStart {
    pub started: Instant,
    pub summary: String,
}

pub struct BridgeSpawnArgs<'a> {
    pub cwd: &'a Path,
    pub model: &'a ParsedModel,
    pub thinking: Option<&'a str>,
    pub io: AgentIoOptions,
    pub run_dir: Option<PathBuf>,
    pub timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
}

impl BridgeSpawnArgs<'_> {
    #[must_use]
    pub fn wire_model(&self) -> String {
        self.model.backend.bridge_wire_model(self.model)
    }
}
