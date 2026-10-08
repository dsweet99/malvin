use super::Exit;
use super::entrypoint::{
    DefaultRouteDispatch, GatesOnlyDispatch, dispatch_command, dispatch_default_route,
    dispatch_do_workflow, dispatch_gates_only_route, dispatch_mixed_requests, finish_entrypoint,
    prepare_cli_output,
};
use crate::cli::args::Cli;
use malvin::modal_run::options::ModalOptions;

#[path = "entrypoint_from_prep.rs"]
mod prep;

use prep::{
    default_route_needs_session_name, entrypoint_acquire_session, entrypoint_before_dispatch,
    entrypoint_preflight, entrypoint_sweep_stale_acp_spawn_locks, parse_cli_args_or_exit,
};

fn run_entrypoint(
    cli: Cli,
    matches: clap::ArgMatches,
    raw: &[std::ffi::OsString],
    modal: ModalOptions,
) -> Exit {
    prepare_cli_output(&cli.shared);
    if let Some(exit) = entrypoint_before_dispatch(&cli, &matches) {
        return exit;
    }
    if cli.shared.remote.is_some() {
        return super::entrypoint::entrypoint_modal::run_modal_route(&cli, raw, modal);
    }
    malvin::local_llm::housekeep_local_llms();
    entrypoint_sweep_stale_acp_spawn_locks();
    if let Some(exit) = entrypoint_preflight(&cli) {
        return exit;
    }
    let needs_session = cli.has_do_request()
        || crate::cli::config_defaults::is_gates_only_route(&cli)
        || default_route_needs_session_name(&cli);
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
    let raw: Vec<std::ffi::OsString> = args.into_iter().map(Into::into).collect();
    let remote_alias =
        |name: &str| malvin::config::malvin_config_file::load_remote_aliases().remove(name);
    let (raw, modal) = match malvin::modal_run::options::extract_modal_options(raw, remote_alias) {
        Ok(split) => split,
        Err(e) => {
            super::entrypoint::print_command_error(&e);
            return Exit::Failure;
        }
    };
    match parse_cli_args_or_exit(raw.clone()) {
        Ok((cli, matches)) => run_entrypoint(cli, matches, &raw, modal),
        Err(exit) => exit,
    }
}
