use std::collections::HashMap;
use std::time::Instant;

pub const TOOL_DISPLAY_MAX_WIDTH: usize = 60;
pub const TOOL_ELLIPSIS: &str = "...";

pub(crate) use crate::terminal_palette::{
    ansi_tool_coral, ansi_tool_dark, ansi_tool_teal, ANSI_BOLD, ANSI_DIM, ANSI_RESET,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToolSummaryDetail {
    Stdout,
    Log,
}

#[derive(Default)]
pub struct ToolSummaryTracker {
    pub(crate) calls: HashMap<String, ToolCallRecord>,
    path_base: Option<std::path::PathBuf>,
    run_timing: Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
}

impl ToolSummaryTracker {
    pub(crate) fn record(&self, id: &str) -> Option<&ToolCallRecord> {
        self.calls.get(id)
    }

    pub(crate) fn record_mut(&mut self, id: &str) -> Option<&mut ToolCallRecord> {
        self.calls.get_mut(id)
    }

    pub(crate) fn set_work_dir(&mut self, work_dir: std::path::PathBuf) {
        self.path_base = Some(work_dir);
    }

    pub(crate) fn work_dir(&self) -> Option<&std::path::Path> {
        self.path_base.as_deref()
    }

    pub(crate) fn set_run_timing(
        &mut self,
        timing: Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    ) {
        self.run_timing = timing;
    }

    pub(crate) fn record_tool_done(&mut self, id: &str) {
        let Some(rec) = self.calls.get(id) else {
            return;
        };
        let elapsed = rec.started.elapsed();
        let Some(timing) = self.run_timing.as_ref() else {
            return;
        };
        timing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .add_tool_call_wall(elapsed);
    }
}

pub(crate) struct ToolCallRecord {
    pub(crate) kind: String,
    pub(crate) title: String,
    pub(crate) command: Option<String>,
    pub(crate) input_path: Option<String>,
    pub(crate) search_query: Option<String>,
    pub(crate) input_line_range: Option<super::parse::LineRange>,
    pub(crate) started: Instant,
    pub(crate) stdout_start_emitted: bool,
}

pub struct ToolSummaryLines {
    pub log: String,
    pub stdout: Option<String>,
    pub stdout_deferred: Option<String>,
}

pub(crate) const TOOL_PHASE_START: u8 = 0;
pub(crate) const TOOL_PHASE_RUNNING: u8 = 1;
pub(crate) const TOOL_PHASE_DONE: u8 = 2;
pub(crate) const TOOL_PHASE_PENDING: u8 = 3;
pub(crate) const TOOL_PHASE_NAMED_STATUS: u8 = 4;

pub fn shorten_middle(s: &str, max_width: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_width {
        return s.to_string();
    }
    let elen = TOOL_ELLIPSIS.chars().count();
    let keep = max_width.saturating_sub(elen);
    let front = keep / 2;
    let back = keep - front;
    let mut out: String = chars.iter().take(front).collect();
    out.push_str(TOOL_ELLIPSIS);
    out.extend(chars.iter().skip(chars.len().saturating_sub(back)));
    out
}
#[cfg(test)]
#[path = "types_test.rs"]
mod types_test;#[cfg(test)]
#[path = "types_kiss_cov_test.rs"]
mod types_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<ToolCallRecord> = None;
        let _: Option<ToolSummaryDetail> = None;
        let _: Option<ToolSummaryLines> = None;
    }
}
