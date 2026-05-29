//! Structural validation for proposed `.malvin/checks` command lines.

use std::path::Path;

pub(crate) fn checks_command_invocation_resolvable(work_dir: &Path, line: &str) -> bool {
    let trimmed = effective_command_line_for_validation(line);
    let first = trimmed.split_whitespace().next().unwrap_or("");
    if first.is_empty() {
        return false;
    }
    if crate::lookup_bin_on_path(first).is_some() {
        return true;
    }
    work_dir.join(first).is_file()
}

fn effective_command_line_for_validation(line: &str) -> &str {
    let trimmed = line.trim();
    trimmed
        .rfind(" && ")
        .map_or(trimmed, |idx| trimmed[idx + 4..].trim())
}

/// Structural validation for a proposed `.malvin/checks` file (no full gate run).
pub fn validate_checks_command_lines(work_dir: &Path, lines: &[String]) -> Result<(), String> {
    if lines.is_empty() {
        return Err("checks file has no commands".to_string());
    }
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Err(format!("checks line {} is empty", i + 1));
        }
        if trimmed.starts_with('#') {
            return Err(format!("checks line {} is comment-only", i + 1));
        }
        if !checks_command_invocation_resolvable(work_dir, trimmed) {
            let first = trimmed.split_whitespace().next().unwrap_or("");
            return Err(format!(
                "checks line {}: command not found on PATH or in repo: {first}",
                i + 1
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_checks_rejects_missing_binary() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(
            validate_checks_command_lines(
                tmp.path(),
                &["definitely_not_a_malvin_binary_xyz".to_string()]
            )
            .is_err()
        );
    }

    #[test]
    fn validate_checks_rejects_comment_only() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(validate_checks_command_lines(tmp.path(), &["# only".to_string()]).is_err());
    }

    #[test]
    fn validate_checks_accepts_repo_relative_script() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("local_gate.sh"), "#!/bin/sh\n").unwrap();
        validate_checks_command_lines(tmp.path(), &["local_gate.sh".to_string()]).unwrap();
    }

    #[test]
    fn validate_checks_rejects_empty_command_token() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(validate_checks_command_lines(tmp.path(), &["   ".to_string()]).is_err());
    }

    #[test]
    fn checks_command_invocation_resolvable_rejects_missing() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!checks_command_invocation_resolvable(tmp.path(), "missing_bin_xyz"));
    }

    #[test]
    fn effective_command_line_for_validation_uses_last_and_segment() {
        assert_eq!(
            effective_command_line_for_validation("cd rust && cargo clippy"),
            "cargo clippy"
        );
        assert_eq!(
            effective_command_line_for_validation("kiss check"),
            "kiss check"
        );
    }

    #[test]
    fn validate_checks_accepts_compound_cd_and_cargo_clippy() {
        if crate::lookup_bin_on_path("cargo").is_some() {
            let tmp = tempfile::tempdir().unwrap();
            validate_checks_command_lines(
                tmp.path(),
                &["cd rust && cargo clippy --all-targets -- -D warnings".to_string()],
            )
            .unwrap();
        }
    }

    #[test]
    fn validate_checks_accepts_kiss_on_path() {
        if crate::lookup_bin_on_path("kiss").is_some() {
            let tmp = tempfile::tempdir().unwrap();
            validate_checks_command_lines(tmp.path(), &["kiss check".to_string()]).unwrap();
        }
    }
}
