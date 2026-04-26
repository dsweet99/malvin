use std::path::Path;
use std::time::Duration;

use super::AcpSpawnArgs;

#[must_use]
pub(super) fn george_mock_spawn_args<'a>(cwd: &'a Path, bin: &'a Path) -> AcpSpawnArgs<'a> {
    AcpSpawnArgs {
        cwd,
        bin_override: Some(bin),
        api_key: Some("george-test-api-key"),
        auth_token: Some("george-test-auth"),
        rpc_timeout: Duration::from_secs(crate::config::DEFAULT_ACP_RPC_TIMEOUT_SECS),
        acp_verbose: false,
        george_acp_lane: None,
        ui_idle_notify: None,
        model: None,
        force: false,
        tee_trace_stdout: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
    }
}

#[test]
fn kiss_stringify_george_mock_spawn_args() {
    let _ = stringify!(george_mock_spawn_args);
}
