use crate::cli::error_run_log;
use crate::kpop_engine::{
    fail_kpop_engine_after_exhausted, finish_kpop_engine_after_pass, run_kpop_engine,
    KPopEngineParams, KPopHardConstraints,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::{prepare_priors_kpop_run, PriorsKpopPrepared};
use super::{effective_priors_max_loops, PriorsArgs};

pub(crate) fn validate_priors_output(resolved_out_path: &std::path::Path) -> Result<(), String> {
    let meta = std::fs::metadata(resolved_out_path).map_err(|_| {
        format!(
            "malvin priors: expected priors report at `{}`",
            resolved_out_path.display()
        )
    })?;
    if !meta.is_file() || meta.len() == 0 {
        return Err(format!(
            "malvin priors: expected non-empty priors report at `{}`",
            resolved_out_path.display()
        ));
    }
    Ok(())
}

struct PriorsGateFinish<'a> {
    shared: &'a SharedOpts,
    prepared: &'a PriorsKpopPrepared,
    agent_ran: bool,
    gates_ok: bool,
    run_timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_backups: &'a crate::artifacts::SessionDotfileBackups,
    summarize_res: Result<(), String>,
}

fn priors_gate_outcome(finish: PriorsGateFinish<'_>) -> Result<(), String> {
    let gate_r = if finish.gates_ok || finish.agent_ran {
        validate_priors_output(&finish.prepared.resolved_out_path)?;
        finish_kpop_engine_after_pass(
            finish.shared,
            &finish.prepared.inner,
            finish.agent_ran,
            finish.run_timing,
        )
    } else {
        fail_kpop_engine_after_exhausted(
            "malvin priors",
            &finish.prepared.inner,
            finish.last_backups,
            KPopHardConstraints::PRIORS,
        )
    };
    crate::cli::workflow_kpop_shared::prefer_gate_outcome_over_summarize(gate_r, finish.summarize_res)
}

pub async fn run_priors(
    priors: &mut PriorsArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let request_arg =
        crate::cli::cli_request::require_cli_request(priors.request.as_ref(), "priors")?;
    let prepared = prepare_priors_kpop_run(&request_arg, &priors.out_path, workflow)?;
    priors.out_path =
        crate::cli::default_output_path::path_relative_to_cwd(&prepared.resolved_out_path)?;
    error_run_log::set_command_error_run_dir(Some(prepared.inner.artifacts.run_dir.clone()));

    emit_run_startup_sequence(
        &prepared.inner.artifacts,
        RunStartupEmitOpts {
            tee_stdout: shared.tee_startup_stdout(),
            host_resources: true,
        },
        &prepared.inner.startup_emit_request,
    )?;

    let max_loops = effective_priors_max_loops(priors.max_loops);
    let (gates_ok, agent_ran, run_timing, last_backups) = run_kpop_engine(KPopEngineParams {
        command: "priors",
        shared,
        workflow,
        prepared: &prepared.inner,
        max_loops,
        max_hypotheses: priors.max_hypotheses,
        behavior: KPopHardConstraints::PRIORS,
    })
    .await?;

    let summarize_res = crate::cli::kpop_summarize::run_outer_loop_summarize_if_warranted(
        &crate::cli::kpop_summarize::OuterLoopSummarizeParams {
            agent_ran,
            shared,
            workflow,
            store: prepared.inner.store(),
            artifacts: prepared.inner.artifacts(),
            malvin_command: "malvin priors",
        },
    )
    .await;

    let r = priors_gate_outcome(PriorsGateFinish {
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

    fn priors_gate_outcome_prepared(tmp: &tempfile::TempDir, out_path: &std::path::Path) -> PriorsKpopPrepared {
        let store = crate::prompts::PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let artifacts = create_kpop_run_artifacts("priors", Some(tmp.path())).expect("artifacts");
        PriorsKpopPrepared {
            inner: crate::kpop_engine::KPopEnginePrepared {
                artifacts,
                context: crate::prompt_stratification::WorkflowRenderContext::default(),
                request_text: "req".into(),
                startup_emit_request: "req".into(),
                store,
                malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
            },
            resolved_out_path: out_path.to_path_buf(),
        }
    }

    #[test]
    fn priors_post_session_validates_output_file_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let err = validate_priors_output(&tmp.path().join("priors.md")).expect_err("missing");
        assert!(err.contains("expected priors report"));
    }

    #[test]
    fn priors_post_session_validates_output_file_non_empty() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let empty = tmp.path().join("priors.md");
        std::fs::write(&empty, "").expect("write");
        let err = validate_priors_output(&empty).expect_err("empty");
        assert!(err.contains("non-empty"));
    }

    #[test]
    fn priors_post_session_accepts_plain_markdown_without_begin_malvin() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("priors.md");
        std::fs::write(&plan, "# Priors\n\n- Use clap Args like delight.\n").expect("write");
        validate_priors_output(&plan).expect("ok");
    }

    #[test]
    fn priors_gate_outcome_succeeds_when_agent_ran_with_valid_output_without_gate_exit() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("priors.md");
        std::fs::write(&plan, "# Priors\n").expect("write");
        let prepared = priors_gate_outcome_prepared(&tmp, &plan);
        let shared = summarize_shared_opts(1);
        let backups = crate::artifacts::SessionDotfileBackups::snapshot(tmp.path()).expect("snap");
        priors_gate_outcome(PriorsGateFinish {
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
    fn priors_gate_outcome_fails_when_agent_ran_with_missing_output() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let prepared = priors_gate_outcome_prepared(&tmp, &tmp.path().join("priors.md"));
        let shared = summarize_shared_opts(1);
        let backups = crate::artifacts::SessionDotfileBackups::snapshot(tmp.path()).expect("snap");
        let err = priors_gate_outcome(PriorsGateFinish {
            shared: &shared,
            prepared: &prepared,
            agent_ran: true,
            gates_ok: false,
            run_timing: None,
            last_backups: &backups,
            summarize_res: Ok(()),
        })
        .expect_err("missing output should fail validation");
        assert!(err.contains("expected priors report"));
        assert!(!err.contains("mpc plan DONE"));
    }
}
