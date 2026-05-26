mod counters;
mod multiturn;
mod multiturn_phases;
mod multiturn_types;
mod block_report;

#[cfg(test)]
#[path = "block_report_tests.rs"]
mod block_report_tests;

#[cfg(test)]
#[path = "counters_tests.rs"]
mod counters_tests;

#[cfg(test)]
mod multiturn_kiss;

pub use counters::{
    KPOP_CATCHUP_CAP, agent_declared_success, block_mean_from_p_creative, count_kpop_entries,
    count_kpop_solved_markers, count_mbc2_entries, hypotheses_emitted, poisson_block_size,
    read_exp_log_text,
};
pub(crate) use block_report::KpopBlockMissSnapshot;
pub use multiturn::KpopMultiturnState;
pub(crate) use multiturn_phases::{kpop_block_progress_ctx, set_last_block_miss};
pub use multiturn_types::KpopMultiturnParams;
