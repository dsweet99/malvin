use super::{
    AgentConfig, DEFAULT_MAX_HYPOTHESES, DEFAULT_MAX_LOOPS, DEFAULT_MAX_LOOPS_CODE,
    ensure_config_parent_dir,
    load_malvin_config, merge_missing_keys, open_malvin_config,
    parse_agent_config, parse_template_value, read_on_disk_config_value, write_config_value,
};
use crate::support_paths::DEFAULT_CLI_MODEL;
use crate::test_utils::with_isolated_home;
use crate::workspace_paths::malvin_config_path;

#[test]
fn merge_missing_keys_adds_top_level_and_nested_tables() {
    let template = parse_template_value().expect("template");
    let mut partial: toml::Value = toml::from_str("mem_limit_gb = 6\n").expect("partial");
    assert!(merge_missing_keys(&mut partial, &template));
    let merged = partial.as_table().expect("table");
    assert_eq!(merged.get("mem_limit_gb").and_then(toml::Value::as_integer), Some(6));
    assert!(merged.get("logs").is_some());
    assert!(merged.get("agent").is_some());
}

#[test]
fn merge_missing_keys_is_idempotent() {
    let template = parse_template_value().expect("template");
    let mut value = template.clone();
    assert!(!merge_missing_keys(&mut value, &template));
}

#[test]
fn open_malvin_config_creates_file_with_all_sections() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        assert!(!path.exists());
        let cfg = open_malvin_config(work).expect("open");
        assert!(path.is_file());
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(text.contains("[logs]"));
        assert!(text.contains("[agent]"));
        assert_eq!(cfg.agent.model, DEFAULT_CLI_MODEL);
        assert_eq!(cfg.agent.max_hypotheses, DEFAULT_MAX_HYPOTHESES);
        assert_eq!(cfg.agent.max_loops, DEFAULT_MAX_LOOPS);
        assert_eq!(cfg.agent.max_loops_code, DEFAULT_MAX_LOOPS_CODE);
        assert!(text.contains("theme"));
        assert_eq!(cfg.theme, crate::terminal_palette::TerminalTheme::Dark);
    });
}

#[test]
fn open_malvin_config_merges_missing_agent_in_memory_only() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(
            &path,
            "mem_limit_gb = 6\n\n[logs]\nmax_age_days = 90\nmax_bytes = \"2GiB\"\n",
        )
        .expect("write");
        let before = std::fs::read_to_string(&path).expect("read before");
        let cfg = open_malvin_config(work).expect("open");
        let after = std::fs::read_to_string(&path).expect("read after");
        assert_eq!(before, after, "existing config.toml must never be rewritten");
        assert_eq!(cfg.mem_limit_gb, 6);
        assert_eq!(cfg.agent.max_hypotheses, DEFAULT_MAX_HYPOTHESES);
    });
}

#[test]
fn parse_agent_config_reads_values() {
    let text = r#"
[agent]
model = "gpt-5"
max_hypotheses = 7
max_loops = 3
max_acp_retries = 5
"#;
    let agent = parse_agent_config(text).expect("parse");
    assert_eq!(
        agent,
        AgentConfig {
            model: "gpt-5".to_string(),
            max_hypotheses: 7,
            max_loops: 3,
            max_loops_code: DEFAULT_MAX_LOOPS_CODE,
            max_acp_retries: 5,
        }
    );
}

#[test]
fn parse_agent_config_accepts_string_numbers() {
    let text = r#"
[agent]
model = "m"
max_hypotheses = "11"
max_loops = "2"
max_acp_retries = "4"
"#;
    let agent = parse_agent_config(text).expect("parse");
    assert_eq!(agent.max_hypotheses, 11);
    assert_eq!(agent.max_loops, 2);
    assert_eq!(agent.max_acp_retries, 4);
}

#[test]
fn parse_theme_accepts_dark_and_light() {
    use super::parse_theme;
    use crate::terminal_palette::TerminalTheme;

    assert_eq!(parse_theme("theme = \"dark\"").expect("dark"), TerminalTheme::Dark);
    assert_eq!(parse_theme("theme = \"light\"").expect("light"), TerminalTheme::Light);
    assert_eq!(parse_theme("mem_limit_gb = 4").expect("missing"), TerminalTheme::Dark);
    assert!(parse_theme("theme = \"neon\"").is_err());
}

#[test]
fn open_malvin_config_merges_theme_in_memory_only() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, "mem_limit_gb = 6\n").expect("write");
        let before = std::fs::read_to_string(&path).expect("read before");
        let cfg = open_malvin_config(work).expect("open");
        let after = std::fs::read_to_string(&path).expect("read after");
        assert_eq!(before, after);
        assert_eq!(cfg.theme, crate::terminal_palette::TerminalTheme::Dark);
    });
}

#[test]
fn load_malvin_config_reads_light_theme() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, "theme = \"light\"\n").expect("write");
        let cfg = load_malvin_config(work);
        assert_eq!(cfg.theme, crate::terminal_palette::TerminalTheme::Light);
    });
}

#[test]
fn parse_agent_config_reads_max_loops_code() {
    let text = r#"
[agent]
model = "m"
max_loops = 1
max_loops_code = 4
"#;
    let agent = parse_agent_config(text).expect("parse");
    assert_eq!(agent.max_loops, 1);
    assert_eq!(agent.max_loops_code, 4);
}

#[test]
fn load_malvin_config_uses_defaults_for_invalid_on_disk_toml() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, "not valid {{{ toml").expect("write");
        let cfg = load_malvin_config(work);
        assert_eq!(cfg.agent.model, DEFAULT_CLI_MODEL);
    });
}

#[test]
fn load_malvin_config_merges_partial_file_in_memory_only() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, "mem_limit_gb = 8\n").expect("write");
        let cfg = load_malvin_config(work);
        assert_eq!(cfg.mem_limit_gb, 8);
        assert_eq!(cfg.agent.max_hypotheses, DEFAULT_MAX_HYPOTHESES);
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(!text.contains("[agent]"));
    });
}

#[test]
fn config_io_helpers_read_missing_file_as_empty_table() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        let value = read_on_disk_config_value(&path).expect("read");
        assert!(value.as_table().expect("table").is_empty());
    });
}

#[test]
fn config_io_helpers_write_and_read_round_trip() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        ensure_config_parent_dir(&path).expect("mkdir");
        let value: toml::Value = toml::from_str("mem_limit_gb = 3").expect("toml");
        write_config_value(&path, &value).expect("write");
        let read = read_on_disk_config_value(&path).expect("read");
        assert_eq!(read.get("mem_limit_gb"), value.get("mem_limit_gb"));
    });
}

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
    assert_eq!(cfg.logs.max_age_days, crate::log_gc_config::LogsGcConfig::default().max_age_days);
    assert_eq!(cfg.agent.model, DEFAULT_CLI_MODEL);
    let full = MalvinConfig {
        mem_limit_gb: cfg.mem_limit_gb,
        theme: cfg.theme,
        logs: cfg.logs,
        agent: cfg.agent.clone(),
    };
    assert_eq!(full.agent, cfg.agent);
    assert_eq!(read_string(None), None);
    assert_eq!(read_usize(None), None);
    assert_eq!(read_u32(None), None);
}
