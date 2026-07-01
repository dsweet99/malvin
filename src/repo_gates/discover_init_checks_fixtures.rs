#[cfg(test)]
use std::path::Path;

#[cfg(test)]
use crate::repo_gates::checks_test_helpers::git_init;

#[cfg(test)]
pub(crate) fn write_repo_files(root: &Path, pairs: &[(&str, &str)]) {
    for (rel, content) in pairs {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).unwrap();
            }
        }
        std::fs::write(path, content).unwrap();
    }
}

#[cfg(test)]
const ENN_HYBRID_FILES: &[(&str, &str)] = &[
    (
        "rust/Cargo.toml",
        "[package]\nname = \"enn\"\nversion = \"0.1.0\"\n",
    ),
    ("src/foo.py", "x = 1\n"),
    ("tests/test_foo.py", "def test_x():\n    pass\n"),
    (
        ".pre-commit-config.yaml",
        "repos:\n- repo: local\n  hooks:\n  - id: ruff\n    entry: ruff check .\n",
    ),
    (
        "Makefile",
        "lint:\n\tcd rust && cargo clippy --all-targets --all-features -- -D warnings\n\truff check\n\ntest:\n\tpytest -sv tests\n",
    ),
];

#[cfg(test)]
pub(crate) fn seed_enn_like_hybrid_fixture(root: &Path) {
    git_init(root);
    write_repo_files(root, ENN_HYBRID_FILES);
}

#[cfg(test)]
const PRECOMMIT_DEDUPE_FILES: &[(&str, &str)] = &[
    ("lib.py", "x = 1\n"),
    (
        ".pre-commit-config.yaml",
        "repos:\n- repo: local\n  hooks:\n  - id: ruff-a\n    entry: ruff check .\n    language: system\n  - id: ruff-b\n    entry: ruff check .\n    language: system\n  - id: compile\n    entry: python3 -m compileall -q .\n    language: system\n",
    ),
    (
        "Makefile",
        "lint:\n\tpython3 -m compileall -q src\n",
    ),
];

#[cfg(test)]
pub(crate) fn seed_precommit_dedupe_fixture(root: &Path) {
    git_init(root);
    write_repo_files(root, PRECOMMIT_DEDUPE_FILES);
}

#[cfg(test)]
pub(crate) fn assert_deduped_precommit_checks(checks: &str) {
    let lines: Vec<&str> = checks
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    assert!(
        lines.first() == Some(&"kiss check"),
        "kiss check must be first; got: {lines:?}"
    );
    assert_eq!(
        lines.iter().filter(|l| **l == "ruff check .").count(),
        1,
        "expected exactly one deduped ruff line; got: {lines:?}"
    );
    assert!(
        lines.contains(&"python3 -m compileall -q ."),
        "expected pre-commit compileall hook; got: {lines:?}"
    );
    assert!(
        !lines.iter().any(|l| l.contains("compileall -q src")),
        "Makefile lint must not override pre-commit signal; got: {lines:?}"
    );
    assert_eq!(
        lines.len(),
        3,
        "expected kiss + deduped ruff + compileall; got: {lines:?}"
    );
}
