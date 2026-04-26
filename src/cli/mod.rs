mod args;
mod do_flow;
mod exit;
mod init_cmd;
mod kiss_clamp;
mod kpop_flow;
mod sync_flow;
mod tidy_flow;
#[cfg(test)]
mod command_log_tests;
#[cfg(test)]
mod markdown_flag_parse_tests;
mod models_cmd;
mod repo_checks;
mod run_emit;
mod shared_opts;
#[cfg(test)]
mod stringify_cov;
mod timing_merge;
pub use args::{Cli, CodeArgs, Commands, KpopArgs};
pub use exit::Exit;
pub use run_emit::emit_run_startup_sequence;
pub use shared_opts::SharedOpts;
use clap::Parser;
use malvin::env_path::require_kiss_for_malvin;
use malvin::output::{MALVIN_WHO, print_stderr_line, print_stdout_line};
pub use do_flow::run_do;
pub use sync_flow::run_sync;
pub use kpop_flow::run_kpop;
pub use tidy_flow::run_tidy;
use malvin::acp::AgentClient;
use malvin::artifacts::{
    RunArtifacts, backup_workspace_grounding_if_present, create_run_artifacts_from_text,
    resolve_user_request,
};
use malvin::orchestrator::{Orchestrator, WorkflowConfig, WorkflowError};
use malvin::prompts::{PromptError, PromptStore};
use sync_flow::SyncRunSpec;
#[derive(Debug, Clone, Copy)] pub struct WorkflowCliOptions { pub force: bool, pub run_learn: bool }
#[derive(Debug, Clone, Copy)] pub struct AgentStdoutTeeFlags { pub emit_stdout_markdown: bool, pub raw_output: bool, pub show_thoughts_on_stdout: bool }
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
pub fn prepare_kpop_prompt_store(workflow: WorkflowCliOptions, require_mbc2: bool) -> Result<PromptStore, String> {
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
fn prepare_code_run(code: &CodeArgs, shared: &SharedOpts, workflow: WorkflowCliOptions) -> Result<(PromptStore, AgentClient, RunArtifacts), String> {
    let store = prepare_prompt_store(workflow)?;
    let emit_stdout_markdown = shared.acp_stdout_markdown_enabled();
    let client = build_agent(shared, workflow, emit_stdout_markdown);
    let (text, work_dir) = resolve_user_request(&code.request)?;
    let artifacts = create_run_artifacts_from_text(&text, Some(work_dir.as_path()))
        .map_err(|e| e.to_string())?;
    Ok((store, client, artifacts))
}
pub async fn run_code(
    code: CodeArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let (store, mut client, artifacts) = prepare_code_run(&code, shared, workflow)?;
    repo_checks::run_repo_workspace_gates(
        &artifacts.work_dir,
        repo_checks::RepoGateOutput::Tagged,
    )?;
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let grounding_backup = backup_workspace_grounding_if_present(&artifacts.work_dir)?;
    emit_run_startup_sequence(&artifacts, shared.tee_startup_stdout(), &code.request)?;
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
    timing_merge::merge_acp_with_grounding_restore(
        workflow_res,
        &artifacts.work_dir,
        &grounding_backup,
    )?;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}
pub const fn agent_io_options(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    tee: AgentStdoutTeeFlags,
) -> malvin::acp::AgentIoOptions {
    malvin::acp::AgentIoOptions {
        force: workflow.force,
        no_tee: shared.no_tee,
        raw_output: tee.raw_output,
        show_thoughts_on_stdout: tee.show_thoughts_on_stdout,
        emit_stdout_markdown: tee.emit_stdout_markdown,
    }
}
pub fn build_agent(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    emit_stdout_markdown: bool,
) -> AgentClient {
    AgentClient::new(
        shared.model.clone(),
        agent_io_options(
            shared,
            workflow,
            AgentStdoutTeeFlags {
                emit_stdout_markdown,
                raw_output: false,
                show_thoughts_on_stdout: false,
            },
        ),
    )
}
fn require_kiss_for_cli_command(cmd: &Commands) -> Result<(), String> {
    match cmd {
        Commands::Code(_) | Commands::Tidy(_) => require_kiss_for_malvin("code"),
        Commands::Do(_)
        | Commands::Init(_)
        | Commands::Kpop(_)
        | Commands::Models(_)
        | Commands::Sync { .. } => Ok(()),
    }
}
fn print_command_error(message: &str) {
    if message.starts_with("ERR:") {
        eprintln!("{message}");
        return;
    }
    print_stderr_line(MALVIN_WHO, message);
}
fn try_tokio_runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("failed to create Tokio runtime: {e}"))
}
fn run_async_cli<F, Fut>(f: F) -> Result<(), String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), String>> + Send,
{
    let rt = try_tokio_runtime()?;
    rt.block_on(f())
}

pub fn entrypoint() -> Exit {
    malvin::invocation::init_from_env();
    let cli = Cli::parse();
    if let Err(e) = require_kiss_for_cli_command(&cli.command) {
        print_command_error(&e);
        return Exit::Failure;
    }
    malvin::output::init_stdout_style(cli.global.no_color);
    let res = match cli.command {
        Commands::Code(code) => {
            let workflow = WorkflowCliOptions {
                force: !cli.shared.no_force,
                run_learn: !code.no_learn,
            };
            run_async_cli(|| run_code(code, &cli.shared, workflow))
        }
        Commands::Kpop(kpop) => {
            let workflow = WorkflowCliOptions {
                force: !cli.shared.no_force,
                run_learn: !kpop.no_learn,
            };
            run_async_cli(|| run_kpop(kpop, &cli.shared, workflow))
        }
        Commands::Tidy(tidy) => {
            let workflow = WorkflowCliOptions {
                force: !cli.shared.no_force,
                run_learn: !tidy.no_learn,
            };
            run_async_cli(|| run_tidy(tidy, &cli.shared, workflow))
        }
        Commands::Do(do_cmd) => {
            let workflow = WorkflowCliOptions {
                force: !cli.shared.no_force,
                run_learn: false,
            };
            run_async_cli(|| run_do(do_cmd, &cli.shared, workflow))
        }
        Commands::Init(init) => init_cmd::run_init(init.path, init.force, &init.languages),
        Commands::Models(_) => models_cmd::run_models(),
        Commands::Sync {
            max_loops,
            no_learn,
        } => {
            let workflow = WorkflowCliOptions {
                force: !cli.shared.no_force,
                run_learn: !no_learn,
            };
            run_async_cli(|| {
                run_sync(
                    SyncRunSpec {
                        max_loops,
                        no_learn,
                    },
                    &cli.shared,
                    workflow,
                )
            })
        }
    };
    match res {
        Ok(()) => Exit::Success,
        Err(e) => {
            print_command_error(&e);
            Exit::Failure
        }
    }
}

