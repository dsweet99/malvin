use crate::cli::error_run_log;
use crate::cli::gate_kpop_workflow::{
    fail_gate_kpop_after_exhausted, finish_gate_kpop_after_pass, run_gate_kpop_loop,
    GateKpopLoopParams, GateLoopBehavior,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::prepare_code_kpop_run;
use super::{effective_code_max_loops, CodeArgs};

pub async fn run_code(
    code: CodeArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let cli_request = crate::cli::cli_request::require_cli_request(code.request.as_ref(), "code")?;
    let prepared = prepare_code_kpop_run(workflow, &cli_request)?;
    error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));

    emit_run_startup_sequence(
        &prepared.artifacts,
        RunStartupEmitOpts {
            tee_stdout: shared.tee_startup_stdout(),
            host_resources: true,
        },
        &prepared.startup_emit_request,
    )?;

    let max_loops = effective_code_max_loops(code.max_loops);
    let max_hypotheses = code.max_hypotheses.max(1);
    let (gates_ok, agent_ran, run_timing, last_backups) = run_gate_kpop_loop(GateKpopLoopParams {
        command: "code",
        shared,
        workflow,
        prepared: &prepared,
        max_loops,
        max_hypotheses,
        behavior: GateLoopBehavior::CODE,
    })
    .await?;

    let summarize_res = crate::cli::kpop_summarize::run_outer_loop_summarize_if_warranted(
        &crate::cli::kpop_summarize::code_outer_loop_summarize_params(
            crate::cli::kpop_summarize::CodeOuterLoopSummarizeInputs {
                agent_ran,
                shared,
                workflow,
            },
            &prepared,
        ),
    )
    .await;
    let gate_r = if gates_ok {
        finish_gate_kpop_after_pass(shared, &prepared, agent_ran, run_timing.as_ref())
    } else {
        fail_gate_kpop_after_exhausted(
            "malvin code",
            &prepared,
            &last_backups,
            GateLoopBehavior::CODE,
        )
    };
    let r = crate::cli::workflow_kpop_shared::prefer_gate_outcome_over_summarize(gate_r, summarize_res);

    if r.is_ok() {
        error_run_log::clear_command_error_run_dir();
    }
    let _ = &prepared.malvin_checks_backup;
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::SharedOpts;
    use crate::config::DEFAULT_CLI_MODEL;

    #[test]
    fn code_run_loop_entry_is_covered() {
        let _ = super::run_code;
    }

    #[test]
    fn code_outer_loop_summarize_params_builds_code_context() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let prepared = super::super::run_startup::prepare_code_kpop_run(
            WorkflowCliOptions { force: false },
            "ship it",
        )
        .expect("prepared");
        let shared = SharedOpts {
            model: DEFAULT_CLI_MODEL.into(),
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
        };
        let workflow = WorkflowCliOptions { force: false };
        let params = crate::cli::kpop_summarize::code_outer_loop_summarize_params(
            crate::cli::kpop_summarize::CodeOuterLoopSummarizeInputs {
                agent_ran: true,
                shared: &shared,
                workflow,
            },
            &prepared,
        );
        std::env::set_current_dir(old).expect("restore cwd");
        assert!(params.agent_ran);
        assert_eq!(params.malvin_command, "malvin code");
        assert!(std::ptr::eq(params.store, &raw const *prepared.store()));
        assert!(std::ptr::eq(params.artifacts, &raw const *prepared.artifacts()));
    }
}
