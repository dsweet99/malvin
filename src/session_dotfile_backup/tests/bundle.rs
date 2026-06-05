use std::path::{Path, PathBuf};

use crate::artifacts::{
    KissConfigBackup, KissignoreBackup, MalvinChecksBackup, MalvinConfigBackup,
    SessionDotfileBackups,
};
use crate::repo_gates::{KISSCONFIG_FILE, KISSIGNORE_FILE, MALVIN_CHECKS_FILE};
use crate::{seed_malvin_config};
use crate::test_utils::with_isolated_home;

fn workspace_five_paths(work: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
    (
        work.join(KISSCONFIG_FILE),
        work.join(MALVIN_CHECKS_FILE),
        work.join(KISSIGNORE_FILE),
        crate::malvin_config_path(work),
        work.join(".gitignore"),
    )
}

fn seed_pair(work: &Path) {
    std::fs::create_dir_all(work).unwrap();
    std::fs::create_dir_all(work.join(".malvin")).unwrap();
    let (k, m, ki, _c, gi) = workspace_five_paths(work);
    std::fs::write(&k, b"k\n").unwrap();
    std::fs::write(&m, b"m\n").unwrap();
    std::fs::write(&ki, b"i\n").unwrap();
    std::fs::write(&gi, b"g\n").unwrap();
    seed_malvin_config(work, "c\n");
}

fn assert_five_paths_restored(work: &Path) {
    let (k, m, ki, cfg, gi) = workspace_five_paths(work);
    assert_eq!(std::fs::read_to_string(&k).unwrap(), "k\n");
    assert_eq!(std::fs::read_to_string(&m).unwrap(), "m\n");
    assert_eq!(std::fs::read_to_string(&ki).unwrap(), "i\n");
    assert_eq!(std::fs::read_to_string(&cfg).unwrap(), "c\n");
    assert_eq!(std::fs::read_to_string(&gi).unwrap(), "g\n");
}

#[test]
fn session_snapshot_bundle_round_trip() {
    with_isolated_home(|work| {
        seed_pair(work);
        let bundle = SessionDotfileBackups::snapshot(work).unwrap();
        let (k, m, ki, _cfg, gi) = workspace_five_paths(work);
        std::fs::write(&k, b"k2\n").unwrap();
        std::fs::write(&m, b"m2\n").unwrap();
        std::fs::write(&ki, b"i2\n").unwrap();
        std::fs::write(&gi, b"g2\n").unwrap();
        seed_malvin_config(work, "c2\n");
        bundle.restore(work).unwrap();
        assert_five_paths_restored(work);
    });
}

#[test]
fn restore_excluding_malvin_checks_leaves_checks_unchanged() {
    with_isolated_home(|work| {
        seed_pair(work);
        let bundle = SessionDotfileBackups::snapshot(work).unwrap();
        let (k, m, ki, c, gi) = workspace_five_paths(work);
        std::fs::write(&m, b"agent-edited\n").unwrap();
        std::fs::write(&k, b"k-agent\n").unwrap();
        std::fs::write(&ki, b"i-agent\n").unwrap();
        std::fs::write(&gi, b"g-agent\n").unwrap();
        seed_malvin_config(work, "c-agent\n");
        bundle.restore_excluding_malvin_checks(work).unwrap();
        assert_eq!(std::fs::read_to_string(&m).unwrap(), "agent-edited\n");
        assert_eq!(std::fs::read_to_string(&k).unwrap(), "k\n");
        assert_eq!(std::fs::read_to_string(&ki).unwrap(), "i\n");
        assert_eq!(std::fs::read_to_string(&gi).unwrap(), "g\n");
        assert_eq!(std::fs::read_to_string(&c).unwrap(), "c\n");
        crate::session_dotfile_backup::restore_workspace_session_dotfiles_excluding_malvin_checks(
            work, &bundle,
        )
        .unwrap();
    });
}

#[test]
fn restore_session_dotfiles_strips_legacy_root_checks_file() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    std::fs::create_dir_all(work).unwrap();
    std::fs::write(work.join(".malvin_checks"), "legacy\n").unwrap();
    SessionDotfileBackups::from_parts(crate::session_dotfile_backup::SessionDotfileParts {
        kissconfig: KissConfigBackup::Missing,
        malvin_checks: MalvinChecksBackup::Missing,
        kissignore: KissignoreBackup::Missing,
        malvin_config: MalvinConfigBackup::Missing,
        gitignore: crate::session_dotfile_backup::GitignoreBackup::Missing,
    })
    .restore(work)
    .unwrap();
    assert!(!work.join(".malvin_checks").exists());
}

#[test]
fn gitignore_snapshot_round_trip() {
    with_isolated_home(|work| {
        seed_pair(work);
        let bundle = SessionDotfileBackups::snapshot(work).unwrap();
        let gi = work.join(".gitignore");
        std::fs::write(&gi, b"tampered\n").unwrap();
        bundle.restore(work).unwrap();
        assert_eq!(std::fs::read(&gi).unwrap(), b"g\n");
    });
}

#[test]
fn gitignore_missing_at_snapshot_removes_agent_created_file() {
    with_isolated_home(|work| {
        std::fs::create_dir_all(work).unwrap();
        let bundle = SessionDotfileBackups::snapshot(work).unwrap();
        let gi = work.join(".gitignore");
        std::fs::write(&gi, b"agent-created\n").unwrap();
        bundle.restore(work).unwrap();
        assert!(!gi.exists());
    });
}

#[test]
fn init_discovery_restore_excludes_malvin_checks() {
    with_isolated_home(|work| {
        seed_pair(work);
        let bundle = SessionDotfileBackups::snapshot(work).unwrap();
        let (k, m, ki, c, gi) = workspace_five_paths(work);
        std::fs::write(&m, b"agent-checks\n").unwrap();
        std::fs::write(&k, b"k-agent\n").unwrap();
        std::fs::write(&ki, b"i-agent\n").unwrap();
        std::fs::write(&gi, b"g-agent\n").unwrap();
        seed_malvin_config(work, "c-agent\n");
        bundle.restore_excluding_malvin_checks(work).unwrap();
        assert_eq!(std::fs::read_to_string(&m).unwrap(), "agent-checks\n");
        assert_eq!(std::fs::read_to_string(&k).unwrap(), "k\n");
        assert_eq!(std::fs::read_to_string(&ki).unwrap(), "i\n");
        assert_eq!(std::fs::read_to_string(&gi).unwrap(), "g\n");
        assert_eq!(std::fs::read_to_string(&c).unwrap(), "c\n");
    });
}
