use std::path::Path;

use super::{git_commit_all, git_init};

pub fn write_repo_files(root: &Path, pairs: &[(&str, &str)]) {
    for (rel, content) in pairs {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).expect("create parent dirs");
            }
        }
        std::fs::write(path, content).expect("write repo file");
    }
}

const ENN_HYBRID_FILES: &[(&str, &str)] = &[
    (
        "rust/Cargo.toml",
        "[package]\nname = \"enn\"\nversion = \"0.1.0\"\n",
    ),
    ("src/foo.py", "x = 1\n"),
    ("tests/test_foo.py", "def test_x():\n    pass\n"),
    (
        ".pre-commit-config.yaml",
        "repos:\n- repo: local\n  hooks:\n  - id: ruff\n    entry: ruff check .\n    language: system\n",
    ),
    (
        "Makefile",
        "lint:\n\tcd rust && cargo clippy --all-targets --all-features -- -D warnings\n\truff check\n\ntest:\n\tpytest -sv tests\n",
    ),
];

pub fn write_enn_hybrid_tree(root: &Path) {
    write_repo_files(root, ENN_HYBRID_FILES);
}

pub fn seed_enn_like_hybrid_fixture(project: &Path) {
    git_init(project);
    write_enn_hybrid_tree(project);
    git_commit_all(project, "seed enn-like hybrid");
}
