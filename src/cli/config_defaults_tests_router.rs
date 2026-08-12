use super::{
    apply_workspace_config_defaults, parse_cli_with_config_defaults, Cli,
};
use clap::{CommandFactory, FromArgMatches};

fn write_default_workflow_max_hypotheses(work: &std::path::Path, value: i64) {
    let path = crate::malvin_config_path(work);
    let mut parsed = std::fs::read_to_string(&path)
        .expect("read")
        .parse::<toml::Value>()
        .expect("parse");
    let section = parsed
        .as_table_mut()
        .expect("table")
        .entry("default_workflow")
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    section
        .as_table_mut()
        .expect("default_workflow table")
        .insert("max_hypotheses".into(), toml::Value::Integer(value));
    std::fs::write(&path, toml::to_string_pretty(&parsed).expect("ser")).expect("write");
}

fn assert_default_route_max_hypotheses(cli_args: &[&str], expected: usize) {
    crate::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        crate::malvin_config_file::open_malvin_config(work).expect("seed");
        write_default_workflow_max_hypotheses(work, 11);
        let (cli, _) = parse_cli_with_config_defaults(cli_args).expect("parse");
        assert_eq!(cli.max_hypotheses, expected);
        std::env::set_current_dir(cwd).expect("restore cwd");
    });
}

#[test]
fn apply_workspace_config_defaults_skips_default_route() {
    crate::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        let config_path = crate::malvin_config_path(work);
        assert!(!config_path.exists());
        let route_matches = Cli::command().get_matches_from(["malvin", "hello"]);
        let mut route_cli = Cli::from_arg_matches(&route_matches).expect("cli");
        apply_workspace_config_defaults(&route_matches, &mut route_cli).expect("apply");
        assert!(!config_path.exists());
        std::env::set_current_dir(cwd).expect("restore cwd");
    });
}

#[test]
fn default_route_max_hypotheses_defaults_to_five() {
    crate::test_utils::with_isolated_home(|_| {
        let (cli, _) = parse_cli_with_config_defaults(["malvin", "hello"]).expect("parse");
        assert_eq!(
            cli.max_hypotheses,
            crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES
        );
    });
}

#[test]
fn default_route_max_hypotheses_uses_default_workflow_config() {
    assert_default_route_max_hypotheses(&["malvin", "hello"], 11);
}

#[test]
fn default_route_max_hypotheses_cli_wins_over_config() {
    assert_default_route_max_hypotheses(
        &["malvin", "--max-hypotheses", "3", "hello"],
        3,
    );
}

#[test]
fn default_route_max_hypotheses_flag_after_request_parses() {
    crate::test_utils::with_isolated_home(|_| {
        let (cli, _) = parse_cli_with_config_defaults([
            "malvin",
            "hello",
            "--max-hypotheses",
            "7",
        ])
        .expect("parse flag after request");
        assert_eq!(cli.max_hypotheses, 7);
        assert_eq!(cli.request.as_deref(), Some("hello"));
    });
}
