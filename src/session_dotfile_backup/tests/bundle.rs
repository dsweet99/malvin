use std::path::{Path, PathBuf};

use crate::artifacts::SessionDotfileBackups;
use crate::repo_gates::{KISSCONFIG_FILE, KISSIGNORE_FILE, MALVIN_CHECKS_FILE};
use crate::test_utils::with_isolated_home;

fn workspace_three_paths(work: &Path) -> (PathBuf, PathBuf, PathBuf) {
    (
        work.join(KISSCONFIG_FILE),
        work.join(MALVIN_CHECKS_FILE),
        work.join(KISSIGNORE_FILE),
    )
}

fn seed_pair(work: &Path) {
    std::fs::create_dir_all(work).unwrap();
    let (k, m, _) = workspace_three_paths(work);
    std::fs::write(&k, b"k\n").unwrap();
    std::fs::write(&m, b"m\n").unwrap();
}

#[test]
fn session_snapshot_bundle_round_trip() {
    with_isolated_home(|work| {
        seed_pair(work);
        let bundle = SessionDotfileBackups::snapshot(work).unwrap();
        let (k, m, ki) = workspace_three_paths(work);
        std::fs::write(&k, b"k2\n").unwrap();
        std::fs::write(&m, b"m2\n").unwrap();
        std::fs::write(&ki, b"i\n").unwrap();
        bundle.restore(work).unwrap();
        let k_txt = std::fs::read_to_string(&k).unwrap();
        let m_txt = std::fs::read_to_string(&m).unwrap();
        assert_eq!(k_txt, "k\n");
        assert_eq!(m_txt, "m\n");
        assert!(!ki.exists());
    });
}
