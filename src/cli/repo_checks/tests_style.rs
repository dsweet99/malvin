use super::{RepoGateOutput, run_repo_workspace_gates};
use super::style_markers::ensure_workspace_style_markers;

#[test]
fn style_markers_are_touched_when_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    ensure_workspace_style_markers(work, RepoGateOutput::Tagged).unwrap();
    let style = work.join(".malvin_memory").join("style.md");
    assert!(style.is_file(), "style.md not created");
    assert_eq!(std::fs::read(&style).unwrap(), Vec::<u8>::new());
}

#[test]
fn style_markers_preserve_existing_content() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    std::fs::create_dir_all(work.join(".malvin_memory")).unwrap();
    std::fs::write(
        work.join(".malvin_memory").join("style.md"),
        b"STYLE STAYS\n",
    )
    .unwrap();
    ensure_workspace_style_markers(work, RepoGateOutput::Tagged).unwrap();
    assert_eq!(
        std::fs::read_to_string(work.join(".malvin_memory").join("style.md")).unwrap(),
        "STYLE STAYS\n"
    );
}

#[test]
fn style_markers_mixed_touch_only_missing_one() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    ensure_workspace_style_markers(work, RepoGateOutput::Stderr).unwrap();
    let style = work.join(".malvin_memory").join("style.md");
    assert!(style.is_file());
    assert_eq!(std::fs::read(&style).unwrap(), Vec::<u8>::new());
}

#[test]
fn style_markers_error_when_style_path_is_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    std::fs::create_dir_all(work.join(".malvin_memory")).unwrap();
    std::fs::create_dir(work.join(".malvin_memory").join("style.md")).unwrap();
    assert!(
        ensure_workspace_style_markers(work, RepoGateOutput::Stderr)
            .unwrap_err()
            .contains("exists but is not a file")
    );
}

#[test]
fn repo_workspace_gates_do_not_create_missing_style_markers() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    run_repo_workspace_gates(work, RepoGateOutput::Stderr, None).unwrap();
    assert!(!work.join(".malvin_memory").join("style.md").exists());
}
