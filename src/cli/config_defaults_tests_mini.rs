use super::{
    apply_workspace_config_defaults, config_defaults_tests::with_seeded_agent_config, Cli,
};
use clap::{CommandFactory, FromArgMatches};

#[test]
fn apply_mini_model_default_uses_model_mini_when_mini_and_no_cli_model() {
    with_seeded_agent_config(|| {
        let matches = Cli::command().get_matches_from(["malvin", "--mini", "code", "hello"]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        apply_workspace_config_defaults(&matches, &mut cli).expect("apply");
        assert_eq!(cli.shared.model, "cfg-mini-model");
    });
}

#[test]
fn apply_mini_model_default_respects_cli_model_override() {
    with_seeded_agent_config(|| {
        let matches = Cli::command().get_matches_from([
            "malvin", "--mini", "--model", "openai/gpt-4o", "code", "hello",
        ]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        apply_workspace_config_defaults(&matches, &mut cli).expect("apply");
        assert_eq!(cli.shared.model, "openai/gpt-4o");
    });
}

#[test]
fn apply_mini_model_default_ignored_without_mini_flag() {
    with_seeded_agent_config(|| {
        let matches = Cli::command().get_matches_from(["malvin", "code", "hello"]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        apply_workspace_config_defaults(&matches, &mut cli).expect("apply");
        assert_eq!(cli.shared.model, "cfg-model");
    });
}

#[test]
fn apply_bare_sequential_mini_model_default() {
    with_seeded_agent_config(|| {
        let matches = Cli::command().get_matches_from(["malvin", "--mini", "a.md", "b.md"]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        crate::cli::bare_invoke::resolve_bare_command(&mut cli, &matches).expect("bare");
        apply_workspace_config_defaults(&matches, &mut cli).expect("apply");
        assert_eq!(cli.shared.model, "cfg-mini-model");
        assert!(cli.command.is_none());
        assert_eq!(cli.bare_args, vec!["a.md", "b.md"]);
    });
}
