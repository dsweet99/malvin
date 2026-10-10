use super::{Cli, apply_workspace_config_defaults, parse_cli_with_config_defaults};
use clap::{CommandFactory, FromArgMatches};

#[test]
fn apply_workspace_config_defaults_skips_default_route() {
    malvin::test_support::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        let config_path = malvin::malvin_config_path(work);
        assert!(!config_path.exists());
        let route_matches = Cli::command().get_matches_from(["malvin", "hello"]);
        let mut route_cli = Cli::from_arg_matches(&route_matches).expect("cli");
        apply_workspace_config_defaults(&route_matches, &mut route_cli).expect("apply");
        assert!(!config_path.exists());
        std::env::set_current_dir(cwd).expect("restore cwd");
    });
}

#[test]
fn raw_usage_errors_end_with_newline() {
    for argv in [
        &["malvin", "--watch", "--do", "x"][..],
        &["malvin", "x", "--creative"][..],
    ] {
        let err = parse_cli_with_config_defaults(argv).expect_err("usage error");
        assert!(err.to_string().ends_with('\n'), "{argv:?}: {err:?}");
    }
}

#[test]
fn removed_max_hypotheses_flag_is_rejected() {
    let err = parse_cli_with_config_defaults(["malvin", "hello", "--max-hypotheses", "7"])
        .expect_err("removed flag");
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected") || msg.contains("unknown") || msg.contains("--max-hypotheses"),
        "{msg}"
    );
}
