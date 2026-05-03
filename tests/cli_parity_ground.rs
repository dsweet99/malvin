mod common;

#[cfg(unix)]
use common::{
    acp_mock_ground_check_abort_js, acp_mock_ground_check_tamper_kissconfig_js,
    acp_mock_ground_loop_converges_with_missing_grounding_js, acp_mock_ground_never_lgtm_loop_js,
    acp_mock_ground_write_tamper_kissconfig_js, only_run_dir, run_code_max_loops_zero_with_mock_stdout,
    run_ground_with_mock_js_with_setup, GroundMockOpts,
};

#[cfg(all(unix, target_os = "linux"))]
use common::{assert_workspace_gate_trace, run_ground_with_fake_gate_trace};

#[cfg_attr(unix, test)]
fn ground_converges_when_grounding_missing_and_loop_runs_check_improve_cycle() {
    let (out, _root, workspace) = run_ground_with_mock_js_with_setup(
        &acp_mock_ground_loop_converges_with_missing_grounding_js(),
        &GroundMockOpts {
            no_tee: true,
            with_kissconfig: false,
        },
        |workspace| {
            std::fs::remove_file(workspace.join("grounding.md")).expect("remove grounding");
        },
    );
    assert!(
        out.status.success(),
        "ground command should converge after converging mock flow: {out:?}"
    );
    let run_dir = only_run_dir(&workspace);
    let review = std::fs::read_to_string(run_dir.join("review.md")).expect("read review");
    assert_eq!(review, "LGTM\n");
    assert_eq!(
        std::fs::read_to_string(workspace.join("grounding.md")).expect("read grounding"),
        "CREATED\n"
    );
}

#[cfg_attr(unix, test)]
fn ground_max_loops_bounds_non_lgtm_grounding_loop() {
    let (out, _root, workspace) = run_ground_with_mock_js_with_setup(
        &acp_mock_ground_never_lgtm_loop_js(),
        &GroundMockOpts {
            no_tee: true,
            with_kissconfig: false,
        },
        |_| {},
    );
    assert!(
        !out.status.success(),
        "ground command should fail after exhausting max loops without LGTM: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("Did not receive LGTM for check_sync.md within max loops."),
        "non-convergent ground loop should emit max loop failure: {combined:?}"
    );
    let run_dir = only_run_dir(&workspace);
    let has_log = |label: &str| -> bool {
        std::fs::read_dir(&run_dir)
            .expect("read run dir")
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().contains(label))
    };
    assert!(
        has_log("check_sync_attempt_1"),
        "expected first check_sync attempt log"
    );
    assert!(
        has_log("check_sync_attempt_5"),
        "expected final check_sync attempt log at configured loop limit"
    );
    assert!(
        !has_log("check_sync_attempt_6"),
        "grounding loop should stop at loop limit and not run attempt 6"
    );
}

#[cfg_attr(unix, test)]
fn ground_aborts_from_check_prompt_result() {
    let (out, _root, workspace) = run_ground_with_mock_js_with_setup(
        &acp_mock_ground_check_abort_js(),
        &GroundMockOpts {
            no_tee: true,
            with_kissconfig: true,
        },
        |_| {},
    );
    assert!(
        !out.status.success(),
        "ground command should fail on check result ABORT: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("ABORT: reviewer requested stop"),
        "expected check ABORT visibility in ground output: {combined:?}"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.join(".kissconfig")).expect("read kissconfig"),
        "k\n"
    );
}

#[cfg_attr(unix, test)]
fn ground_restores_kissconfig_on_write_and_check_paths() {
    let (out_write, _root_write, workspace_write) = run_ground_with_mock_js_with_setup(
        &acp_mock_ground_write_tamper_kissconfig_js(),
        &GroundMockOpts {
            no_tee: true,
            with_kissconfig: true,
        },
        |workspace| {
            std::fs::remove_file(workspace.join("grounding.md")).expect("remove grounding");
        },
    );
    assert!(
        out_write.status.success(),
        "ground write-path flow with tampered kissconfig should restore and succeed: {out_write:?}"
    );
    assert_eq!(
        std::fs::read_to_string(workspace_write.join(".kissconfig")).expect("read kissconfig"),
        "k\n",
        "write-path restoration should restore kissconfig"
    );

    let (out_check, _root_check, workspace_check) = run_ground_with_mock_js_with_setup(
        &acp_mock_ground_check_tamper_kissconfig_js(),
        &GroundMockOpts {
            no_tee: true,
            with_kissconfig: true,
        },
        |_| {},
    );
    assert!(
        out_check.status.success(),
        "ground check-path tampering flow should restore and succeed: {out_check:?}"
    );
    assert_eq!(
        std::fs::read_to_string(workspace_check.join(".kissconfig")).expect("read kissconfig"),
        "k\n",
        "check-path restoration should restore kissconfig"
    );
    assert_eq!(
        std::fs::read_to_string(workspace_check.join("grounding.md")).expect("read grounding"),
        "x"
    );
}

#[cfg_attr(unix, test)]
fn ground_write_improve_and_check_prompts_are_reachable_and_rendered() {
    let (out, _root, workspace) = run_ground_with_mock_js_with_setup(
        &acp_mock_ground_loop_converges_with_missing_grounding_js(),
        &GroundMockOpts {
            no_tee: true,
            with_kissconfig: true,
        },
        |workspace| {
            std::fs::remove_file(workspace.join("grounding.md")).expect("remove grounding");
        },
    );
    assert!(
        out.status.success(),
        "ground prompt render coverage should succeed: {out:?}"
    );
    let run_dir = only_run_dir(&workspace);
    let markers = std::fs::read_to_string(run_dir.join("ground_prompt_visits.txt"))
        .expect("read marker file");
    assert!(
        markers.contains("write"),
        "expected write prompt path: {markers:?}"
    );
    assert!(
        markers.contains("check"),
        "expected check prompt path: {markers:?}"
    );
    assert!(
        markers.contains("improve"),
        "expected improve prompt path: {markers:?}"
    );
    assert!(
        !run_dir.join("result.md").exists(),
        "unrendered placeholder should fail test via result.md"
    );
}

#[cfg_attr(all(unix, target_os = "linux"), test)]
fn ground_runs_repo_workspace_gates_when_source_repo_markers_exist() {
    let (out, _root, trace) = run_ground_with_fake_gate_trace(
        &common::acp_mock_ground_loop_converges_with_missing_grounding_js(),
    );
    assert!(
        out.status.success(),
        "ground run with gates should succeed: {out:?}"
    );
    let trace_log = std::fs::read_to_string(&trace).unwrap_or_default();
    assert_workspace_gate_trace(&trace_log);
}

#[cfg_attr(unix, test)]
fn code_stdout_shows_plain_output_without_jsonrpc_lines() {
    let out = run_code_max_loops_zero_with_mock_stdout();
    assert!(
        !out.status.success(),
        "expected max-loops failure path: {out:?}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("agent message"),
        "expected parsed agent output on stdout: {stdout:?}"
    );
    assert!(
        !stdout.contains("\"jsonrpc\""),
        "stdout leaked JSON-RPC protocol lines: {stdout:?}"
    );
}
