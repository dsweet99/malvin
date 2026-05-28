//! Resolve bare `malvin [OPTIONS] …` invocations (no subcommand) per `plan.md`.

use clap::parser::ValueSource;
use clap::ArgMatches;

use super::args_bug_kpop::KpopArgs;
use super::config_loop::subcommand_flag_from_command_line;
use super::code_flow::CodeArgs;
use super::constrain_flow::ConstrainArgs;
use super::tidy_flow::TidyArgs;
use super::{Cli, Commands};
use crate::do_flow::DoArgs;

const WORKFLOW_CODE: &str = "@code";
const WORKFLOW_CONSTRAIN: &str = "@constrain";
const WORKFLOW_TIDY: &str = "@tidy";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AtWorkflow {
    Code,
    Constrain,
    Tidy,
}

impl AtWorkflow {
    fn parse(token: &str) -> Option<Self> {
        match token {
            WORKFLOW_CODE => Some(Self::Code),
            WORKFLOW_CONSTRAIN => Some(Self::Constrain),
            WORKFLOW_TIDY => Some(Self::Tidy),
            _ => None,
        }
    }
}

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

pub(crate) fn reject_multiple_at_selectors(bare_args: &[String]) -> Result<(), String> {
    let at_count = bare_args
        .iter()
        .filter(|t| t.starts_with('@'))
        .count();
    if at_count > 1 {
        return Err(
            "malvin: only one @workflow selector allowed (e.g. @code, @constrain, @tidy)".into(),
        );
    }
    Ok(())
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

pub(crate) fn resolve_at_workflow(
    cli: &Cli,
    matches: &ArgMatches,
    workflow: AtWorkflow,
) -> Result<Commands, String> {
    reject_multiple_at_selectors(&cli.bare_args)?;
    let rest = cli.bare_args.get(1..).unwrap_or_default();
    match workflow {
        AtWorkflow::Tidy => {
            if !rest.is_empty() {
                return Err("malvin @tidy: does not accept a REQUEST".into());
            }
            let loops = bare_loop_opts(
                cli,
                matches,
                BareLoopOpts {
                    max_loops: 3,
                    max_hypotheses: 10,
                    tenacious: false,
                },
            );
            Ok(Commands::Tidy(TidyArgs {
                max_loops: loops.max_loops,
                max_hypotheses: loops.max_hypotheses,
                tenacious: loops.tenacious,
                quick: false,
            }))
        }
        AtWorkflow::Code => {
            let request = require_bare_request(rest, "@code REQUEST")?;
            let loops = bare_loop_opts(
                cli,
                matches,
                BareLoopOpts {
                    max_loops: 5,
                    max_hypotheses: 10,
                    tenacious: false,
                },
            );
            Ok(Commands::Code(CodeArgs {
                max_loops: loops.max_loops,
                max_hypotheses: loops.max_hypotheses,
                tenacious: loops.tenacious,
                trust_the_plan: false,
                dry_run: false,
                skip_pre_checks: false,
                fast: false,
                request: Some(request),
            }))
        }
        AtWorkflow::Constrain => {
            let request = require_bare_request(rest, "@constrain REQUEST")?;
            let loops = bare_loop_opts(
                cli,
                matches,
                BareLoopOpts {
                    max_loops: 5,
                    max_hypotheses: 10,
                    tenacious: false,
                },
            );
            Ok(Commands::Constrain(ConstrainArgs {
                max_loops: loops.max_loops,
                max_hypotheses: loops.max_hypotheses,
                tenacious: loops.tenacious,
                trust_the_plan: false,
                dry_run: false,
                skip_pre_checks: false,
                fast: false,
                request: Some(request),
            }))
        }
    }
}

pub(crate) fn resolve_bare_do(cli: &Cli) -> Result<Option<Commands>, String> {
    if !cli.do_mode {
        return Ok(None);
    }
    if cli.bare_args.iter().any(|t| t.starts_with('@')) {
        return Err("malvin --do: cannot combine with @workflow selectors".into());
    }
    let request = require_bare_request(&cli.bare_args, "--do REQUEST")?;
    Ok(Some(Commands::Do(DoArgs {
        repo_gates: cli.do_repo_gates,
        thoughts: cli.do_thoughts,
        request: Some(request),
    })))
}

pub(crate) fn resolve_bare_kpop(cli: &Cli, matches: &ArgMatches) -> Result<Commands, String> {
    let request = require_bare_request(&cli.bare_args, "REQUEST")?;
    let loops = bare_loop_opts(
        cli,
        matches,
        BareLoopOpts {
            max_loops: 1,
            max_hypotheses: 10,
            tenacious: false,
        },
    );
    Ok(Commands::Kpop(KpopArgs {
        max_loops: loops.max_loops,
        max_hypotheses: loops.max_hypotheses,
        tenacious: loops.tenacious,
        request: Some(request),
    }))
}

pub(crate) fn resolve_bare_at_or_kpop(
    cli: &Cli,
    matches: &ArgMatches,
) -> Result<Option<Commands>, String> {
    if cli.bare_args.is_empty() {
        return Ok(None);
    }
    reject_multiple_at_selectors(&cli.bare_args)?;
    if let Some(first) = cli.bare_args.first() {
        if let Some(workflow) = AtWorkflow::parse(first) {
            return Ok(Some(resolve_at_workflow(cli, matches, workflow)?));
        }
        if first.starts_with('@') {
            return Err(format!(
                "malvin: unknown @workflow {first:?} (expected @code, @constrain, or @tidy)"
            ));
        }
    }
    Ok(Some(resolve_bare_kpop(cli, matches)?))
}

/// When `command` is unset, interpret `bare_args` / `--do` into a [`Commands`] variant.
pub(crate) fn resolve_bare_command(cli: &mut Cli, matches: &ArgMatches) -> Result<(), String> {
    if cli.command.is_some() || cli.shared.doc {
        return Ok(());
    }
    let cmd = match resolve_bare_do(cli)? {
        Some(do_cmd) => do_cmd,
        None => match resolve_bare_at_or_kpop(cli, matches)? {
            Some(cmd) => cmd,
            None => return Ok(()),
        },
    };
    cli.command = Some(cmd);
    Ok(())
}

#[cfg(test)]
#[path = "bare_invoke_tests.rs"]
mod bare_invoke_tests;
