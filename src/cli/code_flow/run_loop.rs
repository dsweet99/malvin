use crate::cli::error_run_log;
use crate::gate_kpop_workflow::{
    fail_gate_kpop_after_exhausted, finish_gate_kpop_after_pass, run_gate_kpop_loop,
    GateKpopLoopParams, GateLoopBehavior,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::prepare_code_kpop_run;
use super::{effective_code_max_loops, CodeArgs};

fn emit_code_run_startup(
    shared: &SharedOpts,
    prepared: &super::run_startup::CodeKpopPrepared,
) -> Result<(), String> {
    emit_run_startup_sequence(
        &prepared.artifacts,
        RunStartupEmitOpts {
            tee_stdout: shared.tee_startup_stdout(),
            host_resources: true,
        },
        &prepared.startup_emit_request,
    )
}

struct CodeGateFinish<'a> {
    shared: &'a SharedOpts,
    prepared: &'a super::run_startup::CodeKpopPrepared,
    agent_ran: bool,
    gates_ok: bool,
    run_timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_backups: &'a crate::artifacts::SessionDotfileBackups,
    summarize_res: Result<(), String>,
}

fn code_gate_outcome(finish: CodeGateFinish<'_>) -> Result<(), String> {
    let gate_r = if finish.gates_ok {
        finish_gate_kpop_after_pass(
            finish.shared,
            finish.prepared,
            finish.agent_ran,
            finish.run_timing,
        )
    } else {
        fail_gate_kpop_after_exhausted(
            "malvin code",
            finish.prepared,
            finish.last_backups,
            GateLoopBehavior::CODE,
        )
    };
    crate::cli::workflow_kpop_shared::prefer_gate_outcome_over_summarize(gate_r, finish.summarize_res)
}

pub async fn run_code(
    code: CodeArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    request: &str,
) -> Result<(), String> {
    let cli_request = request.trim();
    if cli_request.is_empty() {
        return Err("malvin code: missing required REQUEST (text or path)".into());
    }
    let prepared = prepare_code_kpop_run(workflow, cli_request)?;
    error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));

    emit_code_run_startup(shared, &prepared)?;

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
    let r = code_gate_outcome(CodeGateFinish {
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
        mini_max_http_turns: 32,
        mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_transport_retries: crate::support_paths::DEFAULT_MAX_MINI_TRANSPORT_RETRIES,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
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

#[cfg(test)]
#[path = "run_loop_kiss_cov.rs"]
mod run_loop_kiss_cov;
