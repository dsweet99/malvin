use super::parse_malvin_config;
use crate::test_support::test_utils::with_isolated_home;
use crate::workspace::workspace_paths::malvin_config_path;
#[test]
fn parse_model_aliases_from_config_and_removed_rpi_agent_model_falls_back_to_default() {
    {
        let cfg = parse_malvin_config(
            r#"
        [aliases.models]
        astra = "pi:openrouter/openai/gpt-astra"
        [agent]
        model = "astra"
        "#,
        );
        assert_eq!(
            cfg.model_aliases.get("astra").map(String::as_str),
            Some("pi:openrouter/openai/gpt-astra")
        );
        assert_eq!(
            cfg.agent.model.canonical(),
            "pi:openrouter/openai/gpt-astra"
        );
    }
    {
        let cfg = parse_malvin_config(
            r#"
        disable_rpi = true
        [agent]
        model = "rpi:openai/gpt-4o"
        "#,
        );
        assert_eq!(
            cfg.agent.model.canonical(),
            crate::workspace::support_paths::DEFAULT_CLI_MODEL
        );
    }
}

#[test]
fn resolve_model_expands_alias_and_rejects_rpi() {
    let cfg = parse_malvin_config(
        r#"
[aliases.models]
astra = "pi:openrouter/openai/gpt-astra"
localish = "rpi:openai/gpt-4o"
"#,
    );
    assert_eq!(
        cfg.resolve_model("astra").expect("alias").canonical(),
        "pi:openrouter/openai/gpt-astra"
    );
    let err = cfg.resolve_model("localish").expect_err("rpi removed");
    assert!(err.contains("`rpi:` backend removed"), "{err}");
}

#[test]
fn parse_model_cli_arg_uses_home_model_aliases() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(
            &path,
            r#"
[aliases.models]
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
#[test]
fn legacy_nicknames_table_is_rejected_with_a_rename_error_and_parse_remote_aliases_from_config() {
    {
        let text = "[nicknames]\nastra = \"cursor:gpt-5\"\n[agent]\nmodel = \"cursor:auto\"\n";
        let err = super::parse_model_aliases(text).expect_err("renamed");
        assert!(err.contains("[aliases.models]"), "{err}");
        assert!(parse_malvin_config(text).model_aliases.is_empty());
    }
    {
        let cfg = parse_malvin_config(
            r#"
        [aliases.models]
        astra = "cursor:gpt-5"
        [aliases.remotes]
        big = " modal:sandbox[gpu=A100,mem=32] "
        [agent]
        model = "astra"
        "#,
        );
        assert_eq!(
            cfg.remote_aliases.get("big").map(String::as_str),
            Some("modal:sandbox[gpu=A100,mem=32]")
        );
        assert_eq!(cfg.agent.model.canonical(), "cursor:gpt-5");
    }
}

#[test]
fn remote_alias_names_are_validated() {
    let err = |body: &str| {
        super::parse_remote_aliases(&format!("[aliases.remotes]\n{body}\n")).expect_err(body)
    };
    assert!(err("modal = \"modal:sandbox[gpu=T4]\"").contains("built-in remote"));
    assert!(err("\"modal:sandbox\" = \"modal:sandbox\"").contains("must not contain ':'"));
    assert!(err("\"a[b]\" = \"modal:sandbox\"").contains("must not contain"));
    assert!(err("big = 3").contains("aliases.remotes.big must be a string"));
    assert!(err("big = \"\"").contains("non-empty"));
    let err = super::parse_model_aliases("[aliases.models]\n\"a:b\" = \"cursor:auto\"\n")
        .expect_err("colon");
    assert!(err.contains("must not contain ':'"), "{err}");
}

#[test]
fn one_bad_alias_entry_is_skipped_and_the_rest_are_kept() {
    let cfg = parse_malvin_config(
        "[aliases.remotes]\nmodal = \"modal:sandbox\"\ngpu1 = \"modal:sandbox[gpu=T4]\"\n[aliases.models]\n\"a:b\" = \"cursor:auto\"\nastra = \"cursor:gpt-5\"\n",
    );
    assert_eq!(cfg.remote_aliases.keys().collect::<Vec<_>>(), ["gpu1"]);
    assert_eq!(cfg.model_aliases.keys().collect::<Vec<_>>(), ["astra"]);
}

#[test]
fn load_remote_aliases_reads_home_config() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, "[aliases.remotes]\nbig = \"modal:sandbox[gpu=A100]\"\n[agent]\nmodel = \"cursor:auto\"\n")
            .expect("write");
        let aliases = super::load_remote_aliases();
        assert_eq!(
            aliases.get("big").map(String::as_str),
            Some("modal:sandbox[gpu=A100]")
        );
    });
}
