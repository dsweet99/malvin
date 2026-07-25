use super::*;
use super::super::prep::ExplainPreflightSnapshot;
use super::super::run_startup::ExplainKpopPrepared;

#[test]
fn finish_helpers_witness() {
    let _ = emit_explain_startup;
    let _ = finish_explain_success;
    let tmp = tempfile::tempdir().expect("tmp");
    let work = tmp.path();
    let store = crate::prompts::PromptStore::default_store();
    let _ = store.ensure_defaults();
    let artifacts = crate::artifacts::create_kpop_run_artifacts("explain", Some(work)).expect("a");
    let prepared = ExplainKpopPrepared {
        inner: crate::kpop_engine::KPopEnginePrepared {
            artifacts,
            context: crate::prompt_stratification::WorkflowRenderContext::default(),
            request_text: "r".into(),
            startup_emit_request: "r".into(),
            store,
            malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
        },
        tex_path: work.join("t.tex"),
        pdf_path: work.join("t.pdf"),
        request_work_dir: work.to_path_buf(),
        auto_out_path: false,
        preflight_snapshot: ExplainPreflightSnapshot::default(),
    };
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
    let input = ExplainSuccessInput {
        prepared: &prepared,
        shared: &shared,
        workflow: crate::cli::WorkflowCliOptions { force: true },
        tex_path: &prepared.tex_path,
        pdf_path: &prepared.pdf_path,
        agent_ran: true,
        run_timing: &timing,
    };
    assert!(input.agent_ran);
}
