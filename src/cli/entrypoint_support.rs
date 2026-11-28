use super::{Commands, Exit, SharedOpts};

pub fn print_command_error(message: &str) {
    use crate::cli::repo_checks::{
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
    malvin::run_timing::arm_process_footnotes();
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
        malvin::agent_process::malvin_sandbox::teardown_active_sandbox_for_interrupt();
        malvin::run_timing::emit_process_footnotes_if_armed();
        std::process::exit(130);
    });
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

pub(crate) fn prepare_cli_output(shared: &SharedOpts) {
    malvin::output::set_dm_emit_enabled(!shared.no_emit);
    let loaded = std::env::current_dir()
        .ok()
        .map(|cwd| malvin::config::malvin_config_file::load_malvin_config(&cwd));
    malvin::output::set_dm_on_emit_command(loaded.as_ref().and_then(|cfg| cfg.dm_on_emit.clone()));
    let theme = loaded.map(|cfg| cfg.theme).unwrap_or_default();
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
