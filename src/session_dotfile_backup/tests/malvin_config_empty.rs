use crate::artifacts::SessionDotfileBackups;
use crate::test_utils::with_isolated_home;
use crate::{malvin_config_path, seed_malvin_config};

#[test]
fn repair_breaks_empty_home_config_on_disk_before_next_snapshot() {
    with_isolated_home(|work| {
        seed_malvin_config(work, "mem_limit_gb = 7\n");
        let cfg = malvin_config_path(work);
        std::fs::write(&cfg, b"").expect("agent truncates home config");
        crate::session_dotfile_backup::repair_invalid_malvin_home_config_on_disk(work)
            .expect("repair");
        let restored = std::fs::read_to_string(&cfg).expect("read home config");
        assert!(
            restored.contains("mem_limit_gb"),
            "repair must recreate template defaults, got: {restored:?}"
        );
        assert_ne!(
            restored, "mem_limit_gb = 7\n",
            "empty damage replaced with template"
        );
    });
}

#[test]
fn snapshot_after_ensuring_heals_empty_home_config_before_capture() {
    with_isolated_home(|work| {
        let cfg = malvin_config_path(work);
        if let Some(parent) = cfg.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&cfg, b"").unwrap();
        let _bundle = SessionDotfileBackups::snapshot_after_ensuring_home_config(work).unwrap();
        let text = std::fs::read_to_string(&cfg).expect("read after ensure");
        assert!(
            text.contains("mem_limit_gb"),
            "ensure path must not leave Present(empty) on disk"
        );
    });
}

#[test]
fn empty_home_config_heal_then_user_edit_survives_restore() {
    with_isolated_home(|work| {
        let cfg = malvin_config_path(work);
        if let Some(parent) = cfg.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&cfg, b"").unwrap();
        let bundle = SessionDotfileBackups::snapshot_after_ensuring_home_config(work).unwrap();
        seed_malvin_config(work, "user-mid-session\n");
        bundle.restore_excluding_malvin_checks(work).unwrap();
        let text = std::fs::read_to_string(&cfg).expect("read after restore");
        assert_eq!(
            text, "user-mid-session\n",
            "session restore must not overwrite home config"
        );
    });
}
