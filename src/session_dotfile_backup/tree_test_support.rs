use std::path::Path;

pub(crate) fn init_git_repo(work: &Path) {
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(work)
        .status()
        .expect("git init");
}

#[cfg(test)]
mod kiss_cov {
    use super::init_git_repo;

    #[test]
    fn kiss_witness_tree_test_support_init_git_repo() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let work = tmp.path().join("repo");
        std::fs::create_dir_all(&work).expect("mkdir");
        init_git_repo(&work);
        assert!(work.join(".git").exists());
    }
}
