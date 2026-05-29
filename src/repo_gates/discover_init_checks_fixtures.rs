#[cfg(test)]
use std::path::Path;

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
    write_repo_files(root, ENN_HYBRID_FILES);
}
