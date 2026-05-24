//! Follow-on `malvin init` tests (fresh repo commit + existing repo behavior).

mod common;

use common::{
    InitOk, assert_git_head_commit_count, git_show_rev_path, malvin_init_output,
    tempdir_seeded_dirty_keep,
};

#[test]
fn malvin_init_creates_initial_commit_for_fresh_repo() {
    let w = InitOk::new(&["python"]);
    assert_git_head_commit_count(w.path(), "1");
    assert!(
        w.path().join(".malvin/checks").is_file(),
        "init must write .malvin/checks"
    );
    assert!(
        w.path().join(".malvin/advice.md").is_file(),
        "init must write .malvin/advice.md"
    );
}

#[test]
fn malvin_init_does_not_autocommit_preexisting_repo_changes() {
    let project = tempdir_seeded_dirty_keep();
    let out = malvin_init_output(project.path(), &["python"]);
    assert!(out.status.success(), "malvin init failed: {out:?}");
    assert_git_head_commit_count(project.path(), "1");
    assert_eq!(
        git_show_rev_path(project.path(), "HEAD:keep.txt"),
        "before\n",
        "existing tracked content should not be silently rewritten into a new init commit"
    );
}
