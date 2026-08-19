use super::{
    Cli, Commands, apply_workspace_config_defaults, config_defaults_tests::write_agent_config,
};
use clap::{CommandFactory, FromArgMatches};

fn write_review_max_hypotheses(work: &std::path::Path, value: i64) {
    let path = crate::malvin_config_path(work);
    let mut parsed = std::fs::read_to_string(&path)
        .expect("read")
        .parse::<toml::Value>()
        .expect("parse");
    let review = parsed
        .as_table_mut()
        .expect("table")
        .entry("review")
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    review
        .as_table_mut()
        .expect("review table")
        .insert("max_hypotheses".into(), toml::Value::Integer(value));
    std::fs::write(&path, toml::to_string_pretty(&parsed).expect("ser")).expect("write");
}

fn assert_write_max_hypotheses(cli_args: &[&str], expected: usize) {
    crate::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        crate::malvin_config_file::open_malvin_config(work).expect("seed");
        write_agent_config(work);
        write_review_max_hypotheses(work, 14);
        let matches = Cli::command().get_matches_from(cli_args);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        apply_workspace_config_defaults(&matches, &mut cli).expect("apply");
        match cli.command.expect("command") {
            Commands::Write(a) => assert_eq!(a.max_hypotheses, expected),
            other => panic!("expected write, got {other:?}"),
        }
        std::env::set_current_dir(cwd).expect("restore");
    });
}

#[test]
fn write_max_hypotheses_defaults_to_ten_not_agent() {
    super::config_defaults_tests::with_seeded_agent_config(|| {
        let matches = Cli::command().get_matches_from(["malvin", "write", "topic"]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        apply_workspace_config_defaults(&matches, &mut cli).expect("apply");
        match cli.command.expect("command") {
            Commands::Write(a) => {
                assert_eq!(a.max_hypotheses, 10);
                assert_eq!(a.max_loops, 7);
            }
            other => panic!("expected write, got {other:?}"),
        }
    });
}

#[test]
fn write_max_hypotheses_uses_review_config() {
    assert_write_max_hypotheses(&["malvin", "write", "topic"], 14);
}

#[test]
fn write_max_hypotheses_cli_wins_over_review_config() {
    assert_write_max_hypotheses(&["malvin", "write", "topic", "--max-hypotheses", "3"], 3);
}
