use super::*;

#[test]
fn parse_cursor_and_pi() {
    let c = parse_model_id("cursor:auto").expect("cursor");
    assert_eq!(c.backend, ModelBackend::Cursor);
    assert_eq!(c.canonical(), "cursor:auto");
    assert!(c.params.is_empty());
    let pi = parse_model_id("pi:openai/gpt-4o").expect("pi");
    assert!(pi.is_pi());
    assert_eq!(pi.canonical(), "pi:openai/gpt-4o");
    assert_eq!(pi.pi_provider_and_model(), Some(("openai", "gpt-4o")));
    let pi_nested = parse_model_id("pi:openrouter/anthropic/claude-3-haiku").expect("pi nested");
    assert_eq!(
        pi_nested.pi_provider_and_model(),
        Some(("openrouter", "anthropic/claude-3-haiku"))
    );
    let codex = parse_model_id("codex:gpt-5.6").expect("codex");
    assert!(codex.is_codex());
    assert_eq!(codex.canonical(), "codex:gpt-5.6");
}

#[test]
fn parse_bracket_overrides() {
    let c = parse_model_id("cursor:claude-opus-5[effort=high,fast=true]").expect("cursor params");
    assert_eq!(c.slug, "claude-opus-5");
    assert_eq!(
        c.params,
        vec![
            ModelParam {
                id: "effort".into(),
                value: "high".into(),
            },
            ModelParam {
                id: "fast".into(),
                value: "true".into(),
            },
        ]
    );
    assert_eq!(c.canonical(), "cursor:claude-opus-5[effort=high,fast=true]");
    assert_eq!(
        c.cursor_bridge_model(),
        "claude-opus-5[effort=high,fast=true]"
    );
    let pi = parse_model_id("pi:openai/gpt-4o[thinking=high]").expect("pi thinking");
    assert_eq!(pi.slug, "openai/gpt-4o");
    assert_eq!(pi.thinking_param(), Some("high"));
    assert_eq!(pi.canonical(), "pi:openai/gpt-4o[thinking=high]");
    let codex = parse_model_id("codex:gpt-5.6[thinking=high,service=priority]").expect("codex");
    assert_eq!(codex.slug, "gpt-5.6");
    assert_eq!(codex.thinking_param(), Some("high"));
    assert_eq!(codex.service_param(), Some("priority"));
    assert_eq!(
        codex.canonical(),
        "codex:gpt-5.6[thinking=high,service=priority]"
    );
    assert!(
        parse_model_id("codex:gpt-5.6[fast=true]")
            .expect_err("codex only thinking/service")
            .contains("thinking")
    );
    assert_eq!(
        parse_model_id("codex:gpt-5.6[thinking=off]")
            .expect("shared thinking vocabulary")
            .thinking_param(),
        Some("off")
    );
    assert_eq!(
        parse_model_id("pi:openai/gpt-4o[thinking=ultra]")
            .expect("shared thinking vocabulary")
            .thinking_param(),
        Some("ultra")
    );
    assert!(
        parse_model_id("cursor:opus[")
            .expect_err("unbalanced")
            .contains(']')
    );
    assert!(
        parse_model_id("pi:openai/gpt-4o[fast=true]")
            .expect_err("pi only thinking")
            .contains("thinking")
    );
    assert!(
        parse_model_id("pi:openai/gpt-4o[thinking=nope]")
            .expect_err("bad level")
            .contains("thinking")
    );
}

#[test]
fn format_and_split_bracket_params_helpers() {
    assert_eq!(format_bracket_params(&[]), "");
    let params = vec![
        ModelParam {
            id: "effort".into(),
            value: "high".into(),
        },
        ModelParam {
            id: "fast".into(),
            value: "true".into(),
        },
    ];
    assert_eq!(format_bracket_params(&params), "[effort=high,fast=true]");
    let (base, parsed) = split_bracket_params("claude-opus-5[effort=high,fast=true]").expect("ok");
    assert_eq!(base, "claude-opus-5");
    assert_eq!(parsed, params);
    assert_eq!(split_bracket_params("bare").expect("bare").0, "bare");
    assert!(split_bracket_params("x[a=1,b]").is_err());
    assert!(split_bracket_params("x[=1]").is_err());
    assert!(split_bracket_params("x[a=]").is_err());
    assert!(split_bracket_params("x[a=1,]").is_err());
    assert!(split_bracket_params("[a=1]").is_err());
    let empty_brackets = split_bracket_params("model[]").expect("empty");
    assert_eq!(empty_brackets.0, "model");
    assert!(empty_brackets.1.is_empty());
}

#[test]
fn reject_bare_legacy_and_empty_slug() {
    assert!(parse_model_id("auto").is_err());
    assert!(parse_model_id("cursor:").is_err());
    assert!(parse_model_id("pi:openai").is_err());
    assert!(
        parse_model_id("prime:openai/gpt-4o")
            .expect_err("legacy prime")
            .contains("prime:")
    );
    assert!(
        parse_model_id("mini:openrouter/x")
            .expect_err("legacy mini")
            .contains("mini:")
    );
    assert!(
        parse_model_id("openrouter:x")
            .expect_err("legacy")
            .contains("pi:openrouter/")
    );
    assert!(
        parse_model_id("local:qwen35_9b_q4")
            .expect_err("legacy")
            .contains("local")
    );
}

#[test]
fn require_config_and_helpers() {
    assert!(require_config_model("auto").is_err());
    assert_eq!(
        require_config_model("pi:openai/gpt-4o").expect("ok"),
        "pi:openai/gpt-4o"
    );
    assert_eq!(provider_slug("pi:openai/gpt-4o"), "openai/gpt-4o");
    assert!(uses_pi_backend("pi:openai/gpt-4o"));
    assert!(!uses_pi_backend("cursor:auto"));
}

#[test]
fn model_backend_and_parsed_model_debug() {
    let backend = ModelBackend::Cursor;
    let _ = format!("{backend:?}");
    assert_eq!(backend, ModelBackend::Cursor);
    let parsed = ParsedModel {
        backend: ModelBackend::Pi,
        slug: "openai/gpt-4o".into(),
        params: Vec::new(),
    };
    let clone = parsed.clone();
    assert_eq!(clone, parsed);
    let _ = format!("{parsed:?}");
}
