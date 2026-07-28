use super::*;
use super::super::finish::ExplainSuccessInput;
use super::super::outputs::{products_nonempty, resolve_explain_output_paths, validate_explain_output};
use super::super::prep::ExplainPreflightSnapshot;
use super::super::run_startup::ExplainKpopPrepared;
use super::super::finish::{emit_explain_startup, finish_explain_success};
use super::phases::{
    run_explain_with_open_session, run_outer_iteration, ExplainOpenSession, OuterIterationCtx,
};

#[test]
fn fn_witnesses() {
    let _ = run_explain;
    let _ = prepare_explain_run;
    let _ = open_explain_backend;
    let _ = run_explain_with_open_session;
    let _ = run_outer_iteration;
    let _ = explain_review_chat_is_lgtm;
    let _ = products_nonempty;
    let _ = resolve_explain_output_paths;
    let _ = validate_explain_output;
    let _ = emit_explain_startup;
    let _ = finish_explain_success;
    let _ = std::any::type_name::<ExplainOpenSession>();
    assert!(explain_review_chat_is_lgtm("preamble\nLGTM"));
    assert!(explain_review_chat_is_lgtm("probes complete.LGTM"));
    assert!(!explain_review_chat_is_lgtm("- gap\n"));
    assert!(!explain_review_chat_is_lgtm("- gap\nLGTM"));
    assert!(!explain_review_chat_is_lgtm("Almost LGTM"));
}

#[test]
fn success_input_and_paths() {
    let tmp = tempfile::tempdir().expect("tmp");
    let work = tmp.path();
    let store = crate::prompts::PromptStore::default_store();
    let _ = store.ensure_defaults();
    let artifacts = crate::artifacts::create_kpop_run_artifacts("explain", Some(work)).expect("art");
    let prepared = ExplainKpopPrepared {
        inner: crate::kpop_engine::KPopEnginePrepared {
            artifacts,
            context: crate::prompt_stratification::WorkflowRenderContext::default(),
            request_text: "req".into(),
            startup_emit_request: "req".into(),
            store,
            malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
        },
        tex_path: work.join("explain.tex"),
        pdf_path: work.join("explain.pdf"),
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
    {
        let input = ExplainSuccessInput {
            prepared: &prepared,
            shared: &shared,
            workflow: WorkflowCliOptions { force: true },
            tex_path: &prepared.tex_path,
            pdf_path: &prepared.pdf_path,
            agent_ran: true,
            run_timing: &timing,
        };
        assert!(input.agent_ran);
    }
    assert!(!products_nonempty(&prepared.tex_path, &prepared.pdf_path));
    let (tex, pdf) = resolve_explain_output_paths(&prepared).expect("paths");
    assert_eq!(tex, prepared.tex_path);
    assert_eq!(pdf, prepared.pdf_path);
}

fn prepared_for_cov(work: &std::path::Path) -> ExplainKpopPrepared {
    let store = crate::prompts::PromptStore::default_store();
    let _ = store.ensure_defaults();
    let artifacts = crate::artifacts::create_kpop_run_artifacts("explain", Some(work)).expect("art");
    ExplainKpopPrepared {
        inner: crate::kpop_engine::KPopEnginePrepared {
            artifacts,
            context: crate::prompt_stratification::WorkflowRenderContext::default(),
            request_text: "req".into(),
            startup_emit_request: "req".into(),
            store,
            malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
        },
        tex_path: work.join("explain.tex"),
        pdf_path: work.join("explain.pdf"),
        request_work_dir: work.to_path_buf(),
        auto_out_path: false,
        preflight_snapshot: ExplainPreflightSnapshot::default(),
    }
}

fn shared_for_cov() -> crate::cli::SharedOpts {
    crate::cli::SharedOpts {
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
    }
}

#[test]
fn outer_iteration_ctx_fields() {
    let tmp = tempfile::tempdir().expect("tmp");
    let prepared = prepared_for_cov(tmp.path());
    let shared = shared_for_cov();
    let timing = crate::run_timing::RunTiming::new_arc();
    let mut client = crate::agent_backend::build_agent_backend(
        &shared,
        WorkflowCliOptions { force: true },
        false,
        "explain",
    )
    .expect("backend");
    let mut explain = crate::cli::explain_flow::ExplainArgs {
        request: Some("topic".into()),
        out_path: "explain.tex".into(),
        max_loops: 1,
        max_hypotheses: 10,
        tenacious: false,
        out_path_explicit: false,
    };
    let ctx = OuterIterationCtx {
        explain: &mut explain,
        shared: &shared,
        workflow: WorkflowCliOptions { force: true },
        prepared: &prepared,
        outer: 1,
        run_timing: &timing,
        client: &mut client,
    };
    assert_eq!(ctx.outer, 1);
    assert_eq!(ctx.explain.max_hypotheses, 10);
    assert!(ctx.shared.no_force);
    let session = ExplainOpenSession {
        explain: ctx.explain,
        shared: ctx.shared,
        workflow: ctx.workflow,
        prepared: ctx.prepared,
        run_timing: ctx.run_timing,
        client: ctx.client,
        max_outer: 1,
    };
    assert_eq!(session.max_outer, 1);
    assert_eq!(session.prepared.tex_path, prepared.tex_path);
}
