use std::collections::BTreeMap;

use crate::config::log_gc_config::LogsGcConfig;
use crate::config::model_id::parse_model_id;
use crate::config::model_list::ModelList;
use crate::terminal_palette::TerminalTheme;
use crate::workspace::support_paths::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};

use super::malvin_config_file::TokenCostRates;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentConfig {
    pub model: ModelList,
    pub model_lite: Option<ModelList>,
    pub max_acp_retries: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: parse_model_id(DEFAULT_CLI_MODEL)
                .expect("DEFAULT_CLI_MODEL must parse")
                .into(),
            model_lite: None,
            max_acp_retries: DEFAULT_MAX_ACP_RETRIES,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MalvinConfig {
    pub mem_limit_gb: u64,
    pub context_size: u32,
    pub theme: TerminalTheme,
    pub auto_setup_costs: bool,
    pub model_aliases: BTreeMap<String, String>,
    pub remote_aliases: BTreeMap<String, String>,
    pub token_cost_rates: BTreeMap<String, TokenCostRates>,
    pub logs: LogsGcConfig,
    pub agent: AgentConfig,
    pub dm_on_emit: Option<String>,
}

impl MalvinConfig {
    #[must_use]
    pub fn token_cost_rates_for(&self, model: &str) -> TokenCostRates {
        self.token_cost_rates
            .get(model)
            .copied()
            .unwrap_or_default()
    }
}
