//! CLI entry: `malvin plan.md` (Rust port of Python malvin).

use std::path::PathBuf;

use clap::Parser;
use malvin::agent::AgentClient;
use malvin::artifacts::create_run_artifacts;
use malvin::orchestrator::{Orchestrator, WorkflowConfig, WorkflowError};
use malvin::prompts::{PromptError, PromptStore};

#[derive(Parser, Debug)]
#[command(name = "malvin", version, about = "Implementation and review workflow via agent acp")]
#[allow(clippy::struct_excessive_bools)]
struct Args {
    /// Path to the plan file (copied into `_malvin/.../plan.md`).
    plan_path: PathBuf,
    /// Model label (passed to `agent --model`, same role as Python `--model` for cursor-agent).
    #[arg(long, default_value = "opus-4.5")]
    model: String,
    /// Disable force-mode compatibility flag (default: force on, matching Python malvin).
    #[arg(long, default_value_t = false)]
    no_force: bool,
    #[arg(long, default_value = "5")]
    max_loops: usize,
    #[arg(long, default_value_t = false)]
    tee: bool,
    /// Skip the learn phase (`learn.md`).
    #[arg(long, default_value_t = false)]
    no_learn: bool,
    #[arg(long, default_value_t = false)]
    tee_json: bool,
}

fn main() -> Exit {
    let args = Args::parse();
    let force = !args.no_force;
    let run_learn = !args.no_learn;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    match rt.block_on(run_async(
        args,
        WorkflowCliOptions {
            force,
            run_learn,
        },
    )) {
        Ok(()) => Exit::Success,
        Err(e) => {
            eprintln!("{e}");
            Exit::Failure
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct WorkflowCliOptions {
    force: bool,
    run_learn: bool,
}

fn prepare_prompt_store(workflow: WorkflowCliOptions) -> Result<PromptStore, String> {
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

fn echo_teed_plan(
    artifacts: &malvin::artifacts::RunArtifacts,
    args: &Args,
) -> Result<(), String> {
    if args.tee && !args.tee_json {
        let plan_text = std::fs::read_to_string(&artifacts.plan_path)
            .map_err(|e| e.to_string())?;
        print!("{plan_text}");
        if !plan_text.ends_with('\n') {
            println!();
        }
    }
    Ok(())
}

async fn run_async(args: Args, workflow: WorkflowCliOptions) -> Result<(), String> {
    let store = prepare_prompt_store(workflow)?;

    let mut client = AgentClient::new(
        args.model.clone(),
        malvin::agent::AgentIoOptions {
            force: workflow.force,
            tee: args.tee,
            tee_json: args.tee_json,
        },
    );
    client
        .ensure_authenticated()
        .map_err(|e| e.to_string())?;

    let artifacts = create_run_artifacts(&args.plan_path, None)
        .map_err(|e| e.to_string())?;

    echo_teed_plan(&artifacts, &args)?;

    println!("Logs: {}", format_logs_dir(&artifacts.run_dir)?);

    let mut orch = Orchestrator {
        client: &mut client,
        prompts: &store,
        artifacts: &artifacts,
        config: WorkflowConfig {
            max_loops: args.max_loops,
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

fn format_logs_dir(run_dir: &std::path::Path) -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let cwd_abs = cwd.canonicalize().map_err(|e| e.to_string())?;
    let run_abs = run_dir.canonicalize().map_err(|e| e.to_string())?;
    Ok(run_abs.strip_prefix(&cwd_abs).map_or_else(
        |_| run_abs.display().to_string(),
        |p| format!("./{}", p.display()),
    ))
}

#[derive(Debug)]
enum Exit {
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
        let _ = stringify!(super::Args);
        let _ = stringify!(super::Exit);
        let _ = stringify!(super::WorkflowCliOptions);
        let _ = stringify!(super::main);
        let _ = stringify!(super::run_async);
        let _ = stringify!(super::prepare_prompt_store);
        let _ = stringify!(super::echo_teed_plan);
        let _ = stringify!(super::format_logs_dir);
    }
}
