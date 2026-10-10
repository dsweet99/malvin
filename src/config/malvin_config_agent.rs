use std::collections::BTreeMap;

use super::AgentConfig;

pub(crate) fn parse_agent_config(
    text: &str,
    model_aliases: &BTreeMap<String, String>,
) -> Result<AgentConfig, String> {
    let value: toml::Value = text.parse().map_err(|e| format!("invalid TOML: {e}"))?;
    let agent = value
        .get("agent")
        .ok_or_else(|| "missing [agent] section".to_string())?;
    agent_config_from_table(agent, model_aliases)
}

pub(crate) fn agent_config_from_table(
    agent: &toml::Value,
    model_aliases: &BTreeMap<String, String>,
) -> Result<AgentConfig, String> {
    let defaults = AgentConfig::default();
    agent_config_base(agent, &defaults, model_aliases)
}

fn agent_config_base(
    agent: &toml::Value,
    defaults: &AgentConfig,
    model_aliases: &BTreeMap<String, String>,
) -> Result<AgentConfig, String> {
    let raw_model =
        super::read_string(agent.get("model")).unwrap_or_else(|| defaults.model.canonical());
    let expanded = model_aliases
        .get(raw_model.trim())
        .map_or_else(|| raw_model.trim(), String::as_str);
    let model = crate::config::model_id::require_config_model(expanded)?;
    Ok(AgentConfig {
        model,
        max_acp_retries: super::read_u32(agent.get("max_acp_retries"))
            .unwrap_or(defaults.max_acp_retries),
    })
}
