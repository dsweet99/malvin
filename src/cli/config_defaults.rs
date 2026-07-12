//! Apply `~/.malvin_home/config.toml` defaults to parsed CLI values when flags were not set.

use clap::parser::ValueSource;
use clap::{ArgMatches, CommandFactory, FromArgMatches};

use super::config_loop::subcommand_flag_from_command_line;
use super::{Cli, Commands, SharedOpts};
use crate::malvin_config_file::AgentConfig;
use crate::model_id::require_prefixed_model;

pub(crate) fn global_flag_from_command_line(matches: &ArgMatches, id: &str) -> bool {
    matches
        .value_source(id)
        .is_some_and(|source| source == ValueSource::CommandLine)
}

pub(crate) struct LoopDefaultMut<'a> {
    pub max_loops: &'a mut usize,
    pub max_hypotheses: &'a mut usize,
    pub config_max_loops: usize,
    pub config_max_hypotheses: usize,
}

pub(crate) fn apply_loop_defaults(
    matches: &ArgMatches,
    subcommand: &str,
    loops: LoopDefaultMut<'_>,
) {
    if !subcommand_flag_from_command_line(matches, subcommand, "max_loops") {
        *loops.max_loops = loops.config_max_loops;
    }
    if !subcommand_flag_from_command_line(matches, subcommand, "max_hypotheses") {
        *loops.max_hypotheses = loops.config_max_hypotheses;
    }
}

pub(crate) struct CodeWorkflowLoopMut<'a> {
    pub subcommand: &'a str,
    pub max_loops: &'a mut usize,
    pub max_hypotheses: &'a mut usize,
    pub agent: &'a AgentConfig,
}

fn apply_code_workflow_loop_defaults(
    matches: &ArgMatches,
    loops: CodeWorkflowLoopMut<'_>,
) {
    apply_loop_defaults(
        matches,
        loops.subcommand,
        LoopDefaultMut {
            max_loops: loops.max_loops,
            max_hypotheses: loops.max_hypotheses,
            config_max_loops: loops.agent.max_loops_code,
            config_max_hypotheses: loops.agent.max_hypotheses,
        },
    );
}

fn apply_gate_loop_command_defaults(
    matches: &ArgMatches,
    command: &mut Commands,
    agent: &AgentConfig,
) {
    match command {
        Commands::Code(code) => apply_code_workflow_loop_defaults(
            matches,
            CodeWorkflowLoopMut {
                subcommand: "code",
                max_loops: &mut code.max_loops,
                max_hypotheses: &mut code.max_hypotheses,
                agent,
            },
        ),
        Commands::Kpop(kpop) => apply_loop_defaults(
            matches,
            "kpop",
            LoopDefaultMut {
                max_loops: &mut kpop.max_loops,
                max_hypotheses: &mut kpop.max_hypotheses,
                config_max_loops: agent.max_loops,
                config_max_hypotheses: agent.max_hypotheses,
            },
        ),
        Commands::Tidy(tidy) => apply_code_workflow_loop_defaults(
            matches,
            CodeWorkflowLoopMut {
                subcommand: "tidy",
                max_loops: &mut tidy.max_loops,
                max_hypotheses: &mut tidy.max_hypotheses,
                agent,
            },
        ),
        Commands::Delight(delight) => apply_code_workflow_loop_defaults(
            matches,
            CodeWorkflowLoopMut {
                subcommand: "delight",
                max_loops: &mut delight.max_loops,
                max_hypotheses: &mut delight.max_hypotheses,
                agent,
            },
        ),
        Commands::Priors(priors) => apply_code_workflow_loop_defaults(
            matches,
            CodeWorkflowLoopMut {
                subcommand: "priors",
                max_loops: &mut priors.max_loops,
                max_hypotheses: &mut priors.max_hypotheses,
                agent,
            },
        ),
        Commands::Explain(explain) => apply_code_workflow_loop_defaults(
            matches,
            CodeWorkflowLoopMut {
                subcommand: "explain",
                max_loops: &mut explain.max_loops,
                max_hypotheses: &mut explain.max_hypotheses,
                agent,
            },
        ),
        Commands::Revise(revise) => apply_code_workflow_loop_defaults(
            matches,
            CodeWorkflowLoopMut {
                subcommand: "revise",
                max_loops: &mut revise.max_loops,
                max_hypotheses: &mut revise.max_hypotheses,
                agent,
            },
        ),
        Commands::Do(_)
        | Commands::Inspire(_)
        | Commands::Adaptix(_)
        | Commands::Models(_)
        | Commands::Init(_) => {}
    }
}

fn finalize_shared_model(matches: &ArgMatches, shared: &mut SharedOpts) -> Result<(), String> {
    let _ = matches;
    // CLI `--model` and config-sourced values both require a prefix (Q1=a, Q5=c).
    shared.model = require_prefixed_model(&shared.model)?;
    Ok(())
}

fn load_agent_config(matches: &ArgMatches) -> Result<AgentConfig, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    // Bare on-disk `model` only fails when that value would be used (Q5=c).
    // An explicit CLI `--model` may still override a legacy bare config.
    if global_flag_from_command_line(matches, "model") {
        return Ok(crate::malvin_config_file::load_agent_config_lenient(&cwd));
    }
    crate::malvin_config_file::load_agent_config_strict(&cwd)
}

fn apply_shared_and_finalize(
    matches: &ArgMatches,
    shared: &mut SharedOpts,
    agent: &AgentConfig,
) -> Result<(), String> {
    apply_shared_config_defaults(matches, shared, agent);
    finalize_shared_model(matches, shared)
}

const fn uses_lightweight_config_path(cli: &Cli) -> bool {
    matches!(
        cli.command,
        Some(Commands::Do(_) | Commands::Models(_))
    ) || (cli.command.is_none() && cli.request.is_some())
}

pub fn apply_workspace_config_defaults(
    matches: &ArgMatches,
    cli: &mut Cli,
) -> Result<(), String> {
    if uses_lightweight_config_path(cli) {
        let agent = load_agent_config(matches)?;
        return apply_shared_and_finalize(matches, &mut cli.shared, &agent);
    }
    let Some(command) = cli.command.as_mut() else {
        // Bare `malvin` / help-style paths: validate clap default `--model` only.
        return finalize_shared_model(matches, &mut cli.shared);
    };
    let agent = load_agent_config(matches)?;
    apply_shared_config_defaults(matches, &mut cli.shared, &agent);
    apply_gate_loop_command_defaults(matches, command, &agent);
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
    shared.mini_max_transport_retries = agent.max_mini_transport_retries;
}

pub fn parse_cli_with_config_defaults(
    args: impl IntoIterator<Item = impl Into<std::ffi::OsString> + Clone>,
) -> Result<(Cli, ArgMatches), clap::Error> {
    let cmd = Cli::command();
    let matches = cmd.try_get_matches_from(args)?;
    let mut cli = Cli::from_arg_matches(&matches)?;
    if let Err(e) = apply_workspace_config_defaults(&matches, &mut cli) {
        return Err(clap::Error::raw(
            clap::error::ErrorKind::InvalidValue,
            e,
        ));
    }
    Ok((cli, matches))
}

#[cfg(test)]
#[path = "config_defaults_tests.rs"]
mod config_defaults_tests;

#[cfg(test)]
#[path = "config_defaults_tests_mini.rs"]
mod config_defaults_tests_mini;

#[cfg(test)]
#[path = "config_defaults_tests_router.rs"]
mod config_defaults_tests_router;
