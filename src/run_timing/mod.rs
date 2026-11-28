mod acp_usage;
mod cost;
mod lifecycle;
mod policy;
mod process_ledger;
mod process_ledger_live;
mod report;
#[path = "report_cost_line.rs"]
mod report_cost_line;
mod state;
mod tokens;

use std::path::Path;

pub use policy::{
    CostPolicy, RUN_TIMING_JSON_FILE, RUN_TIMING_SUMMARY_PREFIX, TOOL_CALL_TYPE_MS_KEYS,
    TimingPhase, cost_policy_for_model, set_suppress_stdout_footnotes,
};
pub use process_ledger::{arm_process_footnotes, note_process_start};
pub(crate) use process_ledger_live::replace_tracked_timing;
pub use state::RunTiming;

pub fn emit_process_footnotes_if_armed() {
    process_ledger_live::flush_open_timing();
    process_ledger::emit_process_footnotes_if_armed();
}
pub use report_cost_line::RUN_COST_SUMMARY_PREFIX;
pub use tokens::ResponseUsage;

impl RunTiming {
    pub fn write_json_and_print_summary(&self, run_dir: &Path) -> std::io::Result<()> {
        report::write_json_and_print_summary(self, run_dir)
    }

    pub fn write_json_only(&self, run_dir: &Path) -> std::io::Result<()> {
        report::write_json_only(self, run_dir)
    }
}

pub(crate) use tokens::acp_usage_payload_is_observable;

pub use cost::record_completion_cost;
pub use lifecycle::{
    attach_new_run_timing, attach_new_run_timing_with_cost_policy, finalize_and_emit_run_timing,
    finalize_run_timing_json_only, persist_open_run_timing_json, record_backoff, record_llm,
};
pub use report::print_summary_from_run_dir;
pub use tokens::{
    note_acp_assistant_activity, note_acp_tool_call_completion, note_acp_tool_call_start,
    record_completion_step,
};

#[cfg(test)]
mod timing_tests;

#[cfg(test)]
pub(crate) mod timing_footnote_tests;

pub mod acp_post_run;
