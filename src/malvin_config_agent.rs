//! `[agent]` section parsing for `config.toml`.

use super::AgentConfig;

pub(crate) fn parse_agent_config(text: &str) -> Result<AgentConfig, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    let agent = value
        .get("agent")
        .ok_or_else(|| "missing [agent] section".to_string())?;
    agent_config_from_table(agent)
}

pub(crate) fn agent_config_from_table(agent: &toml::Value) -> Result<AgentConfig, String> {
    let defaults = AgentConfig::default();
    agent_config_base(agent, &defaults)
}

fn agent_config_base(agent: &toml::Value, defaults: &AgentConfig) -> Result<AgentConfig, String> {
    let raw_model =
        super::read_string(agent.get("model")).unwrap_or_else(|| defaults.model.clone());
    // Require `cursor:` / `prime:` prefixes. Ignore legacy `model-mini` if present.
    let model = crate::model_id::require_config_model(&raw_model)?;
    Ok(AgentConfig {
        model,
        max_hypotheses: super::read_usize(agent.get("max_hypotheses"))
            .unwrap_or(defaults.max_hypotheses),
        max_loops: super::read_usize(agent.get("max_loops")).unwrap_or(defaults.max_loops),
        max_loops_code: super::read_usize(agent.get("max_loops_code"))
            .unwrap_or(defaults.max_loops_code),
        max_acp_retries: super::read_u32(agent.get("max_acp_retries"))
            .unwrap_or(defaults.max_acp_retries),
    })
}
