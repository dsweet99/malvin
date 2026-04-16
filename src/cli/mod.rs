//! Subcommand dispatch and async entrypoints for the `malvin` binary.

mod args;
#[cfg(all(test, unix))]
mod command_log_tests;
mod do_flow;
mod exit;
mod init_cmd;
mod kiss_clamp;
mod kpop_flow;
mod repo_checks;
mod models_cmd;
mod shared_opts;
#[cfg(test)]
mod stringify_cov;
mod timing_merge;

pub use args::{Cli, CodeArgs, Commands, KpopArgs};
pub use exit::Exit;
pub use shared_opts::SharedOpts;

use clap::Parser;

use malvin::env_path::require_kiss_for_malvin;
use malvin::output::{
    MALVIN_WHO, format_line, print_stderr_line, print_stdout_line, print_stdout_text,
};
use std::path::Path;

/// Writes `command.log` under `run_dir`. When `echo_stdout` is true (tee on), also prints `Command: …` to stdout — same flag semantics as [`SharedOpts::tee_startup_stdout`].
pub fn emit_command_line(run_dir: &Path, echo_stdout: bool) -> Result<(), String> {
    malvin::invocation::init_from_env();
    let cmd =
        malvin::invocation::command_line().expect("init_from_env populates argv via OnceLock");
    let line = format!("Command: {cmd}");
    if echo_stdout {
        print_stdout_line(MALVIN_WHO, &line);
    }
    let log_path = run_dir.join("command.log");
    std::fs::write(&log_path, format!("{}\n", format_line(MALVIN_WHO, &line)))
        .map_err(|e| format!("command.log: {e}"))?;
    Ok(())
}

pub use do_flow::run_do;
pub use kpop_flow::run_kpop;
use malvin::acp::AgentClient;

use malvin::artifacts::{
    RunArtifacts, backup_workspace_grounding_if_present, create_run_artifacts_from_text,
    resolve_user_request, restore_workspace_grounding, startup_request_tag_label,
};
use malvin::log_paths::format_logs_dir;
use malvin::orchestrator::{Orchestrator, WorkflowConfig, WorkflowError};
use malvin::prompts::{PromptError, PromptStore};

#[derive(Debug, Clone, Copy)]
pub struct WorkflowCliOptions {
    pub force: bool,
    pub run_learn: bool,
}

/// Skip learn phase if elapsed time is below 5 minutes (300,000 ms).
pub const LEARN_MIN_ELAPSED_MS: u64 = 300_000;

pub fn prepare_prompt_store(workflow: WorkflowCliOptions) -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store.validate_required().map_err(|e: PromptError| e.0)?;
    if workflow.run_learn {
        store
            .validate_exists("learn.md")
            .map_err(|e: PromptError| e.0)?;
    }
    Ok(store)
}

/// Like [`prepare_prompt_store`] but only checks prompts used by `malvin kpop` (not the full workflow set).
pub fn prepare_kpop_prompt_store(
    workflow: WorkflowCliOptions,
    require_mbc2: bool,
) -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_kpop_prompts(malvin::prompts::KpopPromptValidation {
            run_learn: workflow.run_learn,
            require_mbc2,
        })
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn echo_primary_to_stdout(
    plan_path: &Path,
    echo_plain: bool,
    startup_tag_label: &str,
) -> Result<(), String> {
    if !echo_plain {
        return Ok(());
    }
    let plan_text = std::fs::read_to_string(plan_path).map_err(|e| e.to_string())?;
    print_stdout_text(startup_tag_label, &plan_text);
    Ok(())
}

/// Write `command.log` / optional `Command:` line, echo the primary run artifact, then print `Logs: …`.
///
/// Shared by `malvin code`, `malvin kpop`, and `malvin do` so startup output stays consistent.
pub fn emit_run_startup_sequence(
    artifacts: &RunArtifacts,
    tee_startup_stdout: bool,
    cli_request: &str,
) -> Result<(), String> {
    emit_command_line(&artifacts.run_dir, tee_startup_stdout)?;
    let tag = startup_request_tag_label(cli_request);
    echo_primary_to_stdout(&artifacts.plan_path, tee_startup_stdout, &tag)?;
    print_stdout_line(
        MALVIN_WHO,
        &format!("Logs: {}", format_logs_dir(&artifacts.run_dir)?),
    );
    Ok(())
}

fn prepare_code_run(
    code: &CodeArgs,
    workflow: WorkflowCliOptions,
) -> Result<(PromptStore, AgentClient, RunArtifacts), String> {
    let store = prepare_prompt_store(workflow)?;
    let client = build_agent(&code.shared, workflow);
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let (text, work_dir) = resolve_user_request(&code.request)?;
    let artifacts = create_run_artifacts_from_text(&text, Some(work_dir.as_path()))
        .map_err(|e| e.to_string())?;
    Ok((store, client, artifacts))
}

pub async fn run_code(code: CodeArgs, workflow: WorkflowCliOptions) -> Result<(), String> {
    let (store, mut client, artifacts) = prepare_code_run(&code, workflow)?;

    repo_checks::run_repo_workspace_gates(&artifacts.work_dir)?;

    let grounding_backup = backup_workspace_grounding_if_present(&artifacts.work_dir)?;

    emit_run_startup_sequence(&artifacts, code.shared.tee_startup_stdout(), &code.request)?;

    let mut orch = Orchestrator {
        client: &mut client,
        prompts: &store,
        artifacts: &artifacts,
        config: WorkflowConfig {
            max_loops: code.max_loops,
            run_learn: workflow.run_learn,
            learn_min_elapsed_ms: LEARN_MIN_ELAPSED_MS,
            skip_check_plan: code.trust_the_plan,
        },
        progress_callback: Box::new(|msg: &str| {
            print_stdout_line(MALVIN_WHO, msg);
        }),
        grounding_backup: grounding_backup.clone(),
    };
    let workflow_res = orch.run().await.map_err(|e: WorkflowError| e.0);
    let restore_res = grounding_backup
        .as_ref()
        .map_or(Ok(()), |b| restore_workspace_grounding(&artifacts.work_dir, b));
    timing_merge::prefer_primary_string_errors(workflow_res, restore_res)?;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

pub fn build_agent(shared: &SharedOpts, workflow: WorkflowCliOptions) -> AgentClient {
    AgentClient::new(
        shared.model.clone(),
        malvin::acp::AgentIoOptions {
            force: workflow.force,
            no_tee: shared.no_tee,
            raw_output: false,
        },
    )
}

/// `malvin code` / `malvin kpop` need `kiss` on `PATH`; check before stdout styling or async work.
fn require_kiss_for_cli_command(cmd: &Commands) -> Result<(), String> {
    match cmd {
        Commands::Code(_) => require_kiss_for_malvin("code"),
        Commands::Kpop(_) => require_kiss_for_malvin("kpop"),
        Commands::Do(_) | Commands::Init(_) | Commands::Models(_) => Ok(()),
    }
}

fn tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime")
}

pub fn entrypoint() -> Exit {
    malvin::invocation::init_from_env();
    let cli = Cli::parse();
    if let Err(e) = require_kiss_for_cli_command(&cli.command) {
        print_stderr_line(MALVIN_WHO, &e);
        return Exit::Failure;
    }
    malvin::output::init_stdout_style(cli.global.no_color);
    let res = match cli.command {
        Commands::Code(code) => {
            let workflow = WorkflowCliOptions {
                force: !code.shared.no_force,
                run_learn: !code.no_learn,
            };
            tokio_runtime().block_on(run_code(code, workflow))
        }
        Commands::Kpop(kpop) => {
            let workflow = WorkflowCliOptions {
                force: !kpop.shared.no_force,
                run_learn: !kpop.no_learn,
            };
            tokio_runtime().block_on(run_kpop(kpop, workflow))
        }
        Commands::Do(do_cmd) => {
            let workflow = WorkflowCliOptions {
                force: !do_cmd.shared.no_force,
                run_learn: false,
            };
            tokio_runtime().block_on(run_do(do_cmd, workflow))
        }
        Commands::Init(init) => init_cmd::run_init(init.path, init.force, &init.languages),
        Commands::Models(_) => models_cmd::run_models(),
    };
    match res {
        Ok(()) => Exit::Success,
        Err(e) => {
            print_stderr_line(MALVIN_WHO, &e);
            Exit::Failure
        }
    }
}
