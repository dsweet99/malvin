use super::{
    command_accepts_session_name, dispatch_command, dispatch_default_route, finish_entrypoint,
    prepare_cli_output, print_command_error, unsupported_name_error, Commands, Exit,
};
use crate::cli::args::Cli;
use crate::cli::entrypoint_checks::{
    ensure_malvin_checks_for_command, ensure_malvin_checks_for_default_route,
};

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
        Commands::Inspire(inspire) | Commands::Adaptix(inspire) => {
            (inspire.request.as_ref(), "inspire")
        }
        Commands::Explain(explain) => (explain.request.as_ref(), "explain"),
        Commands::Priors(priors) => (priors.request.as_ref(), "priors"),
        Commands::Kpop(kpop) => (kpop.requests.first(), "kpop"),
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
    if cli.command.is_none() && cli.request.is_none() && !cli.shared.doc {
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

fn entrypoint_preflight(cli: &Cli) -> Option<Exit> {
    if let Some(command) = cli.command.as_ref() {
        return ensure_malvin_checks_for_command(command).err().map(|e| {
            print_command_error(&e);
            Exit::Failure
        });
    }
    if cli.request.is_some() {
        return ensure_malvin_checks_for_default_route().err().map(|e| {
            print_command_error(&e);
            Exit::Failure
        });
    }
    None
}

fn entrypoint_acquire_session(opt_name: Option<&str>) -> Result<(String, crate::SessionNameGuard), Exit> {
    crate::acquire_session_name(opt_name).map_err(|e| {
        print_command_error(&e);
        Exit::Failure
    })
}

fn entrypoint_validate_name(cli: &Cli) -> Option<Exit> {
    cli.shared.name.as_ref()?;
    if default_route_accepts_session_name(cli) {
        return None;
    }
    let command = cli.command.as_ref().expect("command or default route request");
    unsupported_name_error(command).map(|message| {
        print_command_error(message);
        Exit::Failure
    })
}

const fn default_route_accepts_session_name(cli: &Cli) -> bool {
    cli.command.is_none() && cli.request.is_some()
}

fn entrypoint_sweep_stale_acp_spawn_locks() {
    let Ok(cwd) = std::env::current_dir() else {
        return;
    };
    let chamber = crate::malvin_acp_spawn_chamber_dir(&cwd);
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
    if let Some(exit) = entrypoint_validate_name(&cli) {
        return exit;
    }
    if let Some(exit) = entrypoint_preflight(&cli) {
        return exit;
    }
    if cli.command.is_some() || default_route_accepts_session_name(&cli) {
        let accepts_name = cli
            .command
            .as_ref()
            .is_some_and(command_accepts_session_name)
            || default_route_accepts_session_name(&cli);
        if accepts_name {
            let _session_name_guard = match entrypoint_acquire_session(cli.shared.name.as_deref()) {
                Ok((session_name, guard)) => {
                    crate::set_active_acp_lock_slot(session_name);
                    guard
                }
                Err(exit) => return exit,
            };
        }
    }
    if let Some(command) = cli.command {
        finish_entrypoint(dispatch_command(command, &cli.shared, &matches))
    } else if let Some(request) = cli.request {
        let mut shared = cli.shared;
        finish_entrypoint(dispatch_default_route(
            request,
            cli.max_loops,
            &mut shared,
            &matches,
        ))
    } else {
        Exit::Success
    }
}

pub fn entrypoint_from(
    args: impl IntoIterator<Item = impl Into<std::ffi::OsString> + Clone>,
) -> Exit {
    crate::init_from_env();
    let args: Vec<std::ffi::OsString> = args.into_iter().map(Into::into).collect();
    if let Some(exit) = crate::cli::deprecated_code::exit_if_code_subcommand(&args) {
        return exit;
    }
    match parse_cli_args_or_exit(args) {
        Ok((cli, matches)) => run_entrypoint(cli, matches),
        Err(exit) => exit,
    }
}
