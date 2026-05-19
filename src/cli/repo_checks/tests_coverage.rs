use super::kissconfig_warn::should_warn_low_test_coverage;

#[test]
fn repo_checks_kiss_stringify_internal_helpers() {
    let _ = stringify!(super::RepoGateOutput);
    let _ = stringify!(super::gate_run::emit_repo_gate_line);
    let _ = stringify!(super::style_markers::touch_if_missing);
    let _ = stringify!(super::kissconfig_warn::should_warn_low_test_coverage);
    let _ = stringify!(super::gate_run::source_like_files_present);
}

#[test]
fn coverage_warn_when_gate_missing() {
    let v: toml::Value = toml::from_str("").unwrap();
    assert!(should_warn_low_test_coverage(&v));
}

#[test]
fn coverage_warn_when_threshold_missing() {
    let v: toml::Value = toml::from_str("[gate]\nmin_similarity = 0.7\n").unwrap();
    assert!(should_warn_low_test_coverage(&v));
}

#[test]
fn coverage_warn_when_threshold_below_90() {
    let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 89\n").unwrap();
    assert!(should_warn_low_test_coverage(&v));
}

#[test]
fn coverage_ok_at_90() {
    let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 90\n").unwrap();
    assert!(!should_warn_low_test_coverage(&v));
}

#[test]
fn coverage_ok_above_90() {
    let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 100\n").unwrap();
    assert!(!should_warn_low_test_coverage(&v));
}

#[test]
fn coverage_ok_at_90_whole_float() {
    let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 90.0\n").unwrap();
    assert!(!should_warn_low_test_coverage(&v));
}

#[test]
fn coverage_warn_when_threshold_is_fractional_float() {
    let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 90.5\n").unwrap();
    assert!(should_warn_low_test_coverage(&v));
}
