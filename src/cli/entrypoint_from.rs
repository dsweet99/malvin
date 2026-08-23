use super::{
    DefaultRouteDispatch, Exit, command_accepts_session_name, dispatch_command,
    dispatch_default_route, dispatch_do_workflow, dispatch_gates_only_route, finish_entrypoint,
    prepare_cli_output, print_command_error, unsupported_name_error,
};
use crate::cli::args::Cli;
use crate::cli::config_defaults::is_gates_only_route;
use crate::cli::entrypoint_checks::{
    ensure_malvin_checks_for_command, ensure_malvin_checks_for_default_route,
    ensure_malvin_checks_for_do_workflow, ensure_malvin_checks_for_gates_only_route,
};
use crate::do_flow::DoArgs;

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

fn entrypoint_doc_exit(cli: &Cli) -> Exit {
    match crate::cli::command_docs::print_doc_for_cli(cli) {
        Ok(()) => Exit::Success,
        Err(e) => {
            print_command_error(&e);
            Exit::Failure
        }
    }
}

fn entrypoint_before_dispatch(cli: &Cli) -> Option<Exit> {
    if cli.do_workflow && cli.command.is_some() {
        print_command_error("`--do` cannot be combined with a subcommand");
        return Some(Exit::Failure);
    }
    if cli.command.is_none()
        && cli.request.is_none()
        && !cli.shared.doc
        && !cli.do_workflow
        && !is_gates_only_route(cli)
    {
        let _ = crate::cli::commands_help::print_commands_only_help();
        return Some(Exit::Success);
    }
    if let Some(exit) = super::entrypoint_short_help::entrypoint_request_missing_short_help(cli) {
        return Some(exit);
    }
    if cli.shared.doc {
        return Some(entrypoint_doc_exit(cli));
    }
    None
}

fn entrypoint_preflight(cli: &Cli) -> Option<Exit> {
    if cli.do_workflow {
        return ensure_malvin_checks_for_do_workflow().err().map(|e| {
            print_command_error(&e);
            Exit::Failure
        });
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
    if cli.request.is_some() {
        return ensure_malvin_checks_for_default_route().err().map(|e| {
            print_command_error(&e);
            Exit::Failure
        });
    }
    None
}

fn entrypoint_acquire_session(
    opt_name: Option<&str>,
) -> Result<(String, crate::SessionNameGuard), Exit> {
    crate::acquire_session_name(opt_name).map_err(|e| {
        print_command_error(&e);
        Exit::Failure
    })
}

fn entrypoint_validate_name(cli: &Cli) -> Option<Exit> {
    cli.shared.name.as_ref()?;
    if cli.do_workflow || default_route_accepts_session_name(cli) || is_gates_only_route(cli) {
        return None;
    }
    cli
        .command
        .as_ref()
        .expect("command or default route request");
    print_command_error(unsupported_name_error());
    Some(Exit::Failure)
}

const fn default_route_accepts_session_name(cli: &Cli) -> bool {
    cli.command.is_none() && cli.request.is_some() && !cli.do_workflow
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
    let accepts_name = cli.do_workflow
        || is_gates_only_route(&cli)
        || cli
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
        return dispatch_after_session(cli, matches);
    }
    dispatch_after_session(cli, matches)
}

fn dispatch_after_session(cli: Cli, matches: clap::ArgMatches) -> Exit {
    if cli.do_workflow {
        return finish_entrypoint(dispatch_do_workflow(
            DoArgs {
                request: cli.request,
            },
            &cli.shared,
        ));
    }
    if let Some(command) = cli.command {
        finish_entrypoint(dispatch_command(command, &cli.shared, &matches))
    } else if let Some(request) = cli.request {
        let mut shared = cli.shared;
        finish_entrypoint(dispatch_default_route(DefaultRouteDispatch {
            request,
            max_loops: cli.max_loops,
            max_hypotheses: cli.max_hypotheses,
            shared: &mut shared,
            matches: &matches,
        }))
    } else if is_gates_only_route(&cli) {
        let mut shared = cli.shared;
        finish_entrypoint(dispatch_gates_only_route(
            cli.max_loops,
            cli.max_hypotheses,
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
    match parse_cli_args_or_exit(args) {
        Ok((cli, matches)) => run_entrypoint(cli, matches),
        Err(exit) => exit,
    }
}
