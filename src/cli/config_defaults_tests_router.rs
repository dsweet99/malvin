use super::{
    apply_workspace_config_defaults, Cli,
};
use clap::{CommandFactory, FromArgMatches};

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
