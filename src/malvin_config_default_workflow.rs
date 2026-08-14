
use super::DefaultWorkflowConfig;

pub(crate) fn parse_default_workflow_config(text: &str) -> Result<DefaultWorkflowConfig, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    let Some(section) = value.get("default_workflow") else {
        return Ok(DefaultWorkflowConfig::default());
    };
    Ok(DefaultWorkflowConfig {
        max_hypotheses: super::read_usize(section.get("max_hypotheses")),
    })
}
