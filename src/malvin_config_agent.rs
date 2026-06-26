//! `[agent]` section parsing for `config.toml`.

use super::AgentConfig;

pub(crate) fn parse_agent_config(text: &str) -> Result<AgentConfig, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    let agent = value
        .get("agent")
        .ok_or_else(|| "missing [agent] section".to_string())?;
    Ok(agent_config_from_table(agent))
}

pub(crate) fn agent_config_from_table(agent: &toml::Value) -> AgentConfig {
    let defaults = AgentConfig::default();
    AgentConfig {
        model: super::read_string(agent.get("model")).unwrap_or(defaults.model),
        model_mini: super::read_string(agent.get("model-mini")).unwrap_or(defaults.model_mini),
        max_hypotheses: super::read_usize(agent.get("max_hypotheses"))
            .unwrap_or(defaults.max_hypotheses),
        max_loops: super::read_usize(agent.get("max_loops")).unwrap_or(defaults.max_loops),
        max_loops_code: super::read_usize(agent.get("max_loops_code"))
            .unwrap_or(defaults.max_loops_code),
        max_acp_retries: super::read_u32(agent.get("max_acp_retries"))
            .unwrap_or(defaults.max_acp_retries),
    }
}
