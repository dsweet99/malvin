#[test]
fn kiss_stringify_orchestrator_units() {
    let _ = stringify!(crate::orchestrator::Orchestrator);
    let _ = stringify!(crate::orchestrator::WorkflowError);
    let _ = stringify!(crate::orchestrator::WorkflowConfig);
    let _ = stringify!(crate::orchestrator::prefer_primary_errors_over_timing);
    let _ = stringify!(crate::orchestrator::Orchestrator::run);
    let _ = stringify!(crate::orchestrator::Orchestrator::run_with_pre_summary_gap);
    let _ = stringify!(crate::orchestrator::Orchestrator::run_bug_remediation_gap);
}
