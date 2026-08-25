use super::{
    Commands, Exit, SharedOpts, WorkflowCliOptions, run_do, run_router,
};
use crate::do_flow::DoArgs;

pub(crate) const fn command_accepts_session_name(_command: &Commands) -> bool {
    false
}

pub(crate) const fn unsupported_name_error() -> &'static str {
    "`--name` is only supported for bare `malvin REQUEST`, `--do`, and `malvin -g`"
}

#[path = "entrypoint_from.rs"]
mod entrypoint_from;
#[path = "entrypoint_gates_only.rs"]
mod entrypoint_gates_only;
#[path = "entrypoint_short_help.rs"]
mod entrypoint_short_help;
pub use entrypoint_from::entrypoint_from;
pub(crate) use entrypoint_gates_only::dispatch_gates_only_route;

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
    if super::error_run_log::command_error_already_emitted(message) {
        return;
    }
    super::error_run_log::note_command_error_emitted(message);
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
    rt.block_on(async {
        spawn_ctrl_c_teardown();
        f().await
    })
}

fn spawn_ctrl_c_teardown() {
    tokio::spawn(async {
        if tokio::signal::ctrl_c().await.is_err() {
            return;
        }
        crate::malvin_sandbox::teardown_active_sandbox_for_interrupt();
        std::process::exit(130);
    });
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
    crate::output::init_stdout_style();
    crate::output::set_stdout_suppressed(global.background);
}

pub(crate) fn dispatch_command(
    command: Commands,
    shared: &SharedOpts,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let mut shared = shared.clone();
    match command {
        cmd @ Commands::Write(_) => {
            super::entrypoint_commands::dispatch_plan_authoring_gate(cmd, &mut shared, matches)
        }
        Commands::Inspire(inspire) | Commands::Adaptix(inspire) => {
            super::entrypoint_commands::run_inspire_command(inspire, &shared)
        }
        Commands::Models(models) => dispatch_models(models, &shared),
    }
}

pub fn dispatch_do_workflow(do_args: DoArgs, shared: &SharedOpts) -> Result<(), String> {
    run_async_cli(|| {
        run_do(
            do_args,
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
            },
        )
    })
}

pub struct DefaultRouteDispatch<'a> {
    pub request: String,
    pub max_loops: usize,
    pub max_hypotheses: usize,
    pub shared: &'a mut SharedOpts,
    pub matches: &'a clap::ArgMatches,
}

pub fn dispatch_default_route(input: DefaultRouteDispatch<'_>) -> Result<(), String> {
    use crate::router_flow::RouterArgs;
    let DefaultRouteDispatch {
        request,
        mut max_loops,
        max_hypotheses,
        shared,
        matches,
    } = input;
    super::loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        shared.no_tenacious,
        matches,
    );
    run_async_cli(|| async {
        crate::cli::init_flow::maybe_run_init_bootstrap(
            crate::cli::init_flow::InitWorkflowOpts {
                max_loops,
                max_hypotheses,
            },
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
            },
        )
        .await?;
        run_router(
            RouterArgs {
                request: Some(request),
                max_loops,
                max_hypotheses,
            },
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
            },
        )
        .await
    })
}

fn dispatch_models(
    models: super::models_cmd::ModelsArgs,
    shared: &super::SharedOpts,
) -> Result<(), String> {
    let model = shared.model.canonical();
    super::models_cmd::run_models(models, &model)
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
