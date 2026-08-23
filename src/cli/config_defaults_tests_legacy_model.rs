use super::{Cli, apply_workspace_config_defaults};
use clap::{CommandFactory, FromArgMatches};

#[test]
fn mini_flag_is_unknown_argument() {
    let err = Cli::command()
        .try_get_matches_from(["malvin", "--mini", "hello"])
        .expect_err("removed --mini");
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected argument") || msg.contains("--mini"),
        "{msg}"
    );
}

#[test]
fn mini_model_is_rejected() {
    let err = Cli::command()
        .try_get_matches_from([
            "malvin",
            "--model",
            "mini:openrouter/openai/gpt-4o",
            "hello",
        ])
        .expect_err("legacy mini");
    let msg = err.to_string();
    assert!(msg.contains("mini:"), "{msg}");
}

#[test]
fn bare_cli_model_is_rejected() {
    let err = Cli::command()
        .try_get_matches_from(["malvin", "--model", "auto", "hello"])
        .expect_err("bare");
    let msg = err.to_string();
    assert!(msg.contains("cursor:") || msg.contains("pi:"), "{msg}");
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
        assert!(err.contains("cursor:") || err.contains("pi:"), "{err}");
        let after = std::fs::read_to_string(&path).expect("read");
        assert!(
            after.contains("model = \"auto\""),
            "must not rewrite config"
        );
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
        let matches =
            Cli::command().get_matches_from(["malvin", "--model", "cursor:composer-2", "-g"]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        apply_workspace_config_defaults(&matches, &mut cli).expect("cli model wins");
        assert_eq!(cli.shared.model.canonical(), "cursor:composer-2");
        let after = std::fs::read_to_string(&path).expect("read");
        assert!(
            after.contains("model = \"auto\""),
            "must not rewrite config"
        );
    });
}
