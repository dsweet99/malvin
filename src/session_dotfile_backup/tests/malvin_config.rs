
use crate::artifacts::SessionDotfileBackups;
use crate::malvin_config_file::open_malvin_config;
use crate::test_utils::with_isolated_home;
use crate::{malvin_config_path, seed_malvin_config};

#[test]
fn snapshot_after_ensuring_home_config_creates_file_when_absent() {
    with_isolated_home(|work| {
        let cfg = malvin_config_path(work);
        assert!(!cfg.exists());
        let _bundle = SessionDotfileBackups::snapshot_after_ensuring_home_config(work).unwrap();
        assert!(cfg.is_file(), "ensure must materialize home config");
    });
}

#[test]
fn snapshot_after_ensure_does_not_wipe_user_home_config_on_restore() {
    with_isolated_home(|work| {
        seed_malvin_config(work, "mem_limit_gb = 7\n");
        let cfg = malvin_config_path(work);
        let bundle = SessionDotfileBackups::snapshot_after_ensuring_home_config(work).unwrap();
        seed_malvin_config(work, "TAMPERED\n");
        bundle.restore_excluding_malvin_checks(work).unwrap();
        let after = std::fs::read_to_string(&cfg).unwrap();
        assert!(
            after.contains("TAMPERED"),
            "home config must not be session-restored, got: {after:?}"
        );
    });
}

#[test]
fn snapshot_after_ensure_heals_invalid_on_disk_config() {
    with_isolated_home(|work| {
        seed_malvin_config(work, "mem_limit_gb = 7\n");
        let cfg = malvin_config_path(work);
        std::fs::write(&cfg, b"").unwrap();
        let _ = SessionDotfileBackups::snapshot_after_ensuring_home_config(work).unwrap();
        let healed = std::fs::read_to_string(&cfg).unwrap();
        assert!(
            healed.contains("mem_limit_gb"),
            "empty home config must be healed before snapshot, got: {healed:?}"
        );
    });
}

#[test]
fn open_malvin_config_still_creates_when_missing() {
    with_isolated_home(|work| {
        assert!(!malvin_config_path(work).exists());
        open_malvin_config(work).expect("create ensured default");
        assert!(malvin_config_path(work).is_file());
    });
}
