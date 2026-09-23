use super::{Commands, Exit, SharedOpts};

#[path = "entrypoint_from.rs"]
mod entrypoint_from;
#[path = "entrypoint_gates_only.rs"]
mod entrypoint_gates_only;
#[path = "entrypoint_dispatch.rs"]
mod entrypoint_dispatch;
#[path = "entrypoint_short_help.rs"]
mod entrypoint_short_help;
pub use entrypoint_from::entrypoint_from;
pub use entrypoint_dispatch::{
    DefaultRouteDispatch, dispatch_default_route, dispatch_do_workflow, dispatch_mixed_requests,
};
pub(crate) use entrypoint_gates_only::{GatesOnlyDispatch, dispatch_gates_only_route};

pub fn print_command_error(message: &str) {
    use crate::repo_checks::{
        GATE_FAILURE_MARKER, is_gate_failure_error, is_pure_gate_failure_summary,
    };
    use malvin::output::{MALVIN_WHO, print_log_error, print_stderr_line};
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
        malvin::malvin_sandbox::teardown_active_sandbox_for_interrupt();
        std::process::exit(130);
    });
}

pub fn entrypoint() -> Exit {
    if std::env::args().nth(1).as_deref() == Some(malvin::pi_sdk::INTERNAL_MANAGER_FLAG) {
        return match malvin::pi_sdk::run_local_llm_manager() {
            Ok(()) => Exit::Success,
            Err(e) => {
                eprintln!("local llm manager: {e}");
                Exit::Failure
            }
        };
    }
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

pub(crate) fn prepare_cli_output(_shared: &SharedOpts) {
    let theme = std::env::current_dir()
        .ok()
        .map(|cwd| malvin::malvin_config_file::load_malvin_config(&cwd).theme)
        .unwrap_or_default();
    malvin::terminal_palette::init_terminal_theme(theme);
    malvin::output::init_stdout_style();
    malvin::output::set_stdout_suppressed(false);
}

pub(crate) fn dispatch_command(
    command: Commands,
    model: &str,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let _ = matches;
    match command {
        Commands::Admin(admin) => super::run_admin(admin, model),
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
