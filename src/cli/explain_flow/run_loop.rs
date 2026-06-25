use crate::cli::error_run_log;
use crate::gate_kpop_workflow::{
    fail_gate_kpop_after_exhausted, finish_gate_kpop_after_pass, run_gate_kpop_loop,
    GateKpopLoopParams, GateLoopBehavior,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::{prepare_explain_kpop_run, ExplainKpopPrepared};
use super::prep::discover_explain_outputs_in_work_dir;
use super::{effective_explain_max_loops, ExplainArgs};

fn resolve_explain_output_paths(
    prepared: &ExplainKpopPrepared,
) -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    if prepared.auto_out_path {
        let discovered = discover_explain_outputs_in_work_dir(
            &prepared.request_work_dir,
            &prepared.preflight_snapshot,
        )?;
        return Ok((discovered.tex_path, discovered.pdf_path));
    }
    Ok((prepared.tex_path.clone(), prepared.pdf_path.clone()))
}

pub(crate) fn validate_explain_output(tex_path: &std::path::Path, pdf_path: &std::path::Path) -> Result<(), String> {
    for (label, path) in [("tex", tex_path), ("pdf", pdf_path)] {
        let meta = std::fs::metadata(path).map_err(|_| {
            format!(
                "malvin explain: expected {label} file at `{}`",
                path.display()
            )
        })?;
        if !meta.is_file() || meta.len() == 0 {
            return Err(format!(
                "malvin explain: expected non-empty {label} file at `{}`",
                path.display()
            ));
        }
    }
    Ok(())
}

struct ExplainGateFinish<'a> {
    shared: &'a SharedOpts,
    prepared: &'a ExplainKpopPrepared,
    tex_path: &'a std::path::Path,
    pdf_path: &'a std::path::Path,
    agent_ran: bool,
    gates_ok: bool,
    run_timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_backups: &'a crate::artifacts::SessionDotfileBackups,
    summarize_res: Result<(), String>,
}

fn explain_gate_outcome(finish: ExplainGateFinish<'_>) -> Result<(), String> {
    let gate_r = if finish.gates_ok {
        validate_explain_output(finish.tex_path, finish.pdf_path)?;
        finish_gate_kpop_after_pass(
            finish.shared,
            &finish.prepared.inner,
            finish.agent_ran,
            finish.run_timing,
        )
    } else if finish.agent_ran {
        if let Err(e) = validate_explain_output(finish.tex_path, finish.pdf_path) {
            Err(e)
        } else {
            Err(
                "malvin explain: gate loop did not exit on two consecutive ## KPOP_SOLVED markers"
                    .to_string(),
            )
        }
    } else {
        fail_gate_kpop_after_exhausted(
            "malvin explain",
            &finish.prepared.inner,
            finish.last_backups,
            GateLoopBehavior::EXPLAIN,
        )
    };
    crate::cli::workflow_kpop_shared::prefer_gate_outcome_over_summarize(gate_r, finish.summarize_res)
}

pub async fn run_explain(
    explain: &mut ExplainArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let prepared = prepare_explain_kpop_run(
        explain.request.as_ref(),
        &explain.out_path,
        explain.out_path_explicit,
        workflow,
    )?;
    if explain.out_path_explicit {
        explain.out_path =
            crate::cli::default_output_path::path_relative_to_cwd(&prepared.tex_path)?;
    }
    error_run_log::set_command_error_run_dir(Some(prepared.inner.artifacts.run_dir.clone()));
    emit_explain_startup(shared, &prepared)?;
    let gate_session = run_explain_gate_session(
        explain,
        shared,
        workflow,
        &prepared.inner,
    )
    .await?;
    finish_explain_run(ExplainFinishInput {
        explain,
        prepared: &prepared,
        shared,
        workflow,
        gate_session,
    })
    .await
}

async fn run_explain_gate_session(
    explain: &ExplainArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    prepared: &crate::gate_kpop_workflow::GateKpopPrepared,
) -> Result<
    (
        bool,
        bool,
        Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
        crate::artifacts::SessionDotfileBackups,
    ),
    String,
> {
    run_gate_kpop_loop(GateKpopLoopParams {
        command: "explain",
        shared,
        workflow,
        prepared,
        max_loops: effective_explain_max_loops(explain.max_loops),
        max_hypotheses: explain.max_hypotheses.max(1),
        behavior: GateLoopBehavior::EXPLAIN,
    })
    .await
}

fn emit_explain_startup(
    shared: &SharedOpts,
    prepared: &ExplainKpopPrepared,
) -> Result<(), String> {
    emit_run_startup_sequence(
        &prepared.inner.artifacts,
        RunStartupEmitOpts {
            tee_stdout: shared.tee_startup_stdout(),
            host_resources: true,
        },
        &prepared.inner.startup_emit_request,
    )
}

pub(crate) struct ExplainFinishInput<'a> {
    pub(crate) explain: &'a mut ExplainArgs,
    pub(crate) prepared: &'a ExplainKpopPrepared,
    pub(crate) shared: &'a SharedOpts,
    pub(crate) workflow: WorkflowCliOptions,
    pub(crate) gate_session: (
        bool,
        bool,
        Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
        crate::artifacts::SessionDotfileBackups,
    ),
}

async fn finish_explain_run(input: ExplainFinishInput<'_>) -> Result<(), String> {
    let ExplainFinishInput {
        explain,
        prepared,
        shared,
        workflow,
        gate_session,
    } = input;
    let (gates_ok, agent_ran, run_timing, last_backups) = gate_session;

    let (resolved_tex, resolved_pdf) = if prepared.auto_out_path {
        let discovered = resolve_explain_output_paths(&prepared)?;
        explain.out_path =
            crate::cli::default_output_path::path_relative_to_cwd(&discovered.0)?;
        discovered
    } else {
        (prepared.tex_path.clone(), prepared.pdf_path.clone())
    };

    let summarize_res = crate::cli::kpop_summarize::run_outer_loop_summarize_if_warranted(
        &crate::cli::kpop_summarize::OuterLoopSummarizeParams {
            agent_ran,
            shared,
            workflow,
            store: prepared.inner.store(),
            artifacts: prepared.inner.artifacts(),
            malvin_command: "malvin explain",
        },
    )
    .await;

    let r = explain_gate_outcome(ExplainGateFinish {
        shared,
        prepared: &prepared,
        tex_path: &resolved_tex,
        pdf_path: &resolved_pdf,
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
#[path = "../explain_flow_run_loop_tests.rs"]
mod explain_flow_run_loop_tests;
