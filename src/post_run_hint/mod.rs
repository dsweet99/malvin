//! Post-run metrics hint: gross/net metering and git tree snapshots were removed.
//!
//! [`finish_and_write_report`] / [`finish_post_run_hint_then_return`] still run at workflow
//! boundaries so callers keep a stable hook; they only emit the “not measured” hint on stderr.

mod report;

pub use report::{
    POST_RUN_METRICS_NOT_MEASURED_MESSAGE, finish_and_write_report,
    finish_post_run_hint_then_return,
};
