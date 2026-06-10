use super::{
    dispatch_command, finish_entrypoint, prepare_cli_output, print_command_error,
    require_kiss_for_cli_command, Commands, Exit,
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

fn entrypoint_before_dispatch(cli: &Cli) -> Option<Exit> {
    if cli.command.is_none() && !cli.shared.doc {
        let _ = crate::cli::commands_help::print_commands_only_help();
        return Some(Exit::Success);
    }
    if cli.shared.doc {
        return Some(match crate::cli::command_docs::print_doc(cli.command.as_ref()) {
            Ok(()) => Exit::Success,
            Err(e) => {
                print_command_error(&e);
                Exit::Failure
            }
        });
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

fn entrypoint_acquire_session(opt_name: Option<&str>) -> Result<crate::SessionNameGuard, Exit> {
    crate::acquire_session_name(opt_name)
        .map(|(_name, guard)| guard)
        .map_err(|e| {
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
    if let Some(exit) = entrypoint_preflight(&command) {
        return exit;
    }
    let _session_name_guard = match entrypoint_acquire_session(cli.shared.name.as_deref()) {
        Ok(guard) => guard,
        Err(exit) => return exit,
    };
    finish_entrypoint(dispatch_command(command, &cli.shared, &matches))
}
