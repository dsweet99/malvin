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
fn parse_default_workflow_max_hypotheses_round_trip() {
    use super::{parse_default_workflow_config, parse_malvin_config, DEFAULT_MAX_HYPOTHESES};
    let missing = parse_default_workflow_config("mem_limit_gb = 4\n").expect("missing ok");
    assert_eq!(missing.max_hypotheses, None);
    assert_eq!(missing.max_hypotheses_or_default(), DEFAULT_MAX_HYPOTHESES);
    let cfg = parse_malvin_config("[default_workflow]\nmax_hypotheses = 7\n");
    assert_eq!(cfg.default_workflow.max_hypotheses, Some(7));
    assert_eq!(cfg.default_workflow.max_hypotheses_or_default(), 7);
}
