use super::malvin_config_agent::parse_agent_config;
use super::{AgentConfig, DEFAULT_MAX_LOOPS_CODE, open_malvin_config};
use crate::model_id::UNPREFIXED_MODEL_MESSAGE;
use crate::support_paths::DEFAULT_CLI_MODEL;
use crate::test_utils::with_isolated_home;
use crate::workspace_paths::malvin_config_path;

#[test]
fn parse_agent_config_ignores_legacy_model_key() {
    let text = r#"
[agent]
model = "cursor:gpt-5"
"model-mini" = "openai/gpt-4o"
"#;
    let agent = parse_agent_config(text).expect("parse");
    assert_eq!(agent.model, "cursor:gpt-5");
}

#[test]
fn parse_agent_config_rejects_bare_model() {
    let text = r#"
[agent]
model = "gpt-5"
"#;
    let err = parse_agent_config(text).expect_err("bare");
    assert!(
        err.contains("cursor:") || err.contains("mini:") || err == UNPREFIXED_MODEL_MESSAGE,
        "{err}"
    );
}

#[test]
fn open_malvin_config_leaves_bare_model_on_disk_without_rewrite() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(
            &path,
            r#"mem_limit_gb = 6

[agent]
model = "auto"
max_loops = 5
"#,
        )
        .expect("write");
        let before = std::fs::read_to_string(&path).expect("read before");
        let cfg = open_malvin_config(work).expect("open must not hard-fail on bare model");
        let after = std::fs::read_to_string(&path).expect("read after");
        assert_eq!(
            before, after,
            "existing config.toml must never be rewritten"
        );
        assert_eq!(cfg.agent.model, DEFAULT_CLI_MODEL);
        assert!(after.contains("model = \"auto\""));
    });
}

#[test]
fn open_malvin_config_writes_prefixed_model_on_fresh_init() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        assert!(!path.exists());
        open_malvin_config(work).expect("open");
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(
            text.contains("model = \"cursor:auto\""),
            "expected prefixed model in config, got:\n{text}"
        );
        assert!(!text.contains("model-mini"));
    });
}

#[test]
fn parse_agent_config_reads_values_with_prefixed_default_shape() {
    let text = r#"
[agent]
model = "cursor:gpt-5"
max_loops = 3
max_acp_retries = 5
"#;
    let agent = parse_agent_config(text).expect("parse");
    assert_eq!(
        agent,
        AgentConfig {
            model: "cursor:gpt-5".to_string(),
            max_loops: 3,
            max_hypotheses: crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES,
            max_loops_code: DEFAULT_MAX_LOOPS_CODE,
            max_acp_retries: 5,
        }
    );
    let _ = DEFAULT_CLI_MODEL;
}
