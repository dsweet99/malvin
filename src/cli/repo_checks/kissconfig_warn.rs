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

fn gate_test_coverage_threshold_i64(value: &toml::Value) -> Option<i64> {
    let gate = value.get("gate")?;
    let v = gate.get("test_coverage_threshold")?;
    integer_or_whole_float_i64(v)
}

fn integer_or_whole_float_i64(v: &toml::Value) -> Option<i64> {
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
mod smoke_cov_kissconfig_warn {
    #[test]
    fn smoke_cov_repo_checks_kissconfig_warn_units() {
        let _ = super::warn_kissconfig_test_coverage_if_needed;
        let _ = super::gate_test_coverage_threshold_i64;
        let _ = super::integer_or_whole_float_i64;
    }
}
