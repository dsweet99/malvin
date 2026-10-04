use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::json;

use super::bridge::{ModalBridge, PACKAGE, ensure_installed, node_bridge_command};
use super::results::{KEPT_PATCH_NAME, PatchOutcome, apply_patch, describe_outcome, import_logs};
use super::workspace::{git_placement, pack_logs, pack_workspace, recent_run_dirs, selected_files};

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
        .status
        .success();
    assert!(ok, "git {args:?}");
}

fn write_all(dir: &Path, files: &[(&str, &str)]) {
    for (name, body) in files {
        fs::write(dir.join(name), body).unwrap();
    }
}

fn repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let d = tmp.path();
    git(d, &["init", "-q"]);
    write_all(d, &[(".gitignore", "ignored.txt\n"), ("a.txt", "one\n"), ("gone.txt", "x\n")]);
    git(d, &["add", "-A"]);
    git(d, &["commit", "-qm", "init"]);
    fs::remove_file(d.join("gone.txt")).unwrap();
    write_all(d, &[("new.txt", "n\n"), ("ignored.txt", "i\n")]);
    tmp
}

fn tar_names(tar: &Path) -> Vec<String> {
    let out = Command::new("tar").arg("-tzf").arg(tar).output().unwrap();
    let mut names: Vec<String> = String::from_utf8_lossy(&out.stdout).lines().map(str::to_string).collect();
    names.sort();
    names
}

#[test]
fn pack_workspace_selects_tracked_and_unignored_files() {
    let tmp = repo();
    let list = selected_files(tmp.path()).unwrap();
    assert!(!String::from_utf8_lossy(&list).contains("gone.txt"));
    let out = tempfile::tempdir().unwrap();
    let tar = out.path().join("w.tar.gz");
    assert!(pack_workspace(tmp.path(), &tar).unwrap());
    assert_eq!(tar_names(&tar), vec![".gitignore", "a.txt", "new.txt"]);
    let placement = git_placement(tmp.path()).unwrap();
    assert_eq!(placement.prefix, "");
}

#[test]
fn pack_workspace_without_git_skips_heavy_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("target")).unwrap();
    fs::write(tmp.path().join("target/big"), "x").unwrap();
    fs::write(tmp.path().join("keep.txt"), "k").unwrap();
    let tar = tmp.path().join("..").join(format!("{}.tgz", std::process::id()));
    assert!(!pack_workspace(tmp.path(), &tar).unwrap());
    let names = tar_names(&tar);
    let _ = fs::remove_file(&tar);
    assert!(names.contains(&"./keep.txt".to_string()), "{names:?}");
    assert!(!names.iter().any(|n| n.contains("target")), "{names:?}");
}

#[test]
fn recent_run_dirs_are_newest_first_and_packable() {
    let tmp = tempfile::tempdir().unwrap();
    for name in ["20260101_000000_a", "20260102_000000_b", "20260103_000000_c", "notes"] {
        fs::create_dir_all(tmp.path().join(name)).unwrap();
    }
    let names = recent_run_dirs(tmp.path(), 2);
    assert_eq!(names, vec!["20260103_000000_c", "20260102_000000_b"]);
    let tar = tmp.path().join("logs.tgz");
    assert!(pack_logs(tmp.path(), &names, &tar).unwrap());
    assert!(!pack_logs(tmp.path(), &[], &tar).unwrap());
    let dest = tempfile::tempdir().unwrap();
    let imported = import_logs(&tar, dest.path()).unwrap();
    assert_eq!(imported, vec![dest.path().join("20260102_000000_b"), dest.path().join("20260103_000000_c")]);
    assert!(recent_run_dirs(&tmp.path().join("missing"), 5).is_empty());
}

fn patch_from(dir: &Path, edit: impl Fn(&Path)) -> Vec<u8> {
    edit(dir);
    git(dir, &["add", "-A"]);
    let out = Command::new("git").args(["diff", "--cached", "--binary", "HEAD"]).current_dir(dir).output().unwrap();
    git(dir, &["reset", "-q", "--hard", "HEAD"]);
    out.stdout
}

#[test]
fn apply_patch_applies_keeps_or_reports_empty() {
    let tmp = repo();
    let d = tmp.path();
    git(d, &["add", "-A"]);
    git(d, &["commit", "-qm", "two"]);
    let patch = patch_from(d, |d| fs::write(d.join("a.txt"), "two\n").unwrap());
    let stage = tempfile::tempdir().unwrap();
    let file = stage.path().join("p.patch");
    fs::write(&file, &patch).unwrap();
    assert_eq!(apply_patch(&file, d, stage.path()).unwrap(), PatchOutcome::Applied);
    assert_eq!(fs::read_to_string(d.join("a.txt")).unwrap(), "two\n");
    fs::write(d.join("a.txt"), "local edit\n").unwrap();
    let outcome = apply_patch(&file, d, stage.path()).unwrap();
    assert!(matches!(outcome, PatchOutcome::Kept(_) | PatchOutcome::Conflicted(_)), "{outcome:?}");
    assert!(stage.path().join(KEPT_PATCH_NAME).is_file());
    fs::write(&file, "").unwrap();
    assert_eq!(apply_patch(&file, d, stage.path()).unwrap(), PatchOutcome::Empty);
    assert!(describe_outcome(&PatchOutcome::Merged).contains("3-way"));
}

#[test]
fn apply_patch_outside_git_keeps_the_patch() {
    let plain = tempfile::tempdir().unwrap();
    let file = plain.path().join("p.patch");
    fs::write(&file, "diff --git a/x b/x\n").unwrap();
    let keep = plain.path().join("run");
    let outcome = apply_patch(&file, plain.path(), &keep).unwrap();
    assert_eq!(outcome, PatchOutcome::Kept(keep.join(KEPT_PATCH_NAME)));
    assert!(describe_outcome(&outcome).contains("not applied"));
}

fn fake_bridge(script: &str) -> ModalBridge {
    let mut cmd = Command::new("sh");
    cmd.args(["-c", script]);
    ModalBridge::spawn(cmd).unwrap()
}

#[test]
fn bridge_call_relays_events_and_maps_replies() {
    let mut ok = fake_bridge(r#"read l; echo '{"id":1,"event":"log","data":"hi"}'; echo '{"id":1,"ok":true,"v":3}'"#);
    assert_eq!(ok.call("x", json!({})).unwrap()["v"], 3);
    let mut bad = fake_bridge(r#"read l; echo '{"id":1,"ok":false,"error":"boom"}'"#);
    assert!(bad.call("y", json!({})).unwrap_err().contains("Modal `y` failed: boom"));
    let mut gone = fake_bridge("exit 0");
    assert!(gone.call("z", json!({})).is_err());
    let _ = gone.exited();
    let mut junk = fake_bridge("read l; echo nope");
    assert!(junk.call("w", json!({})).unwrap_err().contains("bad line"));
}

#[test]
fn bridge_tags_bare_remote_lines_across_chunks() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::enable_stdout_capture();
    let mut b = fake_bridge(concat!(
        r#"read l; printf '%s\n' '{"id":1,"event":"stdout","data":"\nadded 11 pack"}'; "#,
        r#"printf '%s\n' '{"id":1,"event":"stdout","data":"ages in 3s\r\no|kept\ntail"}'; "#,
        r#"printf '%s\n' '{"id":1,"ok":true}'"#
    ));
    b.call("exec", json!({})).unwrap();
    let out = crate::ansi_strip::strip_ansi_escapes(&crate::output::take_captured_stdout());
    assert_eq!(out, "r|\nr|added 11 packages in 3s\nr|tail");
}

#[test]
fn bridge_relays_bare_remote_stdout_as_dm_body_in_do_mode() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::set_do_dm_stdout_mode(true);
    crate::output::enable_stdout_capture();
    let mut b = fake_bridge(concat!(
        r#"read l; printf '%s\n' '{"id":1,"event":"stdout","data":"I am on modal.\n\n- **CPU:** 2\n"}'; "#,
        r#"printf '%s\n' '{"id":1,"event":"stderr","data":"added 11 packages\n"}'; "#,
        r#"printf '%s\n' '{"id":1,"ok":true}'"#
    ));
    b.call("exec", json!({})).unwrap();
    let out = crate::ansi_strip::strip_ansi_escapes(&crate::output::take_captured_stdout());
    crate::output::set_do_dm_stdout_mode(false);
    assert_eq!(out, "I am on modal.\n\n- **CPU:** 2");
}

#[test]
fn remote_lines_with_a_who_tag_are_not_retagged() {
    use super::remote_output::is_tagged;
    for tagged in ["o|modal: started", "e|boom", "b| thinking", "\x1b[90mo|\x1b[0mx", "r|"] {
        assert!(is_tagged(tagged), "{tagged:?}");
    }
    for bare in ["", "added 11 packages in 3s", "| a | b |", "O|x", "1|x", "ab|c", "- **CPU:** x"] {
        assert!(!is_tagged(bare), "{bare:?}");
    }
}

#[test]
fn package_embeds_every_non_test_bridge_file() {
    let dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("modal-bridge/dist");
    let mut on_disk: Vec<String> = fs::read_dir(&dist)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "js"))
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .filter(|n| !n.ends_with("_test.js"))
        .map(|n| format!("dist/{n}"))
        .collect();
    on_disk.sort();
    let mut embedded: Vec<String> = PACKAGE.payload.iter().map(|(r, _)| (*r).to_string()).filter(|r| r.starts_with("dist/")).collect();
    embedded.sort();
    assert_eq!(embedded, on_disk);
    let _ = (ensure_installed as fn() -> _, node_bridge_command as fn(&Path) -> _);
}

#[test]
fn shared_installer_installs_the_modal_package() {
    let _g = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let npm = tmp.path().join("fake-npm");
    fs::write(&npm, "#!/bin/sh\nmkdir -p node_modules/modal && echo '{}' > node_modules/modal/package.json\n").unwrap();
    let mode = std::os::unix::fs::PermissionsExt::from_mode(0o755);
    fs::set_permissions(&npm, mode).unwrap();
    let dest = tmp.path().join("bridge");
    crate::acp::with_env("MALVIN_NPM", Some(npm.to_str().unwrap()), || {
        assert_eq!(PACKAGE.install_into(&dest).unwrap(), dest.join("dist/bridge.js"));
    });
    assert!(PACKAGE.npm_deps_current(&dest));
    assert!(PACKAGE.default_install_dir().ends_with("sdk-bridges/modal-bridge"));
}
