use std::path::{Path, PathBuf};

use crate::artifacts::{
    KissConfigBackup, KissignoreBackup, MalvinChecksBackup, MalvinConfigBackup,
    SessionDotfileBackups,
};
use crate::repo_gates::{KISSCONFIG_FILE, KISSIGNORE_FILE, MALVIN_CHECKS_FILE};
use crate::{MALVIN_CONFIG_REL, seed_malvin_config};
use crate::test_utils::with_isolated_home;

fn workspace_four_paths(work: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    (
        work.join(KISSCONFIG_FILE),
        work.join(MALVIN_CHECKS_FILE),
        work.join(KISSIGNORE_FILE),
        work.join(MALVIN_CONFIG_REL),
    )
}

fn seed_pair(work: &Path) {
    std::fs::create_dir_all(work).unwrap();
    std::fs::create_dir_all(work.join(".malvin")).unwrap();
    let (k, m, _, _c) = workspace_four_paths(work);
    std::fs::write(&k, b"k\n").unwrap();
    std::fs::write(&m, b"m\n").unwrap();
    seed_malvin_config(work, "c\n");
}

#[test]
fn session_snapshot_bundle_round_trip() {
    with_isolated_home(|work| {
        seed_pair(work);
        let bundle = SessionDotfileBackups::snapshot(work).unwrap();
        let (k, m, ki, cfg) = workspace_four_paths(work);
        std::fs::write(&k, b"k2\n").unwrap();
        std::fs::write(&m, b"m2\n").unwrap();
        std::fs::write(&ki, b"i\n").unwrap();
        seed_malvin_config(work, "c2\n");
        bundle.restore(work).unwrap();
        let k_txt = std::fs::read_to_string(&k).unwrap();
        let m_txt = std::fs::read_to_string(&m).unwrap();
        let c_txt = std::fs::read_to_string(&cfg).unwrap();
        assert_eq!(k_txt, "k\n");
        assert_eq!(m_txt, "m\n");
        assert_eq!(c_txt, "c\n");
        assert!(!ki.exists());
    });
}

#[test]
fn restore_session_dotfiles_strips_legacy_root_checks_file() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    std::fs::create_dir_all(work).unwrap();
    std::fs::write(work.join(".malvin_checks"), "legacy\n").unwrap();
    SessionDotfileBackups::from_parts(
        KissConfigBackup::Missing,
        MalvinChecksBackup::Missing,
        KissignoreBackup::Missing,
        MalvinConfigBackup::Missing,
    )
    .restore(work)
    .unwrap();
    assert!(!work.join(".malvin_checks").exists());
}
