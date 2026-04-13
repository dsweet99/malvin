//! Shared result merge for ACP runs that emit [`crate::run_timing`] artifacts.

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
