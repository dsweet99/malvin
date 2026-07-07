use crate::cli::error_run_log;
use crate::kpop_engine::{
    fail_kpop_engine_after_exhausted, finish_kpop_engine_after_pass, run_kpop_engine,
    KPopEngineParams, KPopHardConstraints,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::{prepare_revise_kpop_run, ReviseKpopPrepared};
use super::{effective_revise_max_loops, ReviseArgs};

pub(crate) fn validate_revise_output(resolved_doc_path: &std::path::Path) -> Result<(), String> {
    let meta = std::fs::metadata(resolved_doc_path).map_err(|_| {
        format!(
            "malvin revise: expected document at `{}`",
            resolved_doc_path.display()
        )
    })?;
    if !meta.is_file() || meta.len() == 0 {
        return Err(format!(
            "malvin revise: expected non-empty document at `{}`",
            resolved_doc_path.display()
        ));
    }
    Ok(())
}

struct ReviseGateFinish<'a> {
    shared: &'a SharedOpts,
    prepared: &'a ReviseKpopPrepared,
    agent_ran: bool,
    gates_ok: bool,
    run_timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_backups: &'a crate::artifacts::SessionDotfileBackups,
    summarize_res: Result<(), String>,
}

fn revise_gate_outcome(finish: ReviseGateFinish<'_>) -> Result<(), String> {
    let gate_r = if finish.gates_ok || finish.agent_ran {
        validate_revise_output(&finish.prepared.resolved_doc_path)?;
        finish_kpop_engine_after_pass(
            finish.shared,
            &finish.prepared.inner,
            finish.agent_ran,
            finish.run_timing,
        )
    } else {
        fail_kpop_engine_after_exhausted(
            "malvin revise",
            &finish.prepared.inner,
            finish.last_backups,
            KPopHardConstraints::REVISE,
        )
    };
    crate::cli::workflow_kpop_shared::prefer_gate_outcome_over_summarize(gate_r, finish.summarize_res)
}

pub async fn run_revise(
    revise: ReviseArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    crate::cli::checks_discovery_flow::ensure_malvin_checks_discovered_for_cwd(shared).await?;
    let prepared = prepare_revise_kpop_run(&revise.doc_path, workflow)?;
    error_run_log::set_command_error_run_dir(Some(prepared.inner.artifacts.run_dir.clone()));

    emit_run_startup_sequence(
        &prepared.inner.artifacts,
        RunStartupEmitOpts {
            tee_stdout: shared.tee_startup_stdout(),
            host_resources: true,
        },
        &prepared.inner.startup_emit_request,
    )?;

    let max_loops = effective_revise_max_loops(revise.max_loops);
    let (gates_ok, agent_ran, run_timing, last_backups) = run_kpop_engine(KPopEngineParams {
        command: "revise",
        shared,
        workflow,
        prepared: &prepared.inner,
        max_loops,
        max_hypotheses: revise.max_hypotheses,
        behavior: KPopHardConstraints::REVISE,
    })
    .await?;

    let summarize_res = crate::cli::kpop_summarize::run_outer_loop_summarize_if_warranted(
        &crate::cli::kpop_summarize::OuterLoopSummarizeParams {
            agent_ran,
            shared,
            workflow,
            store: prepared.inner.store(),
            artifacts: prepared.inner.artifacts(),
            malvin_command: "malvin revise",
        },
    )
    .await;

    let r = revise_gate_outcome(ReviseGateFinish {
        shared,
        prepared: &prepared,
        agent_ran,
        gates_ok,
        run_timing: run_timing.as_ref(),
        last_backups: &last_backups,
        summarize_res,
    });

    if r.is_ok() {
        error_run_log::clear_command_error_run_dir();
    }
    let _ = &prepared.inner.malvin_checks_backup;
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::create_kpop_run_artifacts;
    use crate::cli::kpop_summarize_tests::summarize_shared_opts;

    fn revise_gate_outcome_prepared(tmp: &tempfile::TempDir, doc_path: &std::path::Path) -> ReviseKpopPrepared {
        let store = crate::prompts::PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let artifacts = create_kpop_run_artifacts("revise", Some(tmp.path())).expect("artifacts");
        ReviseKpopPrepared {
            inner: crate::kpop_engine::KPopEnginePrepared {
                artifacts,
                context: crate::prompt_stratification::WorkflowRenderContext::default(),
                request_text: "req".into(),
                startup_emit_request: "req".into(),
                store,
                malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
            },
            resolved_doc_path: doc_path.to_path_buf(),
        }
    }

    #[test]
    fn revise_post_session_validates_output_file_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let err = validate_revise_output(&tmp.path().join("doc.md")).expect_err("missing");
        assert!(err.contains("expected document"));
    }

    #[test]
    fn revise_post_session_validates_output_file_non_empty() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let empty = tmp.path().join("doc.md");
        std::fs::write(&empty, "").expect("write");
        let err = validate_revise_output(&empty).expect_err("empty");
        assert!(err.contains("non-empty"));
    }

    #[test]
    fn revise_post_session_accepts_non_empty_document() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let doc = tmp.path().join("doc.md");
        std::fs::write(&doc, "# Revised\n\nClear prose.\n").expect("write");
        validate_revise_output(&doc).expect("ok");
    }

    #[test]
    fn revise_gate_outcome_succeeds_when_agent_ran_with_valid_output_without_gate_exit() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let doc = tmp.path().join("doc.md");
        std::fs::write(&doc, "# Revised\n").expect("write");
        let prepared = revise_gate_outcome_prepared(&tmp, &doc);
        let shared = summarize_shared_opts(1);
        let backups = crate::artifacts::SessionDotfileBackups::snapshot(tmp.path()).expect("snap");
        revise_gate_outcome(ReviseGateFinish {
            shared: &shared,
            prepared: &prepared,
            agent_ran: true,
            gates_ok: false,
            run_timing: None,
            last_backups: &backups,
            summarize_res: Ok(()),
        })
        .expect("valid output after agent ran should succeed");
    }

    #[test]
    fn revise_gate_outcome_fails_when_agent_ran_with_missing_output() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let prepared = revise_gate_outcome_prepared(&tmp, &tmp.path().join("doc.md"));
        let shared = summarize_shared_opts(1);
        let backups = crate::artifacts::SessionDotfileBackups::snapshot(tmp.path()).expect("snap");
        let err = revise_gate_outcome(ReviseGateFinish {
            shared: &shared,
            prepared: &prepared,
            agent_ran: true,
            gates_ok: false,
            run_timing: None,
            last_backups: &backups,
            summarize_res: Ok(()),
        })
        .expect_err("missing output should fail validation");
        assert!(err.contains("expected document"));
        assert!(!err.contains("mpc plan DONE"));
    }
}
