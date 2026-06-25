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
        Commands::Code(code) => (code.requests.first(), "code"),
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
    if cli.command.is_none() && cli.bare_args.is_empty() && !cli.shared.doc {
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

fn entrypoint_acquire_session(opt_name: Option<&str>) -> Result<(String, crate::SessionNameGuard), Exit> {
    crate::acquire_session_name(opt_name).map_err(|e| {
        print_command_error(&e);
        Exit::Failure
    })
}

fn entrypoint_sequential_bare_kpop(cli: &Cli, matches: &clap::ArgMatches) -> Option<Exit> {
    if cli.command.is_none() && cli.bare_args.len() > 1 {
        Some(finish_entrypoint(
            crate::cli::entrypoint_commands::run_bare_sequential_kpop(&cli, matches, &cli.shared),
        ))
    } else {
        None
    }
}

fn entrypoint_validate_name(cli: &Cli, command: &Commands, bare_invoke: bool) -> Option<Exit> {
    cli.shared.name.as_ref()?;
    unsupported_name_error(command, bare_invoke).map(|message| {
        print_command_error(message);
        Exit::Failure
    })
}

fn entrypoint_sweep_stale_acp_spawn_locks() {
    let Ok(cwd) = std::env::current_dir() else {
        return;
    };
    if !crate::is_malvin_workspace(&cwd) {
        return;
    }
    let chamber = cwd.join(".malvin").join("acp_spawn");
    if !chamber.is_dir() {
        return;
    }
    if let Err(e) = crate::acp_spawn_sweep::sweep_stale_acp_spawn_locks(&cwd) {
        tracing::warn!(
            target: "malvin::entrypoint",
            error = %e,
            "stale ACP spawn lock sweep failed; continuing"
        );
    }
}

fn run_entrypoint(cli: Cli, matches: clap::ArgMatches) -> Exit {
    prepare_cli_output(&cli.global);
    if let Some(exit) = entrypoint_before_dispatch(&cli) {
        return exit;
    }
    entrypoint_sweep_stale_acp_spawn_locks();
    if let Some(exit) = entrypoint_sequential_bare_kpop(&cli, &matches) {
        return exit;
    }
    let bare_invoke = cli.bare_args.len() == 1;
    let command_ref = cli.command.as_ref().expect("subcommand when not --doc-only");
    if let Some(exit) = entrypoint_validate_name(&cli, command_ref, bare_invoke) {
        return exit;
    }
    if let Some(exit) = entrypoint_preflight(command_ref) {
        return exit;
    }
    if command_accepts_session_name(command_ref, bare_invoke) {
        let _session_name_guard = match entrypoint_acquire_session(cli.shared.name.as_deref()) {
            Ok((session_name, guard)) => {
                crate::set_active_acp_lock_slot(session_name);
                guard
            }
            Err(exit) => return exit,
        };
    }
    let command = cli.command.expect("subcommand when not --doc-only");
    finish_entrypoint(dispatch_command(command, &cli.shared, &matches))
}

pub fn entrypoint_from(
    args: impl IntoIterator<Item = impl Into<std::ffi::OsString> + Clone>,
) -> Exit {
    crate::init_from_env();
    match parse_cli_args_or_exit(args) {
        Ok((cli, matches)) => run_entrypoint(cli, matches),
        Err(exit) => exit,
    }
}
