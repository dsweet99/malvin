use std::path::Path;

use crate::agent_sandbox::feasibility::{agent_install_root, linux_node_in_bundle};
use crate::agent_sandbox::spawn::guest_argv;
use crate::agent_sandbox::{load_mem_config, use_microsandbox_for_spawn};
use crate::agent_sandbox_config::{host_total_memory_bytes, parse_agent_sandbox_config};

#[test]
fn guest_argv_includes_acp_and_force() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bin = tmp.path().join("agent");
    let args = crate::acp::AcpSpawnArgs {
        cwd: tmp.path(),
        bin_override: Some(&bin),
        api_key: None,
        auth_token: None,
        rpc_timeout: std::time::Duration::from_secs(1),
        acp_verbose: false,
        george_acp_lane: None,
        ui_idle_notify: None,
        model: None,
        force: true,
        no_sandbox: true,
        tee_trace_stdout: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        prompts_log_run_dir: None,
        log_full_outgoing_prompts: false,
    };
    let v = guest_argv(&args);
    assert!(v.iter().any(|s| s == "acp"));
    assert!(v.iter().any(|s| s == "--force"));
}

#[test]
fn use_microsandbox_respects_no_sandbox() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(!use_microsandbox_for_spawn(true, tmp.path()));
}

#[test]
fn smoke_acp_ops_body_spawn_helpers() {
    let _ = crate::acp::test_no_real_agent_enabled();
    let _ = crate::acp::resolve_agent_bin();
}

#[test]
fn spawn_resolve_agent_bin_honors_override() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bin = tmp.path().join("agent");
    let args = crate::acp::AcpSpawnArgs {
        cwd: tmp.path(),
        bin_override: Some(&bin),
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
    assert_eq!(
        crate::agent_sandbox::spawn::resolve_spawn_agent_bin(&args).expect("bin"),
        bin
    );
}

#[test]
fn load_mem_config_reads_default_without_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cfg = load_mem_sandbox_config_path(tmp.path());
    assert!(cfg.mem_limit_gb >= 1);
}

fn load_mem_sandbox_config_path(p: &Path) -> crate::agent_sandbox_config::AgentSandboxConfig {
    load_mem_config(p)
}

#[test]
fn host_total_memory_bytes_positive_on_host() {
    if let Some(b) = host_total_memory_bytes() {
        assert!(b > 0);
    }
}

#[test]
fn parse_config_from_template() {
    let text = "mem_limit_gb = 4\n[logs]\nmax_runs = 1\n";
    let cfg = parse_agent_sandbox_config(text).expect("parse");
    assert_eq!(cfg.mem_limit_gb, 4);
}

#[test]
fn agent_install_root_parent() {
    let p = Path::new("/opt/agent/bin/agent");
    assert!(agent_install_root(p).ends_with("bin"));
}

#[test]
fn linux_node_skips_script_shims() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let shim = tmp.path().join("agent");
    std::fs::write(&shim, b"#!/bin/sh\n").expect("write");
    assert!(linux_node_in_bundle(&shim).is_none());
}

