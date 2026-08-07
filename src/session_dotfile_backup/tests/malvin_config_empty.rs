//! Empty / invalid home-config heal paths (split from `malvin_config.rs` for kiss line limits).

use crate::artifacts::{
    MalvinConfigBackup, restore_workspace_malvin_config_backup, SessionDotfileBackups,
};
use crate::session_dotfile_backup::repair_invalid_malvin_home_config_on_disk;
use crate::test_utils::with_isolated_home;
use crate::{malvin_config_path, MALVIN_HOME_CONFIG_FILE, seed_malvin_config};

#[test]
fn repair_breaks_empty_home_config_on_disk_before_next_snapshot() {
    with_isolated_home(|work| {
        seed_malvin_config(work, "mem_limit_gb = 7\n");
        let cfg = malvin_config_path(work);
        std::fs::write(&cfg, b"").expect("agent truncates home config");
        repair_invalid_malvin_home_config_on_disk(work).expect("repair");
        let restored = std::fs::read_to_string(&cfg).expect("read home config");
        assert!(
            restored.contains("mem_limit_gb"),
            "repair must recreate template defaults, got: {restored:?}"
        );
        assert_ne!(restored, "mem_limit_gb = 7\n", "empty damage replaced with template");
    });
}

#[test]
fn restore_present_empty_home_config_writes_template_not_zero_bytes() {
    with_isolated_home(|work| {
        let cfg = malvin_config_path(work);
        if let Some(parent) = cfg.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&cfg, b"keep-me\n").expect("seed");
        let empty = MalvinConfigBackup::Present(crate::session_dotfile_backup::DotfileBackupPayload {
            backup_path: work.join("empty-slot").join(MALVIN_HOME_CONFIG_FILE),
            bytes: Vec::new(),
        });
        restore_workspace_malvin_config_backup(work, &empty).expect("restore");
        let text = std::fs::read_to_string(&cfg).expect("read");
        assert!(
            !text.is_empty(),
            "Present(empty) must not leave a 0-byte home config"
        );
        assert!(text.contains("mem_limit_gb"), "got: {text:?}");
        assert!(!text.contains("keep-me"));
    });
}

#[test]
fn snapshot_after_ensuring_heals_empty_home_config_before_capture() {
    with_isolated_home(|work| {
        let cfg = malvin_config_path(work);
        if let Some(parent) = cfg.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&cfg, b"").expect("empty");
        let bundle = SessionDotfileBackups::snapshot_after_ensuring_home_config(work).unwrap();
        let MalvinConfigBackup::Present(payload) = &bundle.malvin_config else {
            panic!("expected Present after heal");
        };
        assert!(
            !payload.bytes.is_empty(),
            "snapshot must not capture Present(empty)"
        );
        let on_disk = std::fs::read_to_string(&cfg).expect("read");
        assert!(on_disk.contains("mem_limit_gb"));
    });
}

#[test]
fn empty_snapshot_restore_cycle_does_not_leave_zero_byte_home_config() {
    with_isolated_home(|work| {
        let cfg = malvin_config_path(work);
        if let Some(parent) = cfg.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&cfg, b"").expect("pre-session empty");
        let bundle = SessionDotfileBackups::snapshot_after_ensuring_home_config(work).unwrap();
        std::fs::write(&cfg, b"").expect("mid-session truncate");
        bundle.restore_excluding_malvin_checks(work).unwrap();
        let text = std::fs::read_to_string(&cfg).expect("read after restore");
        assert!(
            !text.is_empty(),
            "full empty→snapshot→truncate→restore cycle must not leave 0 bytes"
        );
        assert!(text.contains("mem_limit_gb"), "got: {text:?}");
    });
}
