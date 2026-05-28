use clap::parser::ValueSource;
use clap::ArgMatches;

pub(crate) fn subcommand_flag_from_command_line(
    matches: &ArgMatches,
    subcommand: &str,
    id: &str,
) -> bool {
    let Some((name, sub)) = matches.subcommand() else {
        return bare_workflow_flag_from_command_line(matches, subcommand, id);
    };
    if name != subcommand {
        return false;
    }
    sub.value_source(id)
        .is_some_and(|source| source == ValueSource::CommandLine)
}

pub(crate) fn bare_workflow_flag_from_command_line(
    matches: &ArgMatches,
    subcommand: &str,
    id: &str,
) -> bool {
    if subcommand != "kpop" {
        return false;
    }
    let bare_id = match id {
        "max_loops" => "bare_max_loops",
        "max_hypotheses" => "bare_max_hypotheses",
        "tenacious" => "bare_tenacious",
        _ => return false,
    };
    matches
        .value_source(bare_id)
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

    #[test]
    fn bare_invocation_max_loops_flag_detected_at_top_level() {
        let matches = Cli::command().get_matches_from(["malvin", "--max-loops", "2", "hello"]);
        assert!(subcommand_flag_from_command_line(&matches, "kpop", "max_loops"));
        assert!(!subcommand_flag_from_command_line(&matches, "code", "max_loops"));
    }

    #[test]
    fn bare_invocation_max_hypotheses_flag_detected_at_top_level() {
        let matches = Cli::command().get_matches_from(["malvin", "--max-hypotheses", "7", "hello"]);
        assert!(subcommand_flag_from_command_line(&matches, "kpop", "max_hypotheses"));
    }

    #[test]
    fn bare_workflow_flag_from_command_line_maps_bare_max_loops() {
        let matches = Cli::command().get_matches_from(["malvin", "--max-loops", "2", "hello"]);
        assert!(bare_workflow_flag_from_command_line(&matches, "kpop", "max_loops"));
        assert!(!bare_workflow_flag_from_command_line(&matches, "code", "max_loops"));
        assert!(!bare_workflow_flag_from_command_line(&matches, "kpop", "missing"));
    }
}
