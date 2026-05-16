use std::path::Path;
use std::time::Duration;

use super::AcpSpawnArgs;

#[cfg(test)]
pub(super) fn george_mock_spawn_args<'a>(cwd: &'a Path, bin: &'a Path) -> AcpSpawnArgs<'a> {
    AcpSpawnArgs {
        cwd,
        bin_override: Some(bin),
        api_key: Some("george-test-api-key"),
        auth_token: Some("george-test-auth"),
        rpc_timeout: Duration::from_secs(crate::support_paths::DEFAULT_ACP_RPC_TIMEOUT_SECS),
        acp_verbose: false,
        george_acp_lane: None,
        ui_idle_notify: None,
        model: None,
        force: false,
        tee_trace_stdout: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        prompts_log_run_dir: None,
        log_full_outgoing_prompts: false,
    }
}

#[test]
fn kiss_stringify_george_mock_spawn_args() {
    let _ = george_mock_spawn_args(Path::new("."), Path::new("agent"));
    let _ = stringify!(george_mock_spawn_args);
}
