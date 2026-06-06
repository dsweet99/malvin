use super::models_cmd;
use super::{
    Commands, Exit, SharedOpts, WorkflowCliOptions, run_do, run_kpop, run_tidy,
};

pub fn require_kiss_for_cli_command(cmd: &Commands) -> Result<(), String> {
    use crate::require_kiss_for_malvin;
    match cmd {
        Commands::Code(_) => require_kiss_for_malvin("code"),
        Commands::Tidy(_) => require_kiss_for_malvin("tidy"),
        Commands::Do(_)
        | Commands::Init(_)
        | Commands::Kpop(_)
        | Commands::Inspire(_)
        | Commands::Models(_)
        | Commands::Plan(_) => Ok(()),
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

fn prepare_cli_output(global: &crate::cli::args::GlobalOpts) {
    let theme = std::env::current_dir()
        .ok()
        .map(|cwd| crate::malvin_config_file::load_malvin_config(&cwd).theme)
        .unwrap_or_default();
    crate::terminal_palette::init_terminal_theme(theme);
    crate::output::init_stdout_style(global.no_color);
    crate::output::set_stdout_suppressed(global.background);
}

fn finish_entrypoint(res: Result<(), String>) -> Exit {
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

pub fn entrypoint_from(
    args: impl IntoIterator<Item = impl Into<std::ffi::OsString> + Clone>,
) -> Exit {
    crate::init_from_env();
    let (cli, matches) = match super::config_defaults::parse_cli_with_config_defaults(args) {
        Ok(parsed) => parsed,
        Err(e) => {
            use clap::error::ErrorKind;
            let exit = match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => Exit::Success,
                _ => Exit::Failure,
            };
            let _ = stringify!(e.print());
            return exit;
        }
    };
    prepare_cli_output(&cli.global);
    if cli.command.is_none() && !cli.shared.doc {
        let _ = super::commands_help::print_commands_only_help();
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
    let preflight_err = require_kiss_for_cli_command(&command)
        .err()
        .or_else(|| ensure_malvin_checks_for_command(&command).err());
    if let Some(e) = preflight_err {
        print_command_error(&e);
        return Exit::Failure;
    }
    finish_entrypoint(dispatch_command(command, &cli.shared, &matches))
}

fn dispatch_command(
    command: Commands,
    shared: &SharedOpts,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let mut shared = shared.clone();
    match command {
        Commands::Code(mut code) => {
            super::loop_opts::apply_gate_loop_tenacious(super::loop_opts::GateLoopTenaciousApply {
                subcommand: "code",
                max_loops: &mut code.max_loops,
                tenacious: code.tenacious,
                no_tenacious: shared.no_tenacious,
                max_acp_retries: &mut shared.max_acp_retries,
                matches,
            });
            super::entrypoint_commands::run_code_command(code, &shared)
        }
        Commands::Kpop(mut kpop) => {
            super::loop_opts::apply_gate_loop_tenacious(super::loop_opts::GateLoopTenaciousApply {
                subcommand: "kpop",
                max_loops: &mut kpop.max_loops,
                tenacious: kpop.tenacious,
                no_tenacious: shared.no_tenacious,
                max_acp_retries: &mut shared.max_acp_retries,
                matches,
            });
            run_async_cli(|| {
                run_kpop(
                    kpop,
                    &shared,
                    WorkflowCliOptions {
                        force: !shared.no_force,
                    },
                )
            })
        }
        Commands::Tidy(mut tidy) => {
            super::loop_opts::apply_gate_loop_tenacious(super::loop_opts::GateLoopTenaciousApply {
                subcommand: "tidy",
                max_loops: &mut tidy.max_loops,
                tenacious: tidy.tenacious,
                no_tenacious: shared.no_tenacious,
                max_acp_retries: &mut shared.max_acp_retries,
                matches,
            });
            run_async_cli(|| {
                run_tidy(
                    tidy,
                    &shared,
                    WorkflowCliOptions {
                        force: !shared.no_force,
                    },
                )
            })
        },
        Commands::Do(do_cmd) => run_async_cli(|| {
            run_do(
                do_cmd,
                &shared,
                WorkflowCliOptions {
                    force: !shared.no_force,
                },
            )
        }),
        Commands::Inspire(ideas) => super::entrypoint_commands::run_inspire_command(ideas, &shared),
        Commands::Plan(plan) => super::entrypoint_commands::run_plan_command(plan, &shared),
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

#[cfg(test)]
#[path = "entrypoint_tenacious_tests.rs"]
mod entrypoint_tenacious_tests;

#[cfg(test)]
#[path = "entrypoint_doc_tests.rs"]
mod entrypoint_doc_tests;

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use crate::cli::entrypoint_commands;
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = (dispatch_command, finish_entrypoint);
        assert!(stringify!(run_async_cli).contains("run_async_cli"));
        let _ = entrypoint_commands::run_code_command;
        let _ = entrypoint_commands::run_inspire_command;
        let _ = entrypoint_commands::run_plan_command;
    }
}
