use super::*;

#[test]
fn explain_work_params_fields() {
    let _ = run_explain_work;
    let shared = crate::cli::SharedOpts {
        model: crate::config::DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tenacious: false,
        gates: false,
        no_tee: true,
        no_markdown: true,
        verbose: false,
        max_acp_retries: 1,
        doc: false,
        name: None,
        mini_max_bash_turns: 1,
        mini_max_http_turns: 1,
        mini_max_bash_execs: 1,
        mini_max_http_retries: 0,
        mini_max_transport_retries: 0,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
        no_download: false,
        git: false,
    };
    let timing = crate::run_timing::RunTiming::new_arc();
    let tmp = tempfile::tempdir().expect("tmp");
    let work = tmp.path();
    let store = crate::prompts::PromptStore::default_store();
    let _ = store.ensure_defaults();
    let artifacts = crate::artifacts::create_kpop_run_artifacts("explain", Some(work)).expect("a");
    let prepared = crate::kpop_engine::KPopEnginePrepared {
        artifacts,
        context: crate::prompt_stratification::WorkflowRenderContext::default(),
        request_text: String::from("r"),
        startup_emit_request: String::from("r"),
        store,
        malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
    };
    let p = ExplainWorkParams {
        shared: &shared,
        workflow: WorkflowCliOptions { force: false },
        prepared: &prepared,
        work_request: "w",
        run_timing: &timing,
    };
    assert!(p.shared.no_force);
    assert!(!p.workflow.force);
    assert_eq!(p.work_request, "w");
    assert!(p.run_timing.lock().is_ok());
}
