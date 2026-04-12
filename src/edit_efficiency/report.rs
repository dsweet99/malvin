//! Orchestration helpers: checkpoint logging and writing `edit_efficiency.json`.

use std::path::Path;

use super::EditEfficiencyMeter;

/// Best-effort checkpoint for orchestration; logs on failure without propagating.
pub fn maybe_checkpoint(meter: &mut Option<EditEfficiencyMeter>) {
    let Some(m) = meter.as_mut() else {
        return;
    };
    if let Err(e) = m.checkpoint() {
        tracing::warn!(target: "malvin::edit_efficiency", ?e, "checkpoint failed");
    }
}

/// Run [`EditEfficiencyMeter::finish`], log summary, write `edit_efficiency.json` under `run_dir`.
pub fn finish_and_write_report(meter: Option<EditEfficiencyMeter>, run_dir: &Path) {
    let Some(m) = meter else {
        return;
    };
    match m.finish() {
        Ok(report) => {
            tracing::info!(
                target: "malvin::edit_efficiency",
                gross = report.gross_bytes,
                net = report.net_bytes,
                efficiency = report.efficiency,
                checkpoint_calls = report.checkpoint_calls,
                gross_diff_steps = report.gross_diff_steps,
                "edit efficiency summary"
            );
            let path = run_dir.join("edit_efficiency.json");
            match serde_json::to_vec_pretty(&report) {
                Ok(bytes) => {
                    if let Err(e) = std::fs::write(&path, bytes) {
                        tracing::warn!(
                            target: "malvin::edit_efficiency",
                            ?e,
                            path = %path.display(),
                            "write edit_efficiency.json"
                        );
                    }
                }
                Err(e) => tracing::warn!(target: "malvin::edit_efficiency", ?e, "serialize report"),
            }
        }
        Err(e) => tracing::warn!(target: "malvin::edit_efficiency", ?e, "finish failed"),
    }
}

#[cfg(test)]
mod kiss_stringify_report {
    #[test]
    fn kiss_stringify_report_fns() {
        let _ = stringify!(super::maybe_checkpoint);
        let _ = stringify!(super::finish_and_write_report);
    }
}
