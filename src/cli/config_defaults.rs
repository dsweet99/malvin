use clap::parser::ValueSource;
use clap::{ArgMatches, CommandFactory, FromArgMatches};

use super::{Cli, Commands, SharedOpts};
use malvin::malvin_config_file::AgentConfig;
use malvin::model_id::require_prefixed_model;

pub(crate) fn global_flag_from_command_line(matches: &ArgMatches, id: &str) -> bool {
    matches
        .value_source(id)
        .is_some_and(|source| source == ValueSource::CommandLine)
}

fn finalize_shared_model(matches: &ArgMatches, shared: &mut SharedOpts) -> Result<(), String> {
    let _ = matches;
    let _ = require_prefixed_model(&shared.model.canonical())?;
    Ok(())
}

fn load_agent_config(matches: &ArgMatches) -> Result<AgentConfig, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    if global_flag_from_command_line(matches, "model") {
        return Ok(malvin::malvin_config_file::load_agent_config_lenient(&cwd));
    }
    malvin::malvin_config_file::load_agent_config_strict(&cwd)
}

fn apply_shared_and_finalize(
    matches: &ArgMatches,
    shared: &mut SharedOpts,
    agent: &AgentConfig,
) -> Result<(), String> {
    apply_shared_config_defaults(matches, shared, agent);
    finalize_shared_model(matches, shared)
}

fn apply_default_route_max_hypotheses(matches: &ArgMatches, cli: &mut Cli) -> Result<(), String> {
    if global_flag_from_command_line(matches, "max_hypotheses") {
        return Ok(());
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let configured = malvin::malvin_config_file::load_malvin_config(&cwd)
        .default_workflow
        .max_hypotheses_or_default();
    cli.router.max_hypotheses = if configured == 0 {
        malvin::malvin_config_file::DEFAULT_MAX_HYPOTHESES
    } else {
        configured
    };
    Ok(())
}

fn is_bare_default_route(cli: &Cli) -> bool {
    cli.command.is_none() && cli.has_router_request()
}

#[must_use]
pub(crate) const fn is_gates_only_route(cli: &Cli) -> bool {
    !cli.do_workflow() && cli.command.is_none() && !cli.has_request() && cli.router.gates
}

fn apply_gates_only_loop_defaults(matches: &ArgMatches, cli: &mut Cli, agent: &AgentConfig) {
    if !global_flag_from_command_line(matches, "max_hypotheses") {
        cli.router.max_hypotheses = agent.max_hypotheses;
    }
}

const fn uses_lightweight_config_path(cli: &Cli) -> bool {
    cli.do_workflow()
        || matches!(cli.command, Some(Commands::Admin(_)))
        || (cli.command.is_none() && cli.has_request())
}

const ROUTER_ONLY_WITH_PURE_DO: &[(&str, &str)] = &[
    ("quiet", "--quiet / -q"),
    ("gates", "--gates / -g"),
    ("creative", "--creative"),
    ("no_kpop", "--no-kpop"),
    ("max_loops", "--max-loops"),
    ("max_hypotheses", "--max-hypotheses"),
    ("watch", "--watch"),
];

fn reject_router_only_flags_on_pure_do(
    matches: &ArgMatches,
    cli: &Cli,
) -> Result<(), clap::Error> {
    if !cli.do_workflow() || cli.has_router_request() {
        return Ok(());
    }
    for (id, flag) in ROUTER_ONLY_WITH_PURE_DO {
        if global_flag_from_command_line(matches, id) {
            return Err(clap::Error::raw(
                clap::error::ErrorKind::ArgumentConflict,
                format!("{flag} cannot be used with `--do` unless another REQUEST uses the router"),
            ));
        }
    }
    Ok(())
}

fn apply_gates_only_workspace_defaults(matches: &ArgMatches, cli: &mut Cli) -> Result<(), String> {
    let agent = load_agent_config(matches)?;
    apply_shared_config_defaults(matches, &mut cli.shared, &agent);
    apply_gates_only_loop_defaults(matches, cli, &agent);
    finalize_shared_model(matches, &mut cli.shared)
}

pub fn apply_workspace_config_defaults(matches: &ArgMatches, cli: &mut Cli) -> Result<(), String> {
    if uses_lightweight_config_path(cli) {
        let agent = load_agent_config(matches)?;
        apply_shared_and_finalize(matches, &mut cli.shared, &agent)?;
        if is_bare_default_route(cli) {
            apply_default_route_max_hypotheses(matches, cli)?;
        }
        return Ok(());
    }
    if is_gates_only_route(cli) {
        return apply_gates_only_workspace_defaults(matches, cli);
    }
    finalize_shared_model(matches, &mut cli.shared)
}

pub(crate) fn apply_shared_config_defaults(
    matches: &ArgMatches,
    shared: &mut SharedOpts,
    agent: &AgentConfig,
) {
    if !global_flag_from_command_line(matches, "model") {
        shared.model = agent.model.clone();
    }
    if !global_flag_from_command_line(matches, "max_acp_retries") {
        shared.max_acp_retries = agent.max_acp_retries;
    }
}

pub fn parse_cli_with_config_defaults(
    args: impl IntoIterator<Item = impl Into<std::ffi::OsString> + Clone>,
) -> Result<(Cli, ArgMatches), clap::Error> {
    let args: Vec<std::ffi::OsString> = args.into_iter().map(Into::into).collect();
    let cmd = Cli::command();
    let matches = cmd.try_get_matches_from(args.clone())?;
    let mut cli = Cli::from_arg_matches(&matches)?;
    match crate::cli::request_argv::classify_top_level_requests(&args) {
        Ok(tagged) => cli.tagged_requests = tagged,
        Err(e) => {
            return Err(clap::Error::raw(clap::error::ErrorKind::InvalidValue, e));
        }
    }
    reject_router_only_flags_on_pure_do(&matches, &cli)?;
    if let Err(e) = apply_workspace_config_defaults(&matches, &mut cli) {
        return Err(clap::Error::raw(clap::error::ErrorKind::InvalidValue, e));
    }
    Ok((cli, matches))
}

#[cfg(test)]
#[path = "config_defaults_tests.rs"]
mod config_defaults_tests;

#[cfg(test)]
#[path = "config_defaults_tests_legacy_model.rs"]
mod config_defaults_tests_legacy_model;

#[cfg(test)]
#[path = "config_defaults_tests_router.rs"]
mod config_defaults_tests_router;
