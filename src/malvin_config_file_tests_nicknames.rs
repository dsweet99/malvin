use super::parse_malvin_config;
use crate::test_utils::with_isolated_home;
use crate::workspace_paths::malvin_config_path;

#[test]
fn parse_nicknames_from_config() {
    let cfg = parse_malvin_config(
        r#"
[nicknames]
astra = "pi:openrouter/openai/gpt-astra"
[agent]
model = "astra"
"#,
    );
    assert_eq!(
        cfg.nicknames.get("astra").map(String::as_str),
        Some("pi:openrouter/openai/gpt-astra")
    );
    assert_eq!(
        cfg.agent.model.canonical(),
        "pi:openrouter/openai/gpt-astra"
    );
}

#[test]
fn removed_rpi_agent_model_falls_back_to_default() {
    let cfg = parse_malvin_config(
        r#"
disable_rpi = true
[agent]
model = "rpi:openai/gpt-4o"
"#,
    );
    assert_eq!(
        cfg.agent.model.canonical(),
        crate::support_paths::DEFAULT_CLI_MODEL
    );
}

#[test]
fn resolve_model_expands_nickname_and_rejects_rpi() {
    let cfg = parse_malvin_config(
        r#"
[nicknames]
astra = "pi:openrouter/openai/gpt-astra"
localish = "rpi:openai/gpt-4o"
"#,
    );
    assert_eq!(
        cfg.resolve_model("astra").expect("nick").canonical(),
        "pi:openrouter/openai/gpt-astra"
    );
    let err = cfg.resolve_model("localish").expect_err("rpi removed");
    assert!(err.contains("`rpi:` backend removed"), "{err}");
}

#[test]
fn parse_model_cli_arg_uses_home_nicknames() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(
            &path,
            r#"
[nicknames]
astra = "cursor:gpt-5"
[agent]
model = "cursor:auto"
"#,
        )
        .expect("write");
        let model = super::parse_model_cli_arg("astra").expect("resolve");
        assert_eq!(model.canonical(), "cursor:gpt-5");
    });
}
