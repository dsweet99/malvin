use super::super::Exit;
use super::super::entrypoint::entrypoint_info_flags::entrypoint_advice_or_doc_exit;
use super::super::entrypoint::print_command_error;
use crate::cli::args::Cli;
use crate::cli::config_defaults::is_gates_only_route;
use crate::cli::entrypoint_checks::{
    ensure_malvin_checks_for_command, ensure_malvin_checks_for_default_route,
    ensure_malvin_checks_for_do_workflow, ensure_malvin_checks_for_gates_only_route,
};

pub(super) fn parse_cli_args_or_exit(
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

pub(super) fn entrypoint_before_dispatch(cli: &Cli, matches: &clap::ArgMatches) -> Option<Exit> {
    if cli.do_workflow() && cli.command.is_some() {
        print_command_error("`--do` cannot be combined with a subcommand");
        return Some(Exit::Failure);
    }
    if let Some(msg) = reject_admin_with_workflow_only_flags(cli, matches) {
        print_command_error(&msg);
        return Some(Exit::Failure);
    }
    if let Some(exit) = entrypoint_advice_or_doc_exit(cli) {
        return Some(exit);
    }
    if let Some(exit) = entrypoint_commands_catalog(cli) {
        return Some(exit);
    }
    super::super::entrypoint::entrypoint_short_help::entrypoint_request_missing_short_help(cli)
}

fn entrypoint_commands_catalog(cli: &Cli) -> Option<Exit> {
    let _ = match &cli.command {
        None if !cli.has_request() && !cli.do_workflow() && !is_gates_only_route(cli) => {
            crate::cli::commands_help::print_commands_only_help()
        }
        Some(crate::cli::Commands::Admin(admin)) if admin.command.is_none() => {
            crate::cli::commands_help::print_admin_commands_only_help()
        }
        _ => return None,
    };
    Some(Exit::Success)
}

fn reject_admin_with_workflow_only_flags(cli: &Cli, matches: &clap::ArgMatches) -> Option<String> {
    use crate::cli::config_defaults::global_flag_from_command_line;

    const WORKFLOW_ONLY: &[(&str, &str)] = &[
        ("quiet", "--quiet / -q"),
        ("gates", "--gates / -g"),
        ("creative", "--creative"),
        ("max_loops", "--max-loops"),
        ("verbose", "--verbose / -v"),
        ("max_acp_retries", "--max-acp-retries"),
        ("iml", "--iml"),
        ("remote", "--remote"),
    ];

    if !matches!(cli.command, Some(crate::cli::Commands::Admin(_))) {
        return None;
    }
    for (id, flag) in WORKFLOW_ONLY {
        if global_flag_from_command_line(matches, id) {
            return Some(format!("{flag} cannot be combined with `admin`"));
        }
    }
    None
}

pub(super) fn entrypoint_preflight(cli: &Cli) -> Option<Exit> {
    if cli.has_do_request()
        && let Err(e) = ensure_malvin_checks_for_do_workflow()
    {
        print_command_error(&e);
        return Some(Exit::Failure);
    }
    if let Some(command) = cli.command.as_ref() {
        ensure_malvin_checks_for_command(command);
    }
    if is_gates_only_route(cli) {
        return ensure_malvin_checks_for_gates_only_route().err().map(|e| {
            print_command_error(&e);
            Exit::Failure
        });
    }
    if cli.has_router_request() {
        return ensure_malvin_checks_for_default_route().err().map(|e| {
            print_command_error(&e);
            Exit::Failure
        });
    }
    None
}

pub(super) fn entrypoint_acquire_session() -> Result<(String, malvin::SessionNameGuard), Exit> {
    malvin::acquire_session_name(None).map_err(|e| {
        print_command_error(&e);
        Exit::Failure
    })
}

pub(super) fn default_route_needs_session_name(cli: &Cli) -> bool {
    cli.command.is_none() && cli.has_router_request()
}

pub(super) fn entrypoint_sweep_stale_acp_spawn_locks() {
    let Ok(cwd) = std::env::current_dir() else {
        return;
    };
    let chamber = malvin::malvin_acp_spawn_chamber_dir(&cwd);
    if !chamber.is_dir() {
        return;
    }
    if let Err(e) = malvin::agent_process::acp_spawn_sweep::sweep_stale_acp_spawn_locks(&cwd) {
        tracing::warn!(
            target: "malvin::entrypoint",
            error = %e,
            "stale ACP spawn lock sweep failed; continuing"
        );
    }
}
