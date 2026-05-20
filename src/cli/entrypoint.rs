use clap::Parser;

use super::{
    Cli, CodeArgs, Commands, Exit, SharedOpts, WorkflowCliOptions, run_bug, run_do, run_kpop,
    run_plan, run_tidy,
};
use super::models_cmd;

pub fn require_kiss_for_cli_command(cmd: &Commands) -> Result<(), String> {
    use crate::require_kiss_for_malvin;
    match cmd {
        Commands::Code(_) => require_kiss_for_malvin("code"),
        Commands::Tidy(_) => require_kiss_for_malvin("tidy"),
        Commands::Plan(_) => require_kiss_for_malvin("plan"),
        Commands::Bug(_) => require_kiss_for_malvin("bug"),
        Commands::Do(_) | Commands::Init(_) | Commands::Kpop(_) | Commands::Models(_) => Ok(()),
    }
}

pub fn print_command_error(message: &str) {
    use crate::output::print_log_error;
    super::error_run_log::append_command_error_to_run_log(message);
    print_log_error(message);
}

pub fn try_tokio_runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("failed to create Tokio runtime: {e}"))
}

pub fn run_async_cli<F, Fut>(f: F) -> Result<(), String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), String>> + Send,
{
    let rt = try_tokio_runtime()?;
    rt.block_on(f())
}

pub fn entrypoint() -> Exit {
    crate::init_from_env();
    let cli = Cli::parse();
    if let Err(e) = require_kiss_for_cli_command(&cli.command) {
        print_command_error(&e);
        return Exit::Failure;
    }
    crate::output::init_stdout_style(cli.global.no_color);
    let res = dispatch_command(cli);
    match res {
        Ok(()) => {
            super::error_run_log::clear_command_error_run_dir();
            Exit::Success
        }
        Err(e) => {
            print_command_error(&e);
            super::error_run_log::clear_command_error_run_dir();
            Exit::Failure
        }
    }
}

fn dispatch_command(cli: Cli) -> Result<(), String> {
    match cli.command {
        Commands::Code(code) => run_code_command(code, &cli.shared),
        Commands::Kpop(kpop) => {
            let run_learn = !kpop.no_learn;
            run_async_cli(|| {
                run_kpop(
                    kpop.clone(),
                    &cli.shared,
                    WorkflowCliOptions {
                        force: !cli.shared.no_force,
                        run_learn,
                    },
                )
            })
        }
        Commands::Bug(bug) => {
            let run_learn = !bug.no_learn;
            run_async_cli(|| {
                run_bug(
                    bug.clone(),
                    &cli.shared,
                    WorkflowCliOptions {
                        force: !cli.shared.no_force,
                        run_learn,
                    },
                )
            })
        }
        Commands::Tidy(tidy) => {
            let run_learn = !tidy.no_learn;
            run_async_cli(|| {
                run_tidy(
                    tidy.clone(),
                    &cli.shared,
                    WorkflowCliOptions {
                        force: !cli.shared.no_force,
                        run_learn,
                    },
                )
            })
        }
        Commands::Plan(plan) => run_async_cli(|| {
            run_plan(
                plan,
                &cli.shared,
                WorkflowCliOptions {
                    force: !cli.shared.no_force,
                    run_learn: false,
                },
            )
        }),
        Commands::Do(do_cmd) => run_async_cli(|| {
            run_do(
                do_cmd,
                &cli.shared,
                WorkflowCliOptions {
                    force: !cli.shared.no_force,
                    run_learn: false,
                },
            )
        }),
        Commands::Init(init) => {
            let shared = cli.shared.clone();
            let tee = cli.shared.tee_startup_stdout();
            run_async_cli(|| async move {
                crate::init_cmd::run_init(crate::init_cmd::RunInitRequest {
                    path: init.path,
                    languages: &init.languages,
                    shared: &shared,
                    opts: crate::init_cmd::RunInitOptions {
                        overwrite_templates: init.force,
                        tee_startup_stdout: tee,
                    },
                })
                .await
            })
        }
        Commands::Models(_) => models_cmd::run_models(),
    }
}

fn run_code_command(code: CodeArgs, shared: &SharedOpts) -> Result<(), String> {
    let run_learn = !code.no_learn;
    run_async_cli(|| {
        super::run_code(
            code,
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
                run_learn,
            },
        )
    })
}

