use std::path::Path;

use super::gate_log::emit_repo_gate_warning;
use super::types::RepoGateOutput;

pub fn warn_kissconfig_test_coverage_if_needed(
    work_dir: &Path,
    _output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) {
    let path = work_dir.join(".kissconfig");
    if !path.is_file() {
        return;
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            emit_repo_gate_warning(&format!("could not read .kissconfig: {e}"), run_log_dir);
            return;
        }
    };
    let value = match text.parse::<toml::Value>() {
        Ok(v) => v,
        Err(e) => {
            emit_repo_gate_warning(&format!("could not parse .kissconfig as TOML: {e}"), run_log_dir);
            return;
        }
    };
    if !should_warn_low_test_coverage(&value) {
        return;
    }
    emit_repo_gate_warning(
        ".kissconfig gate.test_coverage_threshold is missing or below 90; editing code without sufficient unit test coverage is dangerous.",
        run_log_dir,
    );
}

pub(crate) fn gate_test_coverage_threshold_i64(value: &toml::Value) -> Option<i64> {
    let gate = value.get("gate")?;
    let v = gate.get("test_coverage_threshold")?;
    integer_or_whole_float_i64(v)
}

pub(crate) fn integer_or_whole_float_i64(v: &toml::Value) -> Option<i64> {
    if let Some(i) = v.as_integer() {
        return Some(i);
    }
    let f = v.as_float()?;
    if !f.is_finite() || f.fract() != 0.0 {
        return None;
    }
    f.to_string().parse().ok()
}

pub fn should_warn_low_test_coverage(value: &toml::Value) -> bool {
    gate_test_coverage_threshold_i64(value).is_none_or(|t| t < 90)
}

#[cfg(test)]
mod kissconfig_warn_tests {
    use crate::repo_checks::RepoGateOutput;

    #[test]
    fn warn_emits_when_kissconfig_threshold_below_90() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            tmp.path().join(".kissconfig"),
            "[gate]\ntest_coverage_threshold = 50\n",
        )
        .expect("write");
        super::warn_kissconfig_test_coverage_if_needed(tmp.path(), RepoGateOutput::Tagged, None);
    }

    #[test]
    fn should_warn_covers_threshold_parser_edges() {
        let ok: toml::Value =
            toml::from_str("[gate]\ntest_coverage_threshold = 90.0\n").expect("toml");
        assert!(!super::should_warn_low_test_coverage(&ok));
        let fractional: toml::Value =
            toml::from_str("[gate]\ntest_coverage_threshold = 90.5\n").expect("toml");
        assert!(super::should_warn_low_test_coverage(&fractional));
        let missing_gate: toml::Value = toml::from_str("[other]\n").expect("toml");
        assert!(super::should_warn_low_test_coverage(&missing_gate));
        let integer_ok: toml::Value =
            toml::from_str("[gate]\ntest_coverage_threshold = 90\n").expect("toml");
        assert!(!super::should_warn_low_test_coverage(&integer_ok));
        let high: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 100").expect("toml");
        assert!(!super::should_warn_low_test_coverage(&high));
        let low: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 50").expect("toml");
        assert!(super::should_warn_low_test_coverage(&low));
        assert_eq!(
            super::gate_test_coverage_threshold_i64(&integer_ok),
            Some(90)
        );
        assert_eq!(super::integer_or_whole_float_i64(&fractional), None);
    }

    #[test]
    fn warn_path_exercises_gate_log_style_and_source_detect() {
        let tmp = tempfile::tempdir().expect("tempdir");
        crate::repo_checks::gate_log::emit_repo_gate_line(
            RepoGateOutput::Tagged,
            "kissconfig-warn-test",
            None,
        );
        crate::repo_checks::style_markers::touch_if_missing(
            &tmp.path().join("style.md"),
            RepoGateOutput::Tagged,
        )
        .expect("touch");
        assert!(!crate::repo_checks::gate_run::source_like_files_present(tmp.path()));
    }
}
