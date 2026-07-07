use super::{
    apply_workspace_config_defaults, Cli,
};
use clap::{CommandFactory, FromArgMatches};

#[test]
fn apply_workspace_config_defaults_skips_router() {
    crate::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        let config_path = crate::malvin_config_path(work);
        assert!(!config_path.exists());
        let router_matches = Cli::command().get_matches_from(["malvin", "router", "hello"]);
        let mut router_cli = Cli::from_arg_matches(&router_matches).expect("cli");
        apply_workspace_config_defaults(&router_matches, &mut router_cli).expect("apply");
        assert!(!config_path.exists());
        std::env::set_current_dir(cwd).expect("restore cwd");
    });
}
