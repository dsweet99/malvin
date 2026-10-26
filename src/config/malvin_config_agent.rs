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
    let model = parse_config_models(agent.get("model"), defaults, model_aliases)?;
    Ok(AgentConfig {
        model,
        max_acp_retries: super::read_u32(agent.get("max_acp_retries"))
            .unwrap_or(defaults.max_acp_retries),
    })
}

fn parse_config_models(
    value: Option<&toml::Value>,
    defaults: &AgentConfig,
    aliases: &BTreeMap<String, String>,
) -> Result<crate::config::model_list::ModelList, String> {
    let Some(value) = value else {
        return Ok(defaults.model.clone());
    };
    let entries = match value {
        toml::Value::Array(entries) => entries.iter().collect(),
        toml::Value::String(_) => vec![value],
        _ => return Err("[agent].model must be a list of model strings".into()),
    };
    let models = entries
        .into_iter()
        .map(|entry| {
            let raw = entry
                .as_str()
                .ok_or("[agent].model entries must be strings")?
                .trim();
            if raw.is_empty() {
                return Err("model must not be empty".into());
            }
            let expanded = aliases.get(raw).map_or(raw, String::as_str);
            crate::config::model_id::require_config_model(expanded)
        })
        .collect::<Result<Vec<_>, _>>()?;
    crate::config::model_list::ModelList::new(models)
}
