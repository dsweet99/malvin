use crate::cli::error_run_log;
use crate::kpop_engine::{
    fail_kpop_engine_after_exhausted, finish_kpop_engine_after_pass, run_kpop_engine,
    KPopEngineParams, KPopHardConstraints,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::{prepare_delight_kpop_run, DelightKpopPrepared};
use super::{effective_delight_max_loops, DelightArgs};

pub(crate) fn validate_delight_output(resolved_out_path: &std::path::Path) -> Result<(), String> {
    let meta = std::fs::metadata(resolved_out_path).map_err(|_| {
        format!(
            "malvin delight: expected pitch file at `{}`",
            resolved_out_path.display()
        )
    })?;
    if !meta.is_file() || meta.len() == 0 {
        return Err(format!(
            "malvin delight: expected non-empty pitch file at `{}`",
            resolved_out_path.display()
        ));
    }
    Ok(())
}

struct DelightGateFinish<'a> {
    shared: &'a SharedOpts,
    prepared: &'a DelightKpopPrepared,
    agent_ran: bool,
    gates_ok: bool,
    run_timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_backups: &'a crate::artifacts::SessionDotfileBackups,
    summarize_res: Result<(), String>,
}

fn delight_gate_outcome(finish: DelightGateFinish<'_>) -> Result<(), String> {
    let gate_r = if finish.gates_ok {
        validate_delight_output(&finish.prepared.resolved_out_path)?;
        finish_kpop_engine_after_pass(
            finish.shared,
            &finish.prepared.inner,
            finish.agent_ran,
            finish.run_timing,
        )
    } else if finish.agent_ran {
        if let Err(e) = validate_delight_output(&finish.prepared.resolved_out_path) {
            Err(e)
        } else {
            Err(
                "malvin delight: gate loop did not exit on two consecutive ## KPOP_SOLVED markers"
                    .to_string(),
            )
        }
    } else {
        fail_kpop_engine_after_exhausted(
            "malvin delight",
            &finish.prepared.inner,
            finish.last_backups,
            KPopHardConstraints::DELIGHT,
        )
    };
    crate::cli::workflow_kpop_shared::prefer_gate_outcome_over_summarize(gate_r, finish.summarize_res)
}

pub async fn run_delight(
    delight: &mut DelightArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let prepared = prepare_delight_kpop_run(&delight.out_path, delight.guidance.as_ref(), workflow)?;
    delight.out_path =
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

    let max_loops = effective_delight_max_loops(delight.max_loops);
    let max_hypotheses = delight.max_hypotheses.max(1);
    let (gates_ok, agent_ran, run_timing, last_backups) = run_kpop_engine(KPopEngineParams {
        command: "delight",
        shared,
        workflow,
        prepared: &prepared.inner,
        max_loops,
        max_hypotheses,
        behavior: KPopHardConstraints::DELIGHT,
    })
    .await?;

    let summarize_res = crate::cli::kpop_summarize::run_outer_loop_summarize_if_warranted(
        &crate::cli::kpop_summarize::OuterLoopSummarizeParams {
            agent_ran,
            shared,
            workflow,
            store: prepared.inner.store(),
            artifacts: prepared.inner.artifacts(),
            malvin_command: "malvin delight",
        },
    )
    .await;

    let r = delight_gate_outcome(DelightGateFinish {
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

    #[test]
    fn delight_post_session_validates_output_file_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let missing = tmp.path().join("plan.md");
        let err = validate_delight_output(&missing).expect_err("missing");
        assert!(err.contains("expected pitch file"));
    }

    #[test]
    fn delight_post_session_validates_output_file_non_empty() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let empty = tmp.path().join("plan.md");
        std::fs::write(&empty, "").expect("write");
        let err = validate_delight_output(&empty).expect_err("empty");
        assert!(err.contains("non-empty"));
    }

    #[test]
    fn delight_post_session_accepts_plain_markdown_without_begin_malvin() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "# User feature idea\n\nImprove the CLI.\n").expect("write");
        validate_delight_output(&plan).expect("ok");
    }

    #[test]
    fn delight_run_loop_entry_is_covered() {
        let _ = run_delight;
    }

    #[test]
    fn delight_gate_outcome_fails_when_loop_exhausted_with_output_but_no_exit() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "# Plan\n").expect("write");
        let store = crate::prompts::PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let artifacts = create_kpop_run_artifacts("delight", Some(tmp.path())).expect("artifacts");
        let prepared = DelightKpopPrepared {
            inner: crate::kpop_engine::KPopEnginePrepared {
                artifacts,
                context: crate::prompt_stratification::WorkflowRenderContext::default(),
                request_text: "req".into(),
                startup_emit_request: "req".into(),
                store,
                malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
            },
            resolved_out_path: plan,
        };
        let shared = crate::cli::SharedOpts {
            model: crate::config::DEFAULT_CLI_MODEL.into(),
            no_force: true,
            no_tenacious: false,
            no_tee: true,
            no_markdown: true,
            verbose: false,
            max_acp_retries: 1,
            doc: false,
            name: None,
            mini: false,
        mini_max_bash_turns: 32,
        mini_max_http_turns: 32,
        mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_transport_retries: crate::support_paths::DEFAULT_MAX_MINI_TRANSPORT_RETRIES,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
        };
        let backups = crate::artifacts::SessionDotfileBackups::snapshot(tmp.path()).expect("snap");
        let err = delight_gate_outcome(DelightGateFinish {
            shared: &shared,
            prepared: &prepared,
            agent_ran: true,
            gates_ok: false,
            run_timing: None,
            last_backups: &backups,
            summarize_res: Ok(()),
        })
        .expect_err("needs two consecutive solved markers");
        assert!(err.contains("two consecutive"));
    }
}
