use super::*;

#[test]
fn parse_logs_gc_config_reads_toml_section() {
    let cfg = parse_logs_gc_config("[logs]\nmax_age_days = 7\nmax_bytes = \"1MiB\"\n").expect("parse");
    assert_eq!(cfg.max_age_days, 7);
    assert_eq!(cfg.max_bytes, parse_byte_size("1MiB"));
}

#[test]
fn parse_logs_gc_config_max_age_days_contract() {
    let default_days = LogsGcConfig::default().max_age_days;
    let cases: &[(&str, u64)] = &[
        ("[logs]\n", default_days),
        ("[logs]\nmax_age_days = 7\n", 7),
        ("[logs]\nmax_age_days = \"14\"\n", 14),
        ("[logs]\nmax_age_days = 0\n", 0),
        ("[logs]\nmax_age_days = true\n", default_days),
        ("[logs]\nmax_age_days = 3.5\n", default_days),
        ("[logs]\nmax_age_days = []\n", default_days),
        ("[logs]\n[logs.nested]\nmax_age_days = 99\n", default_days),
    ];
    for (toml, want_days) in cases {
        let cfg = parse_logs_gc_config(toml).expect("parse");
        assert_eq!(cfg.max_age_days, *want_days, "toml={toml:?}");
    }
}

#[test]
fn parse_max_bytes_value_rejects_non_string() {
    let err = parse_max_bytes_value(&toml::Value::Integer(1)).unwrap_err();
    assert!(err.contains("string"));
}

#[test]
fn parse_max_bytes_value_empty_string_means_unlimited() {
    assert_eq!(parse_max_bytes_value(&toml::Value::String(String::new())).unwrap(), None);
}
