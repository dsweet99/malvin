use std::path::PathBuf;

use super::delight_preflight;

#[test]
fn delight_proceeds_when_out_path_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let (resolved, work_dir) = delight_preflight("plan.md").expect("ok");
    std::env::set_current_dir(old).expect("restore");
    assert_eq!(resolved.file_name().unwrap(), "plan.md");
    assert_eq!(work_dir, PathBuf::from("."));
}

#[test]
fn delight_work_dir_is_parent_of_nested_out_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let (_, work_dir) = delight_preflight("plans/delight.md").expect("ok");
    std::env::set_current_dir(old).expect("restore");
    assert!(work_dir.ends_with("plans"));
}

#[test]
fn delight_work_dir_is_dot_for_root_filename() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let (_, work_dir) = delight_preflight("plan.md").expect("ok");
    std::env::set_current_dir(old).expect("restore");
    assert_eq!(work_dir, PathBuf::from("."));
}

#[test]
fn delight_fails_when_out_path_is_regular_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("plan.md"), "existing\n").expect("write");
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let err = delight_preflight("plan.md").expect_err("exists");
    std::env::set_current_dir(old).expect("restore");
    assert!(err.contains("refusing to overwrite"));
}

#[test]
fn delight_fails_when_out_path_is_zero_byte_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("plan.md"), "").expect("write");
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let err = delight_preflight("plan.md").expect_err("exists");
    std::env::set_current_dir(old).expect("restore");
    assert!(err.contains("refusing to overwrite"));
}

#[test]
fn delight_fails_when_out_path_is_directory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("plans")).expect("mkdir");
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let err = delight_preflight("plans").expect_err("exists");
    std::env::set_current_dir(old).expect("restore");
    assert!(err.contains("refusing to overwrite"));
}

#[cfg(unix)]
#[test]
fn delight_fails_when_out_path_is_symlink_to_existing() {
    use std::os::unix::fs::symlink;
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("target.md"), "x\n").expect("write");
    symlink("target.md", tmp.path().join("link.md")).expect("symlink");
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let err = delight_preflight("link.md").expect_err("exists");
    std::env::set_current_dir(old).expect("restore");
    assert!(err.contains("refusing to overwrite"));
}
