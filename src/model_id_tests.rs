use super::*;

#[test]
fn parse_cursor_and_pi() {
    let c = parse_model_id("cursor:auto").expect("cursor");
    assert_eq!(c.backend, ModelBackend::Cursor);
    assert_eq!(c.canonical(), "cursor:auto");
    assert!(c.params.is_empty());
    let rpi = parse_model_id("rpi:openai/gpt-4o").expect("rpi");
    assert!(rpi.is_pi());
    assert!(!rpi.is_npm_pi());
    assert_eq!(rpi.canonical(), "rpi:openai/gpt-4o");
    assert_eq!(rpi.pi_provider_and_model(), Some(("openai", "gpt-4o")));
    let npm = parse_model_id("pi:openai/gpt-4o").expect("npm pi");
    assert!(npm.is_npm_pi());
    assert!(!npm.is_pi());
    assert_eq!(npm.canonical(), "pi:openai/gpt-4o");
    assert_eq!(npm.pi_provider_and_model(), Some(("openai", "gpt-4o")));
    let pi_nested = parse_model_id("rpi:openrouter/anthropic/claude-3-haiku").expect("pi nested");
    assert_eq!(
        pi_nested.pi_provider_and_model(),
        Some(("openrouter", "anthropic/claude-3-haiku"))
    );
    let local = parse_model_id("rpi:local/whatever_model_name").expect("local alias");
    assert_eq!(local.canonical(), "rpi:local/whatever_model_name");
    assert_eq!(
        local.pi_provider_and_model(),
        Some(("ollama", "whatever_model_name"))
    );
    let local_nested = parse_model_id("rpi:local/ollama/qwen2.5:1.5b").expect("local nested");
    assert_eq!(local_nested.canonical(), "rpi:local/ollama/qwen2.5:1.5b");
    assert_eq!(
        local_nested.pi_provider_and_model(),
        Some(("ollama", "qwen2.5:1.5b"))
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
    let pi = parse_model_id("rpi:openai/gpt-4o[thinking=high]").expect("pi thinking");
    assert_eq!(pi.slug, "openai/gpt-4o");
    assert_eq!(pi.thinking_param(), Some("high"));
    assert_eq!(pi.canonical(), "rpi:openai/gpt-4o[thinking=high]");
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
        parse_model_id("rpi:openai/gpt-4o[thinking=ultra]")
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
        parse_model_id("rpi:openai/gpt-4o[fast=true]")
            .expect_err("pi only thinking")
            .contains("thinking")
    );
    assert!(
        parse_model_id("rpi:openai/gpt-4o[thinking=nope]")
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
    assert!(parse_model_id("rpi:openai").is_err());
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
            .contains("rpi:openrouter/")
    );
    assert!(
        parse_model_id("local:qwen35_9b_q4")
            .expect_err("legacy")
            .contains("local")
    );
    assert!(parse_model_id("pi:openai").is_err());
}

#[test]
fn require_config_and_helpers() {
    assert!(require_config_model("auto").is_err());
    assert_eq!(
        require_config_model("rpi:openai/gpt-4o")
            .expect("ok")
            .canonical(),
        "rpi:openai/gpt-4o"
    );
    let pi = parse_model_id("rpi:openai/gpt-4o").expect("pi");
    assert_eq!(pi.slug, "openai/gpt-4o");
    assert!(pi.is_pi());
    assert!(!parse_model_id("cursor:auto").expect("cursor").is_pi());
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

#[test]
fn parse_canonical_metamorphic_roundtrip() {
    let samples = [
        "cursor:auto",
        "  cursor:claude-opus-5[effort=high,fast=true]  ",
        "pi:openai/gpt-4o",
        "rpi:openrouter/anthropic/claude-3-haiku[thinking=medium]",
        "codex:gpt-5.6[thinking=high,service=priority]",
    ];
    for raw in samples {
        let parsed = parse_model_id(raw).unwrap_or_else(|e| panic!("parse {raw:?}: {e}"));
        let again = parse_model_id(&parsed.canonical()).expect("canonical reparse");
        assert_eq!(again, parsed, "raw={raw:?}");
        assert_eq!(parsed.to_string(), parsed.canonical());
    }
}

#[test]
fn backend_labels_drain_prefixes_and_provider_split() {
    assert_eq!(ModelBackend::Cursor.label(), "cursor");
    assert_eq!(ModelBackend::NpmPi.label(), "pi");
    assert_eq!(ModelBackend::Pi.label(), "rpi");
    assert_eq!(ModelBackend::Codex.label(), "codex");
    assert!(
        ModelBackend::Cursor
            .drain_idle_prefix()
            .contains("bridge timed out")
    );
    assert!(
        ModelBackend::NpmPi
            .drain_idle_prefix()
            .contains("npm pi rpc timed out")
    );
    assert!(
        ModelBackend::Pi
            .drain_idle_prefix()
            .contains("pi rpc timed out")
    );
    assert!(
        ModelBackend::Codex
            .drain_idle_prefix()
            .contains("codex timed out")
    );
    assert!(
        parse_model_id("cursor:auto")
            .expect("cursor")
            .pi_provider_and_model()
            .is_none()
    );
    assert!(
        parse_model_id("codex:gpt-5.6")
            .expect("codex")
            .pi_provider_and_model()
            .is_none()
    );
    assert!(parse_model_id("rpi:/model").is_err());
    assert!(parse_model_id("rpi:provider/").is_err());
    assert!(parse_model_id("").is_err());
    assert_eq!(
        require_config_model("")
            .expect("empty → default")
            .canonical(),
        crate::support_paths::DEFAULT_CLI_MODEL
    );
    assert_eq!(
        require_config_model("   \t")
            .expect("whitespace → default")
            .canonical(),
        crate::support_paths::DEFAULT_CLI_MODEL
    );
    assert_eq!(
        require_prefixed_model("cursor:auto").expect("ok"),
        "cursor:auto"
    );
    assert!(require_prefixed_model("auto").is_err());
}

#[test]
fn bridge_wire_model_follows_backend_policy() {
    let cursor = parse_model_id("cursor:auto[thinking=high]").expect("cursor");
    assert_eq!(
        cursor.backend.bridge_wire_model(&cursor),
        cursor.cursor_bridge_model()
    );
    let npm = parse_model_id("pi:openai/gpt-4o[thinking=low]").expect("npm");
    assert_eq!(npm.backend.bridge_wire_model(&npm), "openai/gpt-4o");
    let codex = parse_model_id("codex:gpt-5.6[service=foo]").expect("codex");
    assert_eq!(codex.backend.bridge_wire_model(&codex), "gpt-5.6");
}

#[test]
fn bracket_format_split_metamorphic_and_reject_fuzz() {
    let params = vec![
        ModelParam {
            id: "thinking".into(),
            value: "low".into(),
        },
        ModelParam {
            id: "service".into(),
            value: "flex".into(),
        },
    ];
    let rendered = format_bracket_params(&params);
    let (base, round) = split_bracket_params(&format!("gpt-5.6{rendered}")).expect("roundtrip");
    assert_eq!(base, "gpt-5.6");
    assert_eq!(round, params);

    let rejects = [
        "x[",
        "x[a",
        "[a=1]",
        "x[[a=1]]",
        "x[a=1][b=2]",
        "x[a=1,]",
        "x[=v]",
        "x[k=]",
        "x[noeq]",
        "x[a=1,,b=2]",
        "x]",
        "ab]c",
    ];
    for raw in rejects {
        assert!(
            split_bracket_params(raw).is_err(),
            "expected reject for {raw:?}"
        );
    }
    assert!(
        split_bracket_params("x]")
            .expect_err("unbalanced closer")
            .contains(']')
    );
}
