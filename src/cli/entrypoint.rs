use super::models_cmd;
use super::{
    Commands, Exit, SharedOpts, WorkflowCliOptions, run_do, run_kpop, run_tidy,
};

/// Commands that accept `--name` acquire a session name lock before substantive work.
/// Only bare `malvin REQUEST` (resolved kpop), `do`, `code`, `tidy`, and `delight` accept `--name`.
pub(crate) const fn command_accepts_session_name(command: &Commands, bare_invoke: bool) -> bool {
    match command {
        Commands::Do(_)
        | Commands::Code(_)
        | Commands::Tidy(_)
        | Commands::Delight(_) => true,
        Commands::Kpop(_) => bare_invoke,
        _ => false,
    }
}

pub(crate) const fn unsupported_name_error(command: &Commands, bare_invoke: bool) -> Option<&'static str> {
    if command_accepts_session_name(command, bare_invoke) {
        return None;
    }
    Some(
        "`--name` is only supported for bare `malvin REQUEST`, `do`, `code`, `tidy`, and `delight`",
    )
}

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
        | Commands::Logs(_)
        | Commands::GenerateScript(_)
        | Commands::Delight(_)
        | Commands::Explain(_)
        | Commands::Revise(_) => Ok(()),
    }
}

#[path = "entrypoint_from.rs"]
mod entrypoint_from;
pub use entrypoint_from::entrypoint_from;

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

pub(crate) fn finish_entrypoint(res: Result<(), String>) -> Exit {
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

pub(crate) fn prepare_cli_output(global: &crate::cli::args::GlobalOpts) {
    let theme = std::env::current_dir()
        .ok()
        .map(|cwd| crate::malvin_config_file::load_malvin_config(&cwd).theme)
        .unwrap_or_default();
    crate::terminal_palette::init_terminal_theme(theme);
    crate::output::init_stdout_style(global.no_color);
    crate::output::set_stdout_suppressed(global.background);
}

pub(crate) fn dispatch_command(
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
        }
        cmd @ (Commands::Delight(_) | Commands::Explain(_) | Commands::Revise(_)) => {
            super::entrypoint_commands::dispatch_plan_authoring_gate(cmd, &mut shared, matches)
        }
        Commands::Do(do_cmd) => run_async_cli(|| {
            run_do(
                do_cmd,
                &shared,
                WorkflowCliOptions {
                    force: !shared.no_force,
                },
            )
        }),
        Commands::Inspire(inspire) => super::entrypoint_commands::run_inspire_command(inspire, &shared),
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
        cmd @ (Commands::Models(_) | Commands::Logs(_) | Commands::GenerateScript(_)) => {
            dispatch_agent_free(cmd)
        }
    }
}

fn dispatch_agent_free(command: Commands) -> Result<(), String> {
    match command {
        Commands::Models(models) => {
            if models.mini {
                run_async_cli(models_cmd::run_mini_models)
            } else {
                models_cmd::run_models(models)
            }
        }
        Commands::Logs(logs) => super::logs_cmd::run_logs(logs),
        Commands::GenerateScript(args) => super::generate_script_cmd::run_generate_script(args),
        _ => Err("dispatch_agent_free: unexpected command".to_string()),
    }
}

#[cfg(test)]
#[path = "entrypoint_tenacious_tests.rs"]
mod entrypoint_tenacious_tests;

#[cfg(test)]
#[path = "entrypoint_doc_tests.rs"]
mod entrypoint_doc_tests;

#[cfg(test)]
#[path = "entrypoint_name_tests.rs"]
mod entrypoint_name_tests;

#[cfg(test)]
#[path = "entrypoint_name_unix_tests.rs"]
mod entrypoint_name_unix_tests;
