use std::collections::BTreeMap;

use super::AgentConfig;

pub(crate) fn parse_agent_config(
    text: &str,
    nicknames: &BTreeMap<String, String>,
    disable_rpi: bool,
) -> Result<AgentConfig, String> {
    let value: toml::Value = text.parse().map_err(|e| format!("invalid TOML: {e}"))?;
    let agent = value
        .get("agent")
        .ok_or_else(|| "missing [agent] section".to_string())?;
    agent_config_from_table(agent, nicknames, disable_rpi)
}

pub(crate) fn agent_config_from_table(
    agent: &toml::Value,
    nicknames: &BTreeMap<String, String>,
    disable_rpi: bool,
) -> Result<AgentConfig, String> {
    let defaults = AgentConfig::default();
    agent_config_base(agent, &defaults, nicknames, disable_rpi)
}

fn agent_config_base(
    agent: &toml::Value,
    defaults: &AgentConfig,
    nicknames: &BTreeMap<String, String>,
    disable_rpi: bool,
) -> Result<AgentConfig, String> {
    let raw_model =
        super::read_string(agent.get("model")).unwrap_or_else(|| defaults.model.canonical());
    let expanded = nicknames
        .get(raw_model.trim())
        .map_or_else(|| raw_model.trim(), String::as_str);
    let model = crate::model_id::require_config_model(expanded)?;
    super::malvin_config_model_policy::reject_disabled_rpi(disable_rpi, &model)?;
    Ok(AgentConfig {
        model,
        max_hypotheses: super::read_usize(agent.get("max_hypotheses"))
            .unwrap_or(defaults.max_hypotheses),
        max_acp_retries: super::read_u32(agent.get("max_acp_retries"))
            .unwrap_or(defaults.max_acp_retries),
    })
}
