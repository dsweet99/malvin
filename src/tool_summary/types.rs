use std::collections::HashMap;
use std::time::Instant;

pub const TOOL_DISPLAY_MAX_WIDTH: usize = 60;
pub const TOOL_ELLIPSIS: &str = "...";

pub(crate) const ANSI_BOLD: &str = "\x1b[1m";
pub(crate) const ANSI_RESET: &str = "\x1b[0m";
pub(crate) const ANSI_TOOL_CREAM: &str = "\x1b[38;2;244;241;222m";
pub(crate) const ANSI_TOOL_CORAL: &str = "\x1b[38;2;224;122;95m";
pub(crate) const ANSI_TOOL_NAVY: &str = "\x1b[38;2;61;64;91m";
pub(crate) const ANSI_TOOL_TEAL: &str = "\x1b[38;2;129;178;154m";
pub(crate) const ANSI_TOOL_SAND: &str = "\x1b[38;2;242;204;143m";

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
