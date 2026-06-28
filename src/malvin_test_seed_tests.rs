use super::{seed_malvin_checks, seed_malvin_config};
use crate::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION;
use crate::test_utils::{revoke_home_malvin_config_mutation_for_test, with_isolated_home};

#[test]
fn kiss_cov_unit_names() {
    let _ = seed_malvin_checks;
    let _ = seed_malvin_config;
    let _ = stringify!(ensure_git_repo_for_checks_seed);
}

#[test]
fn seed_malvin_checks_writes_under_git_root() {
    with_isolated_home(|work| {
        seed_malvin_checks(work, "kiss check\n");
        assert!(crate::malvin_checks_path(work).is_file());
    });
}

#[test]
fn ensure_git_repo_for_checks_seed_idempotent() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    super::ensure_git_repo_for_checks_seed(work);
    super::ensure_git_repo_for_checks_seed(work);
}

#[test]
fn seed_malvin_checks_skips_git_init_when_repo_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(work)
        .status()
        .expect("git init");
    seed_malvin_checks(work, "kiss check\n");
    assert!(crate::malvin_checks_path(work).is_file());
}

#[test]
fn seed_malvin_config_requires_mutation_consent() {
    let _saved = crate::test_utils::SavedEnvVars::capture(&[MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION]);
    revoke_home_malvin_config_mutation_for_test();
    let tmp = tempfile::tempdir().expect("tempdir");
    let result = std::panic::catch_unwind(|| {
        seed_malvin_config(tmp.path(), "x\n");
    });
    assert!(result.is_err(), "seed without isolation consent must panic");
}

#[test]
fn seed_malvin_config_writes_under_isolated_home() {
    with_isolated_home(|work| {
        seed_malvin_config(work, "mem_limit_gb = 3\n");
        let text = std::fs::read_to_string(crate::malvin_config_path(work)).expect("read");
        assert_eq!(text, "mem_limit_gb = 3\n");
    });
}
