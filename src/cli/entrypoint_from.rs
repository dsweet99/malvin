use super::{
    DefaultRouteDispatch, Exit, GatesOnlyDispatch, dispatch_command, dispatch_default_route,
    dispatch_do_workflow, dispatch_gates_only_route, dispatch_mixed_requests, finish_entrypoint,
    prepare_cli_output, print_command_error,
};
use crate::cli::args::Cli;
use crate::cli::config_defaults::is_gates_only_route;
use crate::cli::entrypoint_checks::{
    ensure_malvin_checks_for_command, ensure_malvin_checks_for_default_route,
    ensure_malvin_checks_for_do_workflow, ensure_malvin_checks_for_gates_only_route,
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

fn entrypoint_doc_exit(cli: &Cli) -> Exit {
    match crate::cli::command_docs::print_doc_for_cli(cli) {
        Ok(()) => Exit::Success,
        Err(e) => {
            print_command_error(&e);
            Exit::Failure
        }
    }
}

fn entrypoint_before_dispatch(cli: &Cli, matches: &clap::ArgMatches) -> Option<Exit> {
    if cli.do_workflow() && cli.command.is_some() {
        print_command_error("`--do` cannot be combined with a subcommand");
        return Some(Exit::Failure);
    }
    if let Some(msg) = reject_admin_with_workflow_only_flags(cli, matches) {
        print_command_error(&msg);
        return Some(Exit::Failure);
    }
    if cli.command.is_none()
        && !cli.has_request()
        && !cli.shared.doc
        && !cli.do_workflow()
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

fn reject_admin_with_workflow_only_flags(cli: &Cli, matches: &clap::ArgMatches) -> Option<String> {
    use crate::cli::config_defaults::global_flag_from_command_line;

    const WORKFLOW_ONLY: &[(&str, &str)] = &[
        ("quiet", "--quiet / -q"),
        ("gates", "--gates / -g"),
        ("creative", "--creative"),
        ("no_kpop", "--no-kpop"),
        ("max_loops", "--max-loops"),
        ("max_hypotheses", "--max-hypotheses"),
        ("verbose", "--verbose / -v"),
        ("max_acp_retries", "--max-acp-retries"),
        ("iml", "--iml"),
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

fn entrypoint_preflight(cli: &Cli) -> Option<Exit> {
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

fn entrypoint_acquire_session() -> Result<(String, malvin::SessionNameGuard), Exit> {
    malvin::acquire_session_name(None).map_err(|e| {
        print_command_error(&e);
        Exit::Failure
    })
}

fn default_route_needs_session_name(cli: &Cli) -> bool {
    cli.command.is_none() && cli.has_router_request()
}

fn entrypoint_sweep_stale_acp_spawn_locks() {
    let Ok(cwd) = std::env::current_dir() else {
        return;
    };
    let chamber = malvin::malvin_acp_spawn_chamber_dir(&cwd);
    if !chamber.is_dir() {
        return;
    }
    if let Err(e) = malvin::acp_spawn_sweep::sweep_stale_acp_spawn_locks(&cwd) {
        tracing::warn!(
            target: "malvin::entrypoint",
            error = %e,
            "stale ACP spawn lock sweep failed; continuing"
        );
    }
}

fn run_entrypoint(cli: Cli, matches: clap::ArgMatches) -> Exit {
    prepare_cli_output(&cli.shared);
    if let Some(exit) = entrypoint_before_dispatch(&cli, &matches) {
        return exit;
    }
    malvin::pi_sdk::housekeep_local_llms();
    entrypoint_sweep_stale_acp_spawn_locks();
    if let Some(exit) = entrypoint_preflight(&cli) {
        return exit;
    }
    let needs_session =
        cli.has_do_request() || is_gates_only_route(&cli) || default_route_needs_session_name(&cli);
    if needs_session {
        let _session_name_guard = match entrypoint_acquire_session() {
            Ok((session_name, guard)) => {
                malvin::set_active_acp_lock_slot(session_name);
                guard
            }
            Err(exit) => return exit,
        };
        return dispatch_after_session(cli, matches);
    }
    dispatch_after_session(cli, matches)
}

fn dispatch_after_session(cli: Cli, matches: clap::ArgMatches) -> Exit {
    use crate::cli::malvin_workflow::{MalvinWorkflow, malvin_workflow_from_cli};

    let Some(workflow) = malvin_workflow_from_cli(cli) else {
        return Exit::Success;
    };
    match workflow {
        MalvinWorkflow::Do { requests, shared } => {
            finish_entrypoint(dispatch_do_workflow(requests, &shared))
        }
        MalvinWorkflow::Admin { admin, model } => finish_entrypoint(dispatch_command(
            crate::cli::Commands::Admin(admin),
            &model.canonical(),
            &matches,
        )),
        MalvinWorkflow::DefaultRoute {
            jobs,
            mut shared,
            mut router,
        } => finish_entrypoint(dispatch_default_route(DefaultRouteDispatch {
            jobs,
            max_loops: router.max_loops,
            max_hypotheses: router.max_hypotheses,
            shared: &mut shared,
            router: &mut router,
            matches: &matches,
        })),
        MalvinWorkflow::Mixed {
            jobs,
            mut shared,
            mut router,
        } => finish_entrypoint(dispatch_mixed_requests(
            jobs,
            &mut shared,
            &mut router,
            &matches,
        )),
        MalvinWorkflow::GatesOnly {
            mut shared,
            mut router,
        } => finish_entrypoint(dispatch_gates_only_route(GatesOnlyDispatch {
            max_loops: router.max_loops,
            max_hypotheses: router.max_hypotheses,
            shared: &mut shared,
            router: &mut router,
            matches: &matches,
        })),
    }
}

pub fn entrypoint_from(
    args: impl IntoIterator<Item = impl Into<std::ffi::OsString> + Clone>,
) -> Exit {
    malvin::init_from_env();
    match parse_cli_args_or_exit(args) {
        Ok((cli, matches)) => run_entrypoint(cli, matches),
        Err(exit) => exit,
    }
}
