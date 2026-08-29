use super::env::HerdrEnv;
use super::request::{clear_metadata_teardown, next_seq, report_agent};
use super::send::send_request_checked;

/// Reset the current herdr pane's malvin agent state to idle (not working).
///
/// Requires `HERDR_ENV=1`, `HERDR_SOCKET_PATH`, and `HERDR_PANE_ID`.
pub fn reset_to_not_working() -> Result<(), String> {
    let env = HerdrEnv::from_os_env().ok_or_else(|| {
        "herdr env not set (need HERDR_ENV=1, HERDR_SOCKET_PATH, HERDR_PANE_ID)".to_string()
    })?;
    reset_env_to_not_working(&env)
}

pub(crate) fn reset_env_to_not_working(env: &HerdrEnv) -> Result<(), String> {
    send_request_checked(
        &env.socket_path,
        &report_agent(&env.pane_id, "idle", None, next_seq()),
    )
    .map_err(|e| format!("herdr reset idle failed: {e}"))?;
    send_request_checked(
        &env.socket_path,
        &clear_metadata_teardown(&env.pane_id, next_seq()),
    )
    .map_err(|e| format!("herdr reset clear-metadata failed: {e}"))?;
    Ok(())
}

#[cfg(test)]
#[path = "reset_tests.rs"]
mod reset_tests;
