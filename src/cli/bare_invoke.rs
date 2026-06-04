//! Resolve bare `malvin [OPTIONS] REQUEST` invocations (no subcommand) into kpop.

use clap::parser::ValueSource;
use clap::ArgMatches;

use super::args_bug_kpop::KpopArgs;
use super::config_loop::subcommand_flag_from_command_line;
use super::{Cli, Commands};

pub(crate) fn join_request_parts(parts: &[String]) -> String {
    parts.join(" ")
}

pub(crate) fn require_bare_request(
    parts: &[String],
    usage: &str,
) -> Result<String, String> {
    if parts.is_empty() {
        return Err(format!(
            "malvin {usage}: missing required REQUEST (text or path)"
        ));
    }
    let joined = join_request_parts(parts);
    if joined.trim().is_empty() {
        return Err(format!(
            "malvin {usage}: missing required REQUEST (text or path)"
        ));
    }
    Ok(joined)
}

pub(crate) struct BareLoopOpts {
    max_loops: usize,
    max_hypotheses: usize,
    tenacious: bool,
}

pub(crate) fn bare_loop_opts(cli: &Cli, matches: &ArgMatches, defaults: BareLoopOpts) -> BareLoopOpts {
    BareLoopOpts {
        max_loops: if subcommand_flag_from_command_line(matches, "kpop", "max_loops") {
            cli.bare_max_loops
        } else {
            defaults.max_loops
        },
        max_hypotheses: if subcommand_flag_from_command_line(matches, "kpop", "max_hypotheses") {
            cli.bare_max_hypotheses
        } else {
            defaults.max_hypotheses
        },
        tenacious: if matches
            .value_source("bare_tenacious")
            .is_some_and(|s| s == ValueSource::CommandLine)
        {
            cli.bare_tenacious
        } else {
            defaults.tenacious
        },
    }
}

pub(crate) fn resolve_bare_kpop(cli: &Cli, matches: &ArgMatches) -> Result<Commands, String> {
    let request = require_bare_request(&cli.bare_args, "REQUEST")?;
    let loops = bare_loop_opts(
        cli,
        matches,
        BareLoopOpts {
            max_loops: 1,
            max_hypotheses: crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES,
            tenacious: crate::cli::loop_opts::DEFAULT_TENACIOUS,
        },
    );
    Ok(Commands::Kpop(KpopArgs {
        max_loops: loops.max_loops,
        max_hypotheses: loops.max_hypotheses,
        tenacious: loops.tenacious,
        request: Some(request),
    }))
}

/// When `command` is unset, interpret trailing `bare_args` as a kpop request.
pub(crate) fn resolve_bare_command(cli: &mut Cli, matches: &ArgMatches) -> Result<(), String> {
    if cli.command.is_some() || cli.shared.doc {
        return Ok(());
    }
    if cli.bare_args.is_empty() {
        return Ok(());
    }
    cli.command = Some(resolve_bare_kpop(cli, matches)?);
    Ok(())
}

#[cfg(test)]
#[path = "bare_invoke_tests.rs"]
mod bare_invoke_tests;
