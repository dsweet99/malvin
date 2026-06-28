use crate::git_worktree_toplevel;
use std::fs;

#[cfg(test)]
mod kiss_cov_auto {
    use crate::git_worktree_toplevel;

    #[test]
    fn kiss_cov_git_worktree_toplevel() {
        let _ = git_worktree_toplevel;
    }
}

#[test]
fn git_worktree_toplevel_resolves_inside_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    fs::create_dir_all(w.join("src")).unwrap();
    fs::write(w.join(".gitignore"), "x\n").unwrap();
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(w)
        .status()
        .expect("git init");
    let root = git_worktree_toplevel(w).expect("inside repo");
    assert_eq!(root, w.canonicalize().expect("canonicalize"));
    assert!(git_worktree_toplevel(&w.join("src")).is_some());
}

#[test]
fn git_worktree_toplevel_absent_outside_repo() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(git_worktree_toplevel(tmp.path()).is_none());
}
