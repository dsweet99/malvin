//! Edit-efficiency reporting: gross/net metering and git tree snapshots were removed.
//!
//! [`finish_and_write_report`] / [`finish_edit_efficiency_then_return`] still run at workflow
//! boundaries so callers keep a stable hook; they only emit the “not measured” hint on stderr.

mod report;

pub use report::{
    EDIT_EFFICIENCY_NOT_MEASURED_MESSAGE, finish_and_write_report,
    finish_edit_efficiency_then_return,
};
