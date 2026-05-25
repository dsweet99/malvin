use clap::{CommandFactory, Parser};

use super::models_cmd;
use super::{
    Cli, CodeArgs, Commands, Exit, SharedOpts, WorkflowCliOptions, run_do, run_ideas, run_kpop,
    run_tidy,
};

pub fn require_kiss_for_cli_command(cmd: &Commands) -> Result<(), String> {
    use crate::require_kiss_for_malvin;
    match cmd {
        Commands::Code(_) => require_kiss_for_malvin("code"),
        Commands::Tidy(_) => require_kiss_for_malvin("tidy"),
        Commands::Do(_)
        | Commands::Init(_)
        | Commands::Kpop(_)
        | Commands::Invent(_)
        | Commands::Models(_) => Ok(()),
    }
}

use super::entrypoint_checks::ensure_malvin_checks_for_command;

pub fn print_command_error(message: &str) {
    use crate::output::{MALVIN_WHO, print_log_error, print_stderr_line};
    use crate::repo_checks::{
        GATE_FAILURE_MARKER, is_gate_failure_error, is_pure_gate_failure_summary,
    };
    if is_pure_gate_failure_summary(message) {
        return;
    }
    if is_gate_failure_error(message) {
        let display = message.replace(GATE_FAILURE_MARKER, "");
        print_stderr_line(MALVIN_WHO, &display);
        return;
    }
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
    entrypoint_from(std::env::args_os())
}

pub fn entrypoint_from(
    args: impl IntoIterator<Item = impl Into<std::ffi::OsString> + Clone>,
) -> Exit {
    crate::init_from_env();
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => {
            use clap::error::ErrorKind;
            let exit = match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => Exit::Success,
                _ => Exit::Failure,
            };
            let _ = e.print();
            return exit;
        }
    };
    crate::output::init_stdout_style(cli.global.no_color);
    if cli.command.is_none() && !cli.shared.doc {
        let mut cmd = Cli::command();
        let _ = cmd.print_help();
        return Exit::Success;
    }
    if cli.shared.doc {
        return match super::command_docs::print_doc(cli.command.as_ref()) {
            Ok(()) => Exit::Success,
            Err(e) => {
                print_command_error(&e);
                Exit::Failure
            }
        };
    }
    let command = cli.command.expect("subcommand when not --doc-only");
    if let Err(e) = require_kiss_for_cli_command(&command) {
        print_command_error(&e);
        return Exit::Failure;
    }
    if let Err(e) = ensure_malvin_checks_for_command(&command) {
        print_command_error(&e);
        return Exit::Failure;
    }
    let res = dispatch_command(command, &cli.shared);
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

fn dispatch_command(command: Commands, shared: &SharedOpts) -> Result<(), String> {
    match command {
        Commands::Code(code) => run_code_command(code, shared),
        Commands::Kpop(kpop) => {
            let run_learn = !kpop.no_learn;
            run_async_cli(|| {
                run_kpop(
                    kpop.clone(),
                    shared,
                    WorkflowCliOptions {
                        force: !shared.no_force,
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
                    shared,
                    WorkflowCliOptions {
                        force: !shared.no_force,
                        run_learn,
                    },
                )
            })
        }
        Commands::Do(do_cmd) => run_async_cli(|| {
            run_do(
                do_cmd,
                shared,
                WorkflowCliOptions {
                    force: !shared.no_force,
                    run_learn: false,
                },
            )
        }),
        Commands::Invent(ideas) => run_invent_command(ideas, shared),
        Commands::Init(init) => {
            let shared = shared.clone();
            let tee = shared.tee_startup_stdout();
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

fn run_invent_command(
    ideas: crate::ideas_flow::IdeasArgs,
    shared: &SharedOpts,
) -> Result<(), String> {
    run_async_cli(|| {
        run_ideas(
            ideas,
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
                run_learn: false,
            },
        )
    })
}

fn run_code_command(mut code: CodeArgs, shared: &SharedOpts) -> Result<(), String> {
    if code.fast {
        code.skip_pre_checks = true;
        code.trust_the_plan = true;
    }
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

#[cfg(test)]
mod entrypoint_doc_tests {
    use super::{Exit, entrypoint_from};

    #[test]
    fn entrypoint_from_doc_argv_exits_success() {
        assert_eq!(entrypoint_from(["malvin", "--doc"]), Exit::Success);
    }

    #[test]
    fn entrypoint_from_bare_malvin_exits_success() {
        assert_eq!(entrypoint_from(["malvin"]), Exit::Success);
    }
}
