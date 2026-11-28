use super::{seed_malvin_checks, seed_malvin_config};
use crate::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION;
use crate::test_support::test_utils::{
    revoke_home_malvin_config_mutation_for_test, with_isolated_home,
};

#[test]
fn kiss_cov_unit_names() {
    let _ = seed_malvin_checks;
    let _ = seed_malvin_config;
}

#[test]
fn seed_malvin_checks_writes_under_git_root() {
    with_isolated_home(|work| {
        seed_malvin_checks(work, "true\n");
        assert!(crate::malvin_checks_path(work).is_file());
    });
}

#[test]
fn ensure_git_repo_is_idempotent_and_git_init_asserts_success() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    crate::test_support::test_utils::ensure_git_repo(work);
    crate::test_support::test_utils::ensure_git_repo(work);
    assert!(crate::git_worktree_toplevel(work).is_some());
    let missing = work.join("missing-dir");
    let failed = std::panic::catch_unwind(|| crate::test_support::test_utils::git_init(&missing));
    assert!(failed.is_err());
}

#[test]
fn seed_malvin_checks_skips_git_init_when_repo_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    crate::test_support::test_utils::git_init(work);
    seed_malvin_checks(work, "true\n");
    assert!(crate::malvin_checks_path(work).is_file());
}

#[test]
fn seed_malvin_config_requires_mutation_consent() {
    let _saved = crate::test_support::test_utils::SavedEnvVars::capture(&[
        MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION,
    ]);
    revoke_home_malvin_config_mutation_for_test();
    let result = std::panic::catch_unwind(|| {
        seed_malvin_config("x\n");
    });
    assert!(result.is_err(), "seed without isolation consent must panic");
}

#[test]
fn seed_malvin_config_writes_under_isolated_home() {
    with_isolated_home(|_| {
        seed_malvin_config("mem_limit_gb = 3\n");
        let text = std::fs::read_to_string(crate::malvin_home_config_path()).expect("read");
        assert_eq!(text, "mem_limit_gb = 3\n");
    });
}
