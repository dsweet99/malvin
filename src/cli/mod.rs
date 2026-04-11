//! Subcommand dispatch and async entrypoints for the `malvin` binary.

mod args;
mod kpop_flow;
mod shared_opts;

pub use args::{Cli, CodeArgs, Commands, KpopArgs};
pub use shared_opts::SharedOpts;

use clap::Parser;

use std::path::Path;

fn full_command_line() -> String {
    std::env::args().collect::<Vec<_>>().join(" ")
}

pub fn emit_command_line(run_dir: &Path) {
    let cmd = full_command_line();
    println!("Command: {cmd}");
    let log_path = run_dir.join("command.log");
    let _ = std::fs::write(&log_path, format!("{cmd}\n"));
}

use malvin::agent::AgentClient;
pub use kpop_flow::run_kpop;

use malvin::artifacts::{create_run_artifacts_from_text, resolve_user_request};
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

    echo_primary_to_stdout(&artifacts.plan_path, code.shared.primary_doc_plain_echo())?;

    emit_command_line(&artifacts.run_dir);
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
        malvin::agent::AgentIoOptions {
            force: workflow.force,
            no_tee: shared.no_tee,
        },
    )
}

pub fn format_logs_dir(run_dir: &Path) -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let cwd_abs = cwd.canonicalize().map_err(|e| e.to_string())?;
    let run_abs = run_dir.canonicalize().map_err(|e| e.to_string())?;
    Ok(run_abs.strip_prefix(&cwd_abs).map_or_else(
        |_| run_abs.display().to_string(),
        |p| format!("./{}", p.display()),
    ))
}

pub fn entrypoint() -> Exit {
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

#[cfg(test)]
mod kiss_tests {
    #[test]
    fn kiss_stringify_cli_symbols() {
        let _ = stringify!(super::shared_opts::SharedOpts);
        let _ = stringify!(super::Cli);
        let _ = stringify!(super::Commands);
        let _ = stringify!(super::CodeArgs);
        let _ = stringify!(super::KpopArgs);
        let _ = stringify!(super::SharedOpts);
        let _ = stringify!(super::Exit);
        let _ = stringify!(super::WorkflowCliOptions);
        let _ = stringify!(super::entrypoint);
        let _ = stringify!(super::run_code);
        let _ = stringify!(super::run_kpop);
        let _ = stringify!(super::prepare_prompt_store);
        let _ = stringify!(super::echo_primary_to_stdout);
        let _ = stringify!(super::emit_command_line);
        let _ = stringify!(super::format_logs_dir);
        let _ = stringify!(super::build_agent);
    }
}
