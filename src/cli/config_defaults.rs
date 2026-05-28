//! Apply workspace `.malvin/config.toml` defaults to parsed CLI values when flags were not set.

use clap::parser::ValueSource;
use clap::{ArgMatches, CommandFactory, FromArgMatches};

use super::config_loop::subcommand_flag_from_command_line;
use super::{Cli, Commands, SharedOpts};
use crate::malvin_config_file::AgentConfig;

pub(crate) fn global_flag_from_command_line(matches: &ArgMatches, id: &str) -> bool {
    matches
        .value_source(id)
        .is_some_and(|source| source == ValueSource::CommandLine)
}

pub(crate) struct LoopDefaultMut<'a> {
    pub max_loops: &'a mut usize,
    pub max_hypotheses: &'a mut usize,
}

pub(crate) fn apply_loop_defaults(
    matches: &ArgMatches,
    subcommand: &str,
    loops: LoopDefaultMut<'_>,
    agent: &AgentConfig,
) {
    if !subcommand_flag_from_command_line(matches, subcommand, "max_loops") {
        *loops.max_loops = agent.max_loops;
    }
    if !subcommand_flag_from_command_line(matches, subcommand, "max_hypotheses") {
        *loops.max_hypotheses = agent.max_hypotheses;
    }
}

pub fn apply_workspace_config_defaults(
    matches: &ArgMatches,
    cli: &mut Cli,
) -> Result<(), String> {
    let Some(command) = cli.command.as_mut() else {
        if cli.bare_args.is_empty() {
            return Ok(());
        }
        return Err("internal: bare kpop request not resolved".into());
    };
    match command {
        Commands::Do(_) | Commands::Models(_) => return Ok(()),
        _ => {}
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let agent = crate::malvin_config_file::open_malvin_config(&cwd)?.agent;
    apply_shared_config_defaults(matches, &mut cli.shared, &agent);
    match command {
        Commands::Code(code) => apply_loop_defaults(
            matches,
            "code",
            LoopDefaultMut {
                max_loops: &mut code.max_loops,
                max_hypotheses: &mut code.max_hypotheses,
            },
            &agent,
        ),
        Commands::Constrain(constrain) => apply_loop_defaults(
            matches,
            "constrain",
            LoopDefaultMut {
                max_loops: &mut constrain.max_loops,
                max_hypotheses: &mut constrain.max_hypotheses,
            },
            &agent,
        ),
        Commands::Kpop(kpop) => apply_loop_defaults(
            matches,
            "kpop",
            LoopDefaultMut {
                max_loops: &mut kpop.max_loops,
                max_hypotheses: &mut kpop.max_hypotheses,
            },
            &agent,
        ),
        Commands::Tidy(tidy) => apply_loop_defaults(
            matches,
            "tidy",
            LoopDefaultMut {
                max_loops: &mut tidy.max_loops,
                max_hypotheses: &mut tidy.max_hypotheses,
            },
            &agent,
        ),
        Commands::Do(_)
        | Commands::Init(_)
        | Commands::Invent(_)
        | Commands::Models(_) => {}
    }
    Ok(())
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
) -> Result<Cli, clap::Error> {
    let cmd = Cli::command();
    let matches = cmd.get_matches_from(args);
    let mut cli = Cli::from_arg_matches(&matches)?;
    if let Err(e) = super::bare_invoke::resolve_bare_command(&mut cli, &matches) {
        return Err(clap::Error::raw(clap::error::ErrorKind::InvalidValue, e));
    }
    if let Err(e) = apply_workspace_config_defaults(&matches, &mut cli) {
        return Err(clap::Error::raw(
            clap::error::ErrorKind::InvalidValue,
            e,
        ));
    }
    Ok(cli)
}

#[cfg(test)]
#[path = "config_defaults_tests.rs"]
mod config_defaults_tests;
