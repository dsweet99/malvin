use clap::parser::ValueSource;
use clap::ArgMatches;

pub(crate) fn subcommand_flag_from_command_line(
    matches: &ArgMatches,
    subcommand: &str,
    id: &str,
) -> bool {
    let Some((name, sub)) = matches.subcommand() else {
        return false;
    };
    if name != subcommand {
        return false;
    }
    sub.value_source(id)
        .is_some_and(|source| source == ValueSource::CommandLine)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    use crate::cli::Cli;

    #[test]
    fn subcommand_flag_from_command_line_detects_cli_overrides() {
        let matches = Cli::command().get_matches_from(["malvin", "kpop", "hello"]);
        assert!(!subcommand_flag_from_command_line(&matches, "kpop", "max_loops"));
        assert!(!subcommand_flag_from_command_line(&matches, "missing", "max_loops"));

        let explicit = Cli::command().get_matches_from([
            "malvin", "kpop", "--max-loops", "2", "hello",
        ]);
        assert!(subcommand_flag_from_command_line(&explicit, "kpop", "max_loops"));
    }
}
