use super::{Commands, Exit, SharedOpts};

pub fn print_command_error(message: &str) {
    use malvin::output::print_log_error;
    if super::error_run_log::command_error_already_emitted(message) {
        return;
    }
    super::error_run_log::note_command_error_emitted(message);
    super::error_run_log::append_command_error_to_run_log(message);
    print_log_error(message);
}

pub fn print_cli_error(error: &super::CliError) {
    use malvin::output::{MALVIN_WHO, print_stderr_line};
    match error {
        super::CliError::GatesFailed(summary) => print_stderr_line(MALVIN_WHO, summary),
        super::CliError::Other(message) => print_command_error(message),
    }
}

pub fn try_tokio_runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("failed to create Tokio runtime: {e}"))
}

pub fn run_async_cli<F, Fut, E>(f: F) -> Result<(), E>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), E>> + Send,
    E: From<String>,
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

pub(crate) fn finish_entrypoint(res: Result<(), super::CliError>) -> Exit {
    match res {
        Ok(()) => {
            super::error_run_log::clear_command_error_run_dir();
            Exit::Success
        }
        Err(e) => {
            print_cli_error(&e);
            super::error_run_log::clear_command_error_run_dir();
            Exit::Failure
        }
    }
}

pub(crate) fn prepare_cli_output(shared: &SharedOpts) {
    malvin::output::set_dm_emit_enabled(!shared.no_emit);
    let loaded = malvin::config::malvin_config_file::load_malvin_config();
    malvin::output::set_dm_on_emit_command(loaded.dm_on_emit.clone());
    let theme = loaded.theme;
    malvin::terminal_palette::init_terminal_theme(theme);
    malvin::output::init_stdout_style();
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
