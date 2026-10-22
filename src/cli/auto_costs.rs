use std::process::Command;

use crate::config::malvin_config_file::{MalvinConfig, TokenCostRates, load_malvin_config};
use crate::config::model_id::ParsedModel;

fn setup_request(model: &ParsedModel) -> String {
    format!(
        "Set up the per-token USD cost rates for {} in ~/.malvinconf/config.toml. Research the current rates, then edit only that configuration. Do not change repository files. Finish after the configuration is saved.",
        model.canonical()
    )
}

fn rates_are_unset(rates: TokenCostRates) -> bool {
    rates == TokenCostRates::default()
}

fn model_has_live_costs(model: &ParsedModel) -> bool {
    model.pi_provider_and_model().is_some_and(|(provider, _)| {
        provider.eq_ignore_ascii_case("openrouter") || provider.eq_ignore_ascii_case("portkey")
    })
}

fn should_setup_costs(config: &MalvinConfig, model: &ParsedModel, disabled: bool) -> bool {
    !disabled
        && config.auto_setup_costs
        && !model_has_live_costs(model)
        && rates_are_unset(config.token_cost_rates_for(&model.canonical()))
}

pub(super) fn maybe_setup_costs(model: &ParsedModel, disabled: bool) {
    if cfg!(test) || crate::agent_process::test_no_real_agent_enabled() {
        return;
    }
    let Ok(work_dir) = std::env::current_dir() else {
        return;
    };
    let config = load_malvin_config(&work_dir);
    if !should_setup_costs(&config, model, disabled) {
        return;
    }

    let Ok(executable) = std::env::current_exe() else {
        crate::output::print_log_warning("could not locate malvin to set up token costs");
        return;
    };
    let status = Command::new(executable)
        .arg(format!("--model={}", model.canonical()))
        .arg("--no-auto-costs")
        .arg(setup_request(model))
        .status();
    if !status.is_ok_and(|status| status.success()) {
        crate::output::print_log_warning(
            "automatic token-cost setup did not complete; continuing with configured rates",
        );
    }

    let refreshed = load_malvin_config(&work_dir);
    if rates_are_unset(refreshed.token_cost_rates_for(&model.canonical())) {
        crate::output::print_log_warning("token costs remain unset after automatic setup");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model_id::parse_model_id;

    fn config(auto_setup_costs: bool) -> MalvinConfig {
        crate::config::malvin_config_file::parse_malvin_config(&format!(
            "auto_setup_costs = {auto_setup_costs}"
        ))
    }

    #[test]
    fn setup_request_names_the_canonical_model() {
        let model = parse_model_id("cursor:auto").expect("model");

        assert_eq!(
            setup_request(&model),
            "Set up the per-token USD cost rates for cursor:auto in ~/.malvinconf/config.toml. Research the current rates, then edit only that configuration. Do not change repository files. Finish after the configuration is saved."
        );
    }

    #[test]
    fn setup_is_guarded_by_switches_live_prices_and_configured_rates() {
        let cursor = parse_model_id("cursor:auto").expect("model");
        assert!(should_setup_costs(&config(true), &cursor, false));
        assert!(!should_setup_costs(&config(false), &cursor, false));
        assert!(!should_setup_costs(&config(true), &cursor, true));

        let openrouter = parse_model_id("pi:openrouter/openai/gpt-4o-mini").expect("model");
        let portkey = parse_model_id("pi:portkey/openai/gpt-4o-mini").expect("model");
        assert!(!should_setup_costs(&config(true), &openrouter, false));
        assert!(!should_setup_costs(&config(true), &portkey, false));

        let priced = crate::config::malvin_config_file::parse_malvin_config(
            "[agent.cursor.auto]\nusd_per_microtoken_in = 1.0",
        );
        assert!(!should_setup_costs(&priced, &cursor, false));
    }
}
