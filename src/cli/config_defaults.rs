use clap::parser::ValueSource;
use clap::{ArgMatches, CommandFactory, FromArgMatches};

use super::{Cli, Commands, SharedOpts};
use malvin::config::malvin_config_file::AgentConfig;
use malvin::config::model_id::require_prefixed_model;

pub(crate) fn global_flag_from_command_line(matches: &ArgMatches, id: &str) -> bool {
    matches
        .value_source(id)
        .is_some_and(|source| source == ValueSource::CommandLine)
}

fn finalize_shared_model(matches: &ArgMatches, shared: &mut SharedOpts) -> Result<(), String> {
    let _ = matches;
    let _ = require_prefixed_model(&shared.model.first().canonical())?;
    Ok(())
}

fn load_agent_config(matches: &ArgMatches) -> Result<AgentConfig, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    if global_flag_from_command_line(matches, "model") {
        return Ok(malvin::config::malvin_config_file::load_agent_config_lenient(&cwd));
    }
    malvin::config::malvin_config_file::load_agent_config_strict(&cwd)
}

fn apply_shared_and_finalize(
    matches: &ArgMatches,
    shared: &mut SharedOpts,
    agent: &AgentConfig,
) -> Result<(), String> {
    apply_shared_config_defaults(matches, shared, agent);
    finalize_shared_model(matches, shared)
}

#[must_use]
pub(crate) const fn is_gates_only_route(cli: &Cli) -> bool {
    cli.command.is_none() && !cli.has_request() && cli.router.gates
}

const fn uses_lightweight_config_path(cli: &Cli) -> bool {
    matches!(cli.command, Some(Commands::Admin(_))) || (cli.command.is_none() && cli.has_request())
}

fn apply_gates_only_workspace_defaults(matches: &ArgMatches, cli: &mut Cli) -> Result<(), String> {
    let agent = load_agent_config(matches)?;
    apply_shared_config_defaults(matches, &mut cli.shared, &agent);
    finalize_shared_model(matches, &mut cli.shared)
}

pub fn apply_workspace_config_defaults(matches: &ArgMatches, cli: &mut Cli) -> Result<(), String> {
    if uses_lightweight_config_path(cli) {
        let agent = load_agent_config(matches)?;
        apply_shared_and_finalize(matches, &mut cli.shared, &agent)?;
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

fn tag_per_request_models(
    matches: &ArgMatches,
    cli: &Cli,
    segments: Option<&crate::cli::model_segments::ModelSegments>,
) -> Result<Vec<crate::cli::request_argv::TaggedRequest>, clap::Error> {
    use crate::cli::request_argv::{tag_requests, tag_requests_with_models};
    let Some(seg) = segments else {
        return Ok(tag_requests(&cli.requests));
    };
    let clap_indices: Vec<usize> = matches
        .indices_of("requests")
        .map(Iterator::collect)
        .unwrap_or_default();
    let indices = seg.arg_positions(&clap_indices);
    let models = if seg.has_multiple_models() {
        seg.models_at(&indices, &cli.shared.model)
            .map_err(|e| usage_error(clap::error::ErrorKind::InvalidValue, &e))?
    } else {
        Vec::new()
    };
    let mut tagged = tag_requests_with_models(&cli.requests, models);
    let gates = seg.gates_at(&indices);
    if gates.contains(&true) {
        for (job, on) in tagged.iter_mut().zip(gates) {
            job.gates = Some(on);
        }
    }
    Ok(tagged)
}

pub fn parse_cli_with_config_defaults(
    args: impl IntoIterator<Item = impl Into<std::ffi::OsString> + Clone>,
) -> Result<(Cli, ArgMatches), clap::Error> {
    let args: Vec<std::ffi::OsString> = args.into_iter().map(Into::into).collect();
    let segments = crate::cli::model_segments::split_repeated_model_flags(&args);
    let cmd = Cli::command();
    let matches = match &segments {
        Some(seg) => cmd.try_get_matches_from(seg.args.clone())?,
        None => cmd.try_get_matches_from(args)?,
    };
    let mut cli = Cli::from_arg_matches(&matches)?;
    cli.tagged_requests = tag_per_request_models(&matches, &cli, segments.as_ref())?;
    if let Err(e) = apply_workspace_config_defaults(&matches, &mut cli) {
        return Err(usage_error(clap::error::ErrorKind::InvalidValue, &e));
    }
    Ok((cli, matches))
}

fn usage_error(kind: clap::error::ErrorKind, message: &str) -> clap::Error {
    if message.ends_with('\n') {
        clap::Error::raw(kind, message)
    } else {
        clap::Error::raw(kind, format!("{message}\n"))
    }
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
