use super::tests_gates_scenarios::{
    scenario_executes_only_malvin_checks_when_present,
    scenario_gate_run_wires_no_kiss_clamp_runner,
    scenario_gate_run_wires_quality_gates_runner,
    scenario_gate_run_wires_workspace_gates_runner,
    scenario_materializes_default_malvin_checks,
    scenario_prepare_repo_workspace_skips_quality_commands,
    scenario_quality_gates_log_records_gate_lines_when_run_log_dir_set,
    scenario_quality_gates_with_details_skips_auto_clamp_before_kiss_check,
    scenario_runs_tree_builtins_without_git_or_malvin_checks,
    scenario_runs_kiss_clamp_from_checks_when_kissconfig_valid,
    scenario_skips_pytest_without_test_named_py_files,
    scenario_strict_kissconfig_full_gates_skips_auto_clamp_before_kiss_check,
};

#[test]
fn run_repo_workspace_gates_executes_only_malvin_checks_when_present() {
    scenario_executes_only_malvin_checks_when_present();
}

#[test]
fn run_repo_workspace_gates_materializes_default_malvin_checks() {
    scenario_materializes_default_malvin_checks();
}

#[test]
fn run_repo_workspace_gates_runs_tree_builtins_without_git_or_malvin_checks() {
    scenario_runs_tree_builtins_without_git_or_malvin_checks();
}

#[test]
fn run_repo_workspace_gates_skips_pytest_without_test_named_py_files() {
    scenario_skips_pytest_without_test_named_py_files();
}

#[test]
fn quality_gates_log_records_gate_lines_when_run_log_dir_set() {
    scenario_quality_gates_log_records_gate_lines_when_run_log_dir_set();
}

#[test]
fn prepare_repo_workspace_skips_quality_commands() {
    scenario_prepare_repo_workspace_skips_quality_commands();
}

#[test]
fn run_repo_workspace_gates_runs_kiss_clamp_from_checks_when_kissconfig_valid() {
    scenario_runs_kiss_clamp_from_checks_when_kissconfig_valid();
}

#[test]
fn run_repo_workspace_gates_skips_auto_clamp_before_kiss_check_with_strict_kissconfig() {
    scenario_strict_kissconfig_full_gates_skips_auto_clamp_before_kiss_check();
}

#[test]
fn quality_gates_with_details_skips_auto_clamp_before_kiss_check() {
    scenario_quality_gates_with_details_skips_auto_clamp_before_kiss_check();
}

#[test]
fn gate_run_wires_quality_gates_runner_on_minimal_workspace() {
    scenario_gate_run_wires_quality_gates_runner();
}

#[test]
fn gate_run_wires_workspace_gates_runner_on_minimal_workspace() {
    scenario_gate_run_wires_workspace_gates_runner();
}

#[test]
fn gate_run_wires_no_kiss_clamp_runner_on_minimal_workspace() {
    scenario_gate_run_wires_no_kiss_clamp_runner();
}
