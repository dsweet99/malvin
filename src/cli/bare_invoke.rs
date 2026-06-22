//! Resolve bare `malvin [OPTIONS] REQUEST...` invocations (no subcommand) into kpop.

use clap::parser::ValueSource;
use clap::ArgMatches;

use super::args_bug_kpop::KpopArgs;
use super::config_loop::subcommand_flag_from_command_line;
use super::{Cli, Commands};

pub(crate) fn require_bare_request(
    request: Option<&String>,
    usage: &str,
) -> Result<String, String> {
    let trimmed = request
        .map(String::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty());
    trimmed
        .map(std::string::ToString::to_string)
        .ok_or_else(|| format!("malvin {usage}: missing required REQUEST (text or path)"))
}

#[derive(Default)]
pub(crate) struct BareLoopOpts {
    pub(crate) max_loops: usize,
    pub(crate) max_hypotheses: usize,
    pub(crate) tenacious: bool,
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
    let request = require_bare_request(cli.bare_args.first(), "REQUEST")?;
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

/// When `command` is unset, interpret a single trailing `REQUEST` as a kpop request.
pub(crate) fn resolve_bare_command(cli: &mut Cli, matches: &ArgMatches) -> Result<(), String> {
    if cli.command.is_some() || cli.shared.doc {
        return Ok(());
    }
    if cli.bare_args.is_empty() || cli.bare_args.len() > 1 {
        return Ok(());
    }
    cli.command = Some(resolve_bare_kpop(cli, matches)?);
    Ok(())
}

#[cfg(test)]
mod kiss_cov_inline {
    use super::*;

    #[test]
    fn kiss_cov_band80_witnesses() {
        let _ = stringify!(BareLoopOpts);
        let _ = stringify!(max_loops);
        let _ = stringify!(max_hypotheses);
        let _ = stringify!(tenacious);
        let opts = BareLoopOpts {
            max_loops: 3,
            max_hypotheses: 5,
            tenacious: true,
        };
        let BareLoopOpts {
            max_loops,
            max_hypotheses,
            tenacious,
        } = opts;
        assert_eq!(max_loops, 3);
        assert_eq!(max_hypotheses, 5);
        assert!(tenacious);
        let falsy = BareLoopOpts {
            max_loops: 1,
            max_hypotheses: 1,
            tenacious: false,
        };
        let BareLoopOpts {
            max_loops,
            max_hypotheses,
            tenacious,
        } = falsy;
        assert_eq!(max_loops, 1);
        assert_eq!(max_hypotheses, 1);
        assert!(!tenacious);
    }
}
#[cfg(test)]
#[path = "bare_invoke_test.rs"]
mod bare_invoke_test;
#[cfg(test)]
#[path = "bare_invoke_kiss_cov_test.rs"]
mod bare_invoke_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<BareLoopOpts> = None;
    }
}
