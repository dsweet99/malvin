use crate::repo_gates::discover_init_checks::*;
use crate::repo_gates::discover_init_checks_signals::{
    canonical_tool, ensure_kiss_check_first, parse_makefile_targets, parse_yaml_scalar,
};
use crate::repo_gates::{KISS_CHECK_COMMAND};

#[test]
fn precommit_hook_entries_parses_quoted_and_plain() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join(".pre-commit-config.yaml"),
        "repos:\n- repo: local\n  hooks:\n  - id: a\n    entry: ruff check .\n  - id: b\n    entry: \"pytest -sv tests\"\n",
    )
    .unwrap();
    let entries = precommit_hook_entries(tmp.path());
    assert_eq!(
        entries,
        vec!["ruff check .".to_string(), "pytest -sv tests".to_string()]
    );
}

#[test]
fn dedupe_check_lines_keeps_one_per_tool() {
    let lines = vec![
        "ruff check .".to_string(),
        "ruff format --check .".to_string(),
        "pytest -sv tests".to_string(),
    ];
    assert_eq!(
        dedupe_check_lines(&lines),
        vec!["ruff check .".to_string(), "pytest -sv tests".to_string()]
    );
}

#[test]
fn makefile_gate_targets_reads_lint_and_test() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("Makefile"),
        "lint:\n\tmake check\n\ntest:\n\tpytest -sv tests\n",
    )
    .unwrap();
    assert_eq!(
        makefile_gate_targets(tmp.path()),
        vec!["make check".to_string(), "pytest -sv tests".to_string()]
    );
}

#[test]
fn discover_init_check_commands_prefers_precommit_over_makefile() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join(".pre-commit-config.yaml"),
        "repos:\n- repo: local\n  hooks:\n  - id: ruff\n    entry: ruff check .\n",
    )
    .unwrap();
    std::fs::write(tmp.path().join("Makefile"), "lint:\n\tmake lint-fallback\n").unwrap();
    let lines = discover_init_check_commands(tmp.path());
    assert!(lines.first().is_some_and(|l| l == KISS_CHECK_COMMAND));
    assert!(lines.iter().any(|l| l == "ruff check ."));
    assert!(!lines.iter().any(|l| l.contains("lint-fallback")));
}

#[test]
fn parse_yaml_scalar_strips_single_quotes() {
    assert_eq!(parse_yaml_scalar("'ruff check .'"), "ruff check .");
}

#[test]
fn parse_makefile_targets_skips_blank_and_comment_lines() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("Makefile"), "# header\n\nlint:\n\tmake lint\n").unwrap();
    assert_eq!(
        makefile_gate_targets(tmp.path()),
        vec!["make lint".to_string()]
    );
}

#[test]
fn canonical_tool_lowercases_first_token() {
    assert_eq!(canonical_tool("Cargo test -p foo"), "cargo");
    assert_eq!(canonical_tool(""), "");
}

#[test]
fn ensure_kiss_check_first_moves_existing_kiss_line() {
    let mut lines = vec![
        "ruff check .".to_string(),
        KISS_CHECK_COMMAND.to_string(),
    ];
    ensure_kiss_check_first(&mut lines);
    assert_eq!(lines[0], KISS_CHECK_COMMAND);
    assert_eq!(lines.len(), 2);
}

#[test]
fn discover_init_check_commands_uses_makefile_when_no_precommit() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("Makefile"), "test:\n\tpytest -sv tests\n").unwrap();
    let lines = discover_init_check_commands(tmp.path());
    assert!(lines.first().is_some_and(|l| l == KISS_CHECK_COMMAND));
    assert!(lines.iter().any(|l| l == "pytest -sv tests"));
}

#[test]
fn makefile_gate_targets_skips_whitespace_only_recipe() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("Makefile"), "lint:\n\t   \n").unwrap();
    assert_eq!(makefile_gate_targets(tmp.path()), Vec::<String>::new());
}

#[test]
fn makefile_gate_targets_skips_comment_only_recipe() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("Makefile"), "lint:\n\t# no-op\n").unwrap();
    assert_eq!(makefile_gate_targets(tmp.path()), Vec::<String>::new());
}

#[test]
fn parse_makefile_targets_direct_ignores_unlisted_targets() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("Makefile");
    std::fs::write(&path, "all:\n\techo noop\n").unwrap();
    assert_eq!(parse_makefile_targets(&path), Vec::<String>::new());
}

#[test]
fn parse_yaml_scalar_strips_double_quotes() {
    assert_eq!(parse_yaml_scalar("\"pytest -sv tests\""), "pytest -sv tests");
}

#[test]
fn checks_cover_precommit_signals_true_without_precommit_file() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(checks_cover_precommit_signals(tmp.path(), &["kiss check".to_string()]));
}

#[test]
fn makefile_gate_targets_reads_recipe_after_blank_lines() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("Makefile"), "test:\n\n\n\tpytest -sv tests\n").unwrap();
    assert_eq!(
        makefile_gate_targets(tmp.path()),
        vec!["pytest -sv tests".to_string()]
    );
}

#[test]
fn makefile_gate_targets_reads_gnumakefile() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("GNUmakefile"), "test:\n\tcargo test\n").unwrap();
    assert_eq!(
        makefile_gate_targets(tmp.path()),
        vec!["cargo test".to_string()]
    );
}

#[test]
fn makefile_gate_targets_skips_empty_recipe_line() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("Makefile"), "lint:\n\t\n").unwrap();
    assert_eq!(makefile_gate_targets(tmp.path()), Vec::<String>::new());
}

#[cfg(unix)]
#[test]
fn makefile_gate_targets_returns_empty_when_unreadable() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("Makefile");
    std::fs::write(&path, "test:\n\tcargo test\n").unwrap();
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o0);
    std::fs::set_permissions(&path, perms).unwrap();
    assert_eq!(makefile_gate_targets(tmp.path()), Vec::<String>::new());
}

#[test]
fn makefile_gate_targets_ignores_targets_without_recipe() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("Makefile"), "lint:\nhelp:\n\techo help\n").unwrap();
    assert_eq!(makefile_gate_targets(tmp.path()), Vec::<String>::new());
}

#[test]
fn checks_cover_precommit_signals_detects_missing_hook() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join(".pre-commit-config.yaml"),
        "repos:\n- repo: local\n  hooks:\n  - id: ruff\n    entry: ruff check .\n",
    )
    .unwrap();
    assert!(!checks_cover_precommit_signals(
        tmp.path(),
        &["kiss check".to_string()]
    ));
    assert!(checks_cover_precommit_signals(
        tmp.path(),
        &["kiss check".to_string(), "ruff check .".to_string()]
    ));
}

#[test]
fn finalize_init_checks_from_repo_writes_malvin_checks() {
    if crate::lookup_bin_on_path("kiss").is_none() || crate::lookup_bin_on_path("ruff").is_none() {
        return;
    }
    crate::test_utils::with_isolated_home(|work| {
        std::fs::write(
            work.join(".pre-commit-config.yaml"),
            "repos:\n- repo: local\n  hooks:\n  - id: ruff\n    entry: ruff check .\n",
        )
        .unwrap();
        finalize_init_checks_from_repo(work).unwrap();
        let checks = std::fs::read_to_string(crate::malvin_checks_path(work)).unwrap();
        assert!(checks.contains("kiss check"));
        assert!(checks.contains("ruff check ."));
    });
}
