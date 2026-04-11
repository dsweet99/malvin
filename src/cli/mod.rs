//! Subcommand dispatch and async entrypoints for the `malvin` binary.

mod args;
#[cfg(all(test, unix))]
mod command_log_tests;
mod init_cmd;
mod kpop_flow;
mod models_cmd;
mod shared_opts;

pub use args::{Cli, CodeArgs, Commands, KpopArgs};
pub use shared_opts::SharedOpts;

use clap::Parser;

use std::path::Path;

/// Writes `command.log` under `run_dir`. When `echo_stdout` is true (tee on), also prints `Command: …` to stdout — same flag semantics as [`SharedOpts::tee_startup_stdout`].
pub fn emit_command_line(run_dir: &Path, echo_stdout: bool) -> Result<(), String> {
    malvin::invocation::init_from_env();
    let cmd = malvin::invocation::command_line()
        .expect("init_from_env populates argv via OnceLock");
    let line = format!("Command: {cmd}");
    if echo_stdout {
        println!("{line}");
    }
    let log_path = run_dir.join("command.log");
    std::fs::write(&log_path, format!("{line}\n")).map_err(|e| format!("command.log: {e}"))?;
    Ok(())
}

use malvin::acp::AgentClient;
pub use kpop_flow::run_kpop;

use malvin::artifacts::{create_run_artifacts_from_text, resolve_user_request};
use malvin::log_paths::format_logs_dir;
use malvin::orchestrator::{Orchestrator, WorkflowConfig, WorkflowError};
use malvin::prompts::{PromptError, PromptStore};

#[derive(Debug, Clone, Copy)]
pub struct WorkflowCliOptions {
    pub force: bool,
    pub run_learn: bool,
}

pub fn prepare_prompt_store(workflow: WorkflowCliOptions) -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store
        .ensure_defaults()
        .map_err(|e: PromptError| e.0)?;
    store.validate_required().map_err(|e: PromptError| e.0)?;
    if workflow.run_learn {
        store.validate_exists("learn.md").map_err(|e: PromptError| e.0)?;
    }
    Ok(store)
}

/// Like [`prepare_prompt_store`] but only checks prompts used by `malvin kpop` (not the full workflow set).
pub fn prepare_kpop_prompt_store(
    workflow: WorkflowCliOptions,
    p_creative: f64,
) -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store
        .ensure_defaults()
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_kpop_prompts(workflow.run_learn, p_creative)
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn echo_primary_to_stdout(plan_path: &Path, echo_plain: bool) -> Result<(), String> {
    if !echo_plain {
        return Ok(());
    }
    let plan_text = std::fs::read_to_string(plan_path).map_err(|e| e.to_string())?;
    print!("{plan_text}");
    if !plan_text.ends_with('\n') {
        println!();
    }
    Ok(())
}

pub async fn run_code(code: CodeArgs, workflow: WorkflowCliOptions) -> Result<(), String> {
    let store = prepare_prompt_store(workflow)?;

    let mut client = build_agent(&code.shared, workflow);
    client
        .ensure_authenticated()
        .map_err(|e| e.to_string())?;

    let (text, work_dir) = resolve_user_request(&code.request)?;
    let artifacts =
        create_run_artifacts_from_text(&text, Some(work_dir.as_path())).map_err(|e| e.to_string())?;

    echo_primary_to_stdout(&artifacts.plan_path, code.shared.tee_startup_stdout())?;

    emit_command_line(&artifacts.run_dir, code.shared.tee_startup_stdout())?;
    println!("Logs: {}", format_logs_dir(&artifacts.run_dir)?);

    let mut orch = Orchestrator {
        client: &mut client,
        prompts: &store,
        artifacts: &artifacts,
        config: WorkflowConfig {
            max_loops: code.max_loops,
            run_learn: workflow.run_learn,
        },
        progress_callback: Box::new(|msg: &str| {
            println!("{msg}");
        }),
    };
    orch.run()
        .await
        .map_err(|e: WorkflowError| e.0)?;
    println!("DONE");
    Ok(())
}

pub fn build_agent(shared: &SharedOpts, workflow: WorkflowCliOptions) -> AgentClient {
    AgentClient::new(
        shared.model.clone(),
        malvin::acp::AgentIoOptions {
            force: workflow.force,
            no_tee: shared.no_tee,
        },
    )
}

pub fn entrypoint() -> Exit {
    malvin::invocation::init_from_env();
    let cli = Cli::parse();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let res = match cli.command {
        Commands::Code(code) => {
            let workflow = WorkflowCliOptions {
                force: !code.shared.no_force,
                run_learn: !code.no_learn,
            };
            rt.block_on(run_code(code, workflow))
        }
        Commands::Kpop(kpop) => {
            let workflow = WorkflowCliOptions {
                force: !kpop.shared.no_force,
                run_learn: !kpop.no_learn,
            };
            rt.block_on(run_kpop(kpop, workflow))
        }
        Commands::Init(init) => init_cmd::run_init(init.path, init.force),
        Commands::Models(_) => models_cmd::run_models(),
    };
    match res {
        Ok(()) => Exit::Success,
        Err(e) => {
            eprintln!("{e}");
            Exit::Failure
        }
    }
}

#[derive(Debug)]
pub enum Exit {
    Success,
    Failure,
}

impl std::process::Termination for Exit {
    fn report(self) -> std::process::ExitCode {
        match self {
            Self::Success => std::process::ExitCode::SUCCESS,
            Self::Failure => std::process::ExitCode::from(1),
        }
    }
}

#[test]
fn kiss_stringify_cli_symbols() {
    let _ = stringify!(crate::cli::shared_opts::SharedOpts);
    let _ = stringify!(crate::cli::Cli);
    let _ = stringify!(crate::cli::Commands);
    let _ = stringify!(crate::cli::CodeArgs);
    let _ = stringify!(crate::cli::init_cmd::InitArgs);
    let _ = stringify!(crate::cli::models_cmd::ModelsArgs);
    let _ = stringify!(crate::cli::KpopArgs);
    let _ = stringify!(crate::cli::SharedOpts);
    let _ = stringify!(crate::cli::Exit);
    let _ = stringify!(crate::cli::WorkflowCliOptions);
    let _ = stringify!(crate::cli::entrypoint);
    let _ = stringify!(crate::cli::run_code);
    let _ = stringify!(crate::cli::run_kpop);
    let _ = stringify!(crate::cli::prepare_prompt_store);
    let _ = stringify!(crate::cli::prepare_kpop_prompt_store);
    let _ = stringify!(crate::cli::echo_primary_to_stdout);
    let _ = stringify!(crate::cli::emit_command_line);
    let _ = stringify!(malvin::log_paths::format_logs_dir);
    let _ = stringify!(crate::cli::build_agent);
    let _ = stringify!(crate::cli::shared_opts::SharedOpts::tee_startup_stdout);
    let _ = stringify!(crate::cli::init_cmd::run_init);
    let _ = stringify!(crate::cli::models_cmd::run_models);
    let _ = stringify!(malvin::env_path::lookup_bin_on_path);
}

