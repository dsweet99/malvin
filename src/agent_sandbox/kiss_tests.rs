use super::{load_mem_config, sandbox_test_no_real_agent_enabled, use_microsandbox_for_spawn};
use crate::agent_sandbox::spawn::resolve_spawn_agent_bin;

#[test]
fn smoke_agent_sandbox_private_helpers() {
    let _ = sandbox_test_no_real_agent_enabled();
    let tmp = tempfile::tempdir().expect("tempdir");
    let _ = load_mem_config(tmp.path());
    let _ = use_microsandbox_for_spawn(true, tmp.path());
    let bin = tmp.path().join("bin");
    let args = crate::acp::AcpSpawnArgs {
        cwd: tmp.path(),
        bin_override: Some(bin.as_path()),
        api_key: None,
        auth_token: None,
        rpc_timeout: std::time::Duration::from_secs(1),
        acp_verbose: false,
        george_acp_lane: None,
        ui_idle_notify: None,
        model: None,
        force: false,
        no_sandbox: true,
        tee_trace_stdout: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        prompts_log_run_dir: None,
        log_full_outgoing_prompts: false,
    };
    let _ = resolve_spawn_agent_bin(&args);
}

