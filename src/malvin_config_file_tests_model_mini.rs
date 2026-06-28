use super::malvin_config_agent::parse_agent_config;
use super::{
    AgentConfig, DEFAULT_MAX_LOOPS_CODE, open_malvin_config,
};
use crate::support_paths::MINI_DEFAULT_MODEL;
use crate::test_utils::with_isolated_home;
use crate::workspace_paths::malvin_config_path;

#[test]
fn parse_agent_config_reads_model_mini() {
    let text = r#"
[agent]
model = "gpt-5"
"model-mini" = "openai/gpt-4o"
"#;
    let agent = parse_agent_config(text).expect("parse");
    assert_eq!(agent.model, "gpt-5");
    assert_eq!(agent.model_mini, "openai/gpt-4o");
}

#[test]
fn parse_agent_config_model_mini_defaults_when_missing() {
    let text = r#"
[agent]
model = "gpt-5"
"#;
    let agent = parse_agent_config(text).expect("parse");
    assert_eq!(agent.model_mini, MINI_DEFAULT_MODEL);
}

#[test]
fn open_malvin_config_merges_model_mini_in_memory_only() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(
            &path,
            r#"mem_limit_gb = 6

[agent]
model = "auto"
max_hypotheses = 5
"#,
        )
        .expect("write");
        let before = std::fs::read_to_string(&path).expect("read before");
        let cfg = open_malvin_config(work).expect("open");
        let after = std::fs::read_to_string(&path).expect("read after");
        assert_eq!(before, after, "existing config.toml must never be rewritten");
        assert_eq!(cfg.agent.model_mini, MINI_DEFAULT_MODEL);
        assert!(!after.contains("model-mini"));
    });
}

#[test]
fn open_malvin_config_writes_model_mini_on_fresh_init() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        assert!(!path.exists());
        open_malvin_config(work).expect("open");
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(
            text.contains("model-mini") || text.contains("\"model-mini\""),
            "expected model-mini in config, got:\n{text}"
        );
        assert!(text.contains(MINI_DEFAULT_MODEL));
    });
}

#[test]
fn parse_agent_config_reads_values_includes_model_mini_default() {
    let text = r#"
[agent]
model = "gpt-5"
max_hypotheses = 7
max_loops = 3
max_acp_retries = 5
"#;
    let agent = parse_agent_config(text).expect("parse");
    assert_eq!(
        agent,
        AgentConfig {
            model: "gpt-5".to_string(),
            model_mini: MINI_DEFAULT_MODEL.to_string(),
            max_hypotheses: 7,
            max_loops: 3,
            max_loops_code: DEFAULT_MAX_LOOPS_CODE,
            max_acp_retries: 5,
            max_mini_transport_retries: 3,
        }
    );
}
