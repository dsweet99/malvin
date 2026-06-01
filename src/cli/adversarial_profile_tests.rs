use std::path::{Path, PathBuf};

use super::*;

#[test]
fn adversarial_glob_matches_adversarial_filename() {
    assert!(path_matches_adversarial_glob(Path::new("foo_adversarial.md")));
    assert!(path_matches_adversarial_glob(Path::new("adv_system_plan.md")));
    assert!(!path_matches_adversarial_glob(Path::new("plan.md")));
}

#[test]
fn profile_active_when_smell_registry_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join(SMELL_REGISTRY_FILE), "[]").expect("write");
    assert!(adversarial_profile_active(
        tmp.path().join("plan.md").as_path(),
        tmp.path()
    ));
}

#[test]
fn profile_inactive_for_generic_plan() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(!adversarial_profile_active(
        tmp.path().join("plan.md").as_path(),
        tmp.path()
    ));
}

#[test]
fn overlay_hint_none_when_profile_inactive() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    assert!(adversarial_overlay_hint(&plan, tmp.path()).is_none());
}

#[test]
fn resolve_work_dir_for_plan_uses_parent_or_dot() {
    assert_eq!(
        resolve_work_dir_for_plan(Path::new("subdir/plan.md")),
        PathBuf::from("subdir")
    );
    assert_eq!(
        resolve_work_dir_for_plan(Path::new("plan.md")),
        PathBuf::from(".")
    );
}

#[test]
fn overlay_hint_lists_reasons_when_adversarial_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("adversarial.md");
    std::fs::write(&plan, "p").expect("write");
    let hint = adversarial_overlay_hint(&plan, tmp.path()).expect("hint");
    assert!(hint.contains("adversarial glob"));
}
