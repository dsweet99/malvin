//! Integration-style tests for [`super::EditEfficiencyMeter`].
#![allow(clippy::float_cmp)]

use std::process::Command;

use tempfile::TempDir;

use super::EditEfficiencyMeter;

fn repo_with_git() -> TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    let p = tmp.path();
    assert!(Command::new("git")
        .args(["init", "-q"])
        .current_dir(p)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args(["config", "user.email", "t@e.st"])
        .current_dir(p)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args(["config", "user.name", "t"])
        .current_dir(p)
        .status()
        .unwrap()
        .success());
    tmp
}

#[test]
fn thrash_increases_gross_net_zero_efficiency_zero() {
    let tmp = repo_with_git();
    let p = tmp.path();
    std::fs::write(p.join("x.rs"), b"aaa").unwrap();
    let mut m = EditEfficiencyMeter::new(p).unwrap();
    std::fs::write(p.join("x.rs"), b"bbb").unwrap();
    m.checkpoint().unwrap();
    std::fs::write(p.join("x.rs"), b"aaa").unwrap();
    m.checkpoint().unwrap();
    let r = m.finish().unwrap();
    assert_eq!(r.net_bytes, 0);
    assert!(r.gross_bytes > 0);
    assert_eq!(r.efficiency, 0.0);
}

#[test]
fn single_edit_efficiency_one() {
    let tmp = repo_with_git();
    let p = tmp.path();
    std::fs::write(p.join("f.rs"), b"hello").unwrap();
    let mut m = EditEfficiencyMeter::new(p).unwrap();
    std::fs::write(p.join("f.rs"), b"hello!").unwrap();
    m.checkpoint().unwrap();
    let r = m.finish().unwrap();
    assert_eq!(r.gross_bytes, r.net_bytes);
    assert_eq!(r.efficiency, 1.0);
}

/// `finish()` must record a final uncheckpointed tree diff in `gross_diff_steps` but not in
/// `checkpoint_calls`.
#[test]
fn finish_tail_increments_gross_diff_steps_not_checkpoint_calls() {
    let tmp = repo_with_git();
    let p = tmp.path();
    std::fs::write(p.join("a.rs"), b"v1").unwrap();
    let m = EditEfficiencyMeter::new(p).unwrap();
    std::fs::write(p.join("a.rs"), b"v2").unwrap();
    let r = m.finish().unwrap();
    assert!(r.gross_bytes > 0, "uncheckpointed edit must contribute via finish() tail");
    assert_eq!(r.checkpoint_calls, 0);
    assert_eq!(r.gross_diff_steps, 1);
}

#[test]
fn ignores_non_measured_files() {
    let tmp = repo_with_git();
    let p = tmp.path();
    std::fs::write(p.join("n.txt"), b"zzz").unwrap();
    let mut m = EditEfficiencyMeter::new(p).unwrap();
    std::fs::write(p.join("n.txt"), b"qqq").unwrap();
    m.checkpoint().unwrap();
    let r = m.finish().unwrap();
    assert_eq!(r.gross_bytes, 0);
    assert_eq!(r.net_bytes, 0);
    assert_eq!(r.efficiency, 1.0);
}
