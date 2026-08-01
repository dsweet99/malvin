use super::{
    apply_workspace_config_defaults, config_defaults_tests::with_seeded_agent_config, Cli,
};
use clap::{CommandFactory, FromArgMatches};

#[test]
fn mini_flag_is_unknown_argument() {
    let err = Cli::command()
        .try_get_matches_from(["malvin", "--mini", "code", "hello"])
        .expect_err("removed --mini");
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected argument") || msg.contains("--mini"),
        "{msg}"
    );
}

#[test]
fn openrouter_model_selects_without_mini_flag() {
    with_seeded_agent_config(|| {
        let matches = Cli::command().get_matches_from([
            "malvin",
            "--model",
            "openrouter:openai/gpt-4o",
            "code",
            "hello",
        ]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        apply_workspace_config_defaults(&matches, &mut cli).expect("apply");
        assert_eq!(cli.shared.model, "openrouter:openai/gpt-4o");
    });
}

#[test]
fn bare_cli_model_is_rejected() {
    with_seeded_agent_config(|| {
        let matches =
            Cli::command().get_matches_from(["malvin", "--model", "auto", "code", "hello"]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        let err = apply_workspace_config_defaults(&matches, &mut cli).expect_err("bare");
        assert!(err.contains("cursor:") || err.contains("openrouter:"), "{err}");
    });
}

#[test]
fn bare_config_model_is_rejected() {
    use crate::test_utils::with_isolated_home;
    use crate::workspace_paths::malvin_config_path;
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(
            &path,
            r#"
[agent]
model = "auto"
"#,
        )
        .expect("write");
        std::env::set_current_dir(work).expect("cd");
        let matches = Cli::command().get_matches_from(["malvin", "--do", "hello"]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        let err = apply_workspace_config_defaults(&matches, &mut cli).expect_err("bare config");
        assert!(err.contains("cursor:") || err.contains("openrouter:"), "{err}");
        let after = std::fs::read_to_string(&path).expect("read");
        assert!(after.contains("model = \"auto\""), "must not rewrite config");
    });
}

#[test]
fn cli_model_overrides_bare_config_model() {
    use crate::test_utils::with_isolated_home;
    use crate::workspace_paths::malvin_config_path;
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(
            &path,
            r#"
[agent]
model = "auto"
max_loops = 9
"#,
        )
        .expect("write");
        std::env::set_current_dir(work).expect("cd");
        let matches = Cli::command().get_matches_from([
            "malvin",
            "--model",
            "cursor:composer-2",
            "tidy",
        ]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        apply_workspace_config_defaults(&matches, &mut cli).expect("cli model wins");
        assert_eq!(cli.shared.model, "cursor:composer-2");
        let after = std::fs::read_to_string(&path).expect("read");
        assert!(after.contains("model = \"auto\""), "must not rewrite config");
    });
}
