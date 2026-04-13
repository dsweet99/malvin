//! Shared result merge for ACP runs that emit [`crate::run_timing`] artifacts.

use std::path::Path;
use std::sync::{Arc, Mutex};

use malvin::acp::AgentClient;
use malvin::run_timing::RunTiming;

/// Prefer ACP failures over run-timing artifact errors once run timing emission completes.
pub fn merge_acp_and_timing_results(
    acp_result: Result<(), String>,
    timing_result: std::io::Result<()>,
) -> Result<(), String> {
    match acp_result {
        Ok(()) => timing_result.map_err(|e| e.to_string()),
        Err(e) => {
            let _ = timing_result;
            Err(e)
        }
    }
}

/// After ACP work: write `run_timing.json`, print `TIMING:`, clear [`AgentClient`] timing slot, merge errors.
pub fn emit_run_timing_after_acp(
    client: &mut AgentClient,
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
    acp_result: Result<(), String>,
) -> Result<(), String> {
    let timing_result = malvin::run_timing::finalize_and_emit_run_timing(run_dir, timing);
    client.set_run_timing(None);
    merge_acp_and_timing_results(acp_result, timing_result)
}
