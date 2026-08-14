use super::{
    ensure_config_parent_dir, load_malvin_config, read_on_disk_config_value,
};
use crate::support_paths::DEFAULT_CLI_MODEL;
use crate::test_utils::with_isolated_home;
use crate::workspace_paths::malvin_config_path;

#[test]
fn read_on_disk_config_value_rejects_invalid_toml() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        ensure_config_parent_dir(&path).expect("mkdir");
        std::fs::write(&path, "not toml").expect("write");
        assert!(read_on_disk_config_value(&path).is_err());
    });
}

#[test]
fn parse_malvin_config_falls_back_when_values_invalid_or_missing() {
    use super::{parse_malvin_config, read_string, read_u32, read_usize, MalvinConfig};
    let cfg = parse_malvin_config("mem_limit_gb = 0\n");
    assert!(cfg.mem_limit_gb >= 1);
    assert_eq!(cfg.context_size, super::DEFAULT_CONTEXT_SIZE);
    assert_eq!(cfg.logs.max_age_days, crate::log_gc_config::LogsGcConfig::default().max_age_days);
    assert_eq!(cfg.agent.model, DEFAULT_CLI_MODEL);
    let full = MalvinConfig {
        mem_limit_gb: cfg.mem_limit_gb,
        context_size: cfg.context_size,
        theme: cfg.theme,
        token_cost_rates: cfg.token_cost_rates.clone(),
        logs: cfg.logs,
        agent: cfg.agent.clone(),
        review: cfg.review.clone(),
        default_workflow: cfg.default_workflow.clone(),
    };
    assert_eq!(full.agent, cfg.agent);
    assert_eq!(read_string(None), None);
    assert_eq!(read_usize(None), None);
    assert_eq!(read_u32(None), None);
}

#[test]
fn load_malvin_config_ignores_legacy_mpc_key() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, "mem_limit_gb = 4\nmpc = false\n").expect("write");
        let cfg = load_malvin_config(work);
        assert_eq!(cfg.mem_limit_gb, 4);
    });
}

#[test]
#[allow(clippy::float_cmp)]
fn parse_model_token_cost_rates_defaults_and_rejects_negative() {
    use super::{parse_malvin_config, parse_model_token_cost_rates};
    let missing = parse_model_token_cost_rates("mem_limit_gb = 4\n").expect("defaults");
    assert!(missing.is_empty());
    let cfg = parse_malvin_config(
        "[agent.cursor.auto]\nusd_per_microtoken_in = 3.0\nusd_per_microtoken_out = 15.0\nusd_per_microtoken_cache_read = 0.3\nusd_per_microtoken_cache_write = 3.75\n",
    );
    let rates = cfg.token_cost_rates_for("cursor:auto");
    assert!((rates.usd_per_microtoken_in - 3.0).abs() < f64::EPSILON);
    assert!((rates.usd_per_microtoken_out - 15.0).abs() < f64::EPSILON);
    assert!((rates.usd_per_microtoken_cache_read - 0.3).abs() < f64::EPSILON);
    assert!((rates.usd_per_microtoken_cache_write - 3.75).abs() < f64::EPSILON);
    assert_eq!(cfg.token_cost_rates_for("cursor:other"), super::TokenCostRates::default());
    let dual = parse_malvin_config(
        "[agent.cursor.auto]\nusd_per_microtoken_in = 3.0\n[agent.cursor.gpt-5]\nusd_per_microtoken_out = 15.0\n",
    );
    assert!((dual.token_cost_rates_for("cursor:auto").usd_per_microtoken_in - 3.0).abs() < f64::EPSILON);
    assert!((dual.token_cost_rates_for("cursor:gpt-5").usd_per_microtoken_out - 15.0).abs() < f64::EPSILON);
    assert!(parse_model_token_cost_rates("[agent.cursor.auto]\nusd_per_microtoken_in = -1\n").is_err());
    assert_eq!(cfg.token_cost_rates_for("auto"), super::TokenCostRates::default());
    let pi = parse_malvin_config(
        "[agent.pi.\"openai/gpt-4o-mini\"]\nusd_per_microtoken_in = 1.5\nusd_per_microtoken_out = 2.5\n",
    );
    assert!((pi.token_cost_rates_for("pi:openai/gpt-4o-mini").usd_per_microtoken_in - 1.5).abs() < f64::EPSILON);
    assert!((pi.token_cost_rates_for("pi:openai/gpt-4o-mini").usd_per_microtoken_out - 2.5).abs() < f64::EPSILON);
    let nested = parse_model_token_cost_rates(
        "[agent.pi.openai.gpt-4o-mini]\nusd_per_microtoken_in = 9.0\n",
    )
    .expect("parse");
    assert!(!nested.contains_key("pi:openai/gpt-4o-mini"));
}

