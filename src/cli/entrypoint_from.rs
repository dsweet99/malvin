use super::{
    command_accepts_session_name, dispatch_command, finish_entrypoint, prepare_cli_output,
    print_command_error, require_kiss_for_cli_command, unsupported_name_error, Commands, Exit,
};
use crate::cli::args::Cli;
use crate::cli::entrypoint_checks::ensure_malvin_checks_for_command;

fn parse_cli_args_or_exit(
    args: impl IntoIterator<Item = impl Into<std::ffi::OsString> + Clone>,
) -> Result<(Cli, clap::ArgMatches), Exit> {
    match crate::cli::config_defaults::parse_cli_with_config_defaults(args) {
        Ok(parsed) => Ok(parsed),
        Err(e) => {
            use clap::error::ErrorKind;
            let exit = match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => Exit::Success,
                _ => Exit::Failure,
            };
            let _ = e.print();
            Err(exit)
        }
    }
}

fn entrypoint_short_help_when_request_missing(
    doc: bool,
    request: Option<&String>,
    subcommand: &str,
) -> Option<Exit> {
    if doc || request.is_some() {
        return None;
    }
    let _ = crate::cli::commands_help::print_subcommand_short_help(subcommand);
    Some(Exit::Success)
}

fn entrypoint_request_missing_short_help(cli: &Cli) -> Option<Exit> {
    let command = cli.command.as_ref()?;
    let (request, subcommand) = match command {
        Commands::Code(code) => (code.request.as_ref(), "code"),
        Commands::Inspire(inspire) => (inspire.request.as_ref(), "inspire"),
        Commands::Explain(explain) => (explain.request.as_ref(), "explain"),
        _ => return None,
    };
    entrypoint_short_help_when_request_missing(cli.shared.doc, request, subcommand)
}

fn entrypoint_doc_exit(cli: &Cli) -> Exit {
    match crate::cli::command_docs::print_doc(cli.command.as_ref()) {
        Ok(()) => Exit::Success,
        Err(e) => {
            print_command_error(&e);
            Exit::Failure
        }
    }
}

fn entrypoint_before_dispatch(cli: &Cli) -> Option<Exit> {
    if cli.command.is_none() && !cli.shared.doc {
        let _ = crate::cli::commands_help::print_commands_only_help();
        return Some(Exit::Success);
    }
    if let Some(exit) = entrypoint_request_missing_short_help(cli) {
        return Some(exit);
    }
    if cli.shared.doc {
        return Some(entrypoint_doc_exit(cli));
    }
    None
}

fn entrypoint_preflight(command: &Commands) -> Option<Exit> {
    require_kiss_for_cli_command(command)
        .err()
        .or_else(|| ensure_malvin_checks_for_command(command).err())
        .map(|e| {
            print_command_error(&e);
            Exit::Failure
        })
}

pub fn entrypoint_from(
    args: impl IntoIterator<Item = impl Into<std::ffi::OsString> + Clone>,
) -> Exit {
    crate::init_from_env();
    let (cli, matches) = match parse_cli_args_or_exit(args) {
        Ok(parsed) => parsed,
        Err(exit) => return exit,
    };
    prepare_cli_output(&cli.global);
    if let Some(exit) = entrypoint_before_dispatch(&cli) {
        return exit;
    }
    let command = cli.command.expect("subcommand when not --doc-only");
    let bare_invoke = cli.bare_request.is_some();
    if cli.shared.name.is_some() {
        if let Some(message) = unsupported_name_error(&command, bare_invoke) {
            print_command_error(message);
            return Exit::Failure;
        }
    }
    if let Some(exit) = entrypoint_preflight(&command) {
        return exit;
    }
    let _session_name_guard = if command_accepts_session_name(&command, bare_invoke) {
        match crate::acquire_session_name(cli.shared.name.as_deref()) {
            Ok((session_name, guard)) => {
                crate::set_active_acp_lock_slot(session_name);
                Some(guard)
            }
            Err(e) => {
                print_command_error(&e);
                return Exit::Failure;
            }
        }
    } else {
        None
    };
    finish_entrypoint(dispatch_command(command, &cli.shared, &matches))
}
