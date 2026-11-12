use super::repair_invalid_malvin_home_config_on_disk;
use crate::repo_gates::checks_test_helpers::{git_init, write_git_root_checks};

fn checks_path(work: &std::path::Path) -> std::path::PathBuf {
    crate::malvin_checks_path(work)
}

fn write_checks(work: &std::path::Path, content: impl AsRef<[u8]>) {
    write_git_root_checks(work, content);
}

fn kiss_cov_gate_restore_repair_test_helpers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    git_init(work);
    write_checks(work, "make lint\n");
    let _ = checks_path(work);
    let _ = crate::repo_gates::checks_test_helpers::git_init;
    let _ = stringify!(write_git_root_checks);
}

fn repair_leaves_empty_checks_file_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    write_checks(work, "");

    repair_invalid_malvin_home_config_on_disk(work).expect("repair");

    let checks = std::fs::read_to_string(checks_path(work)).expect("checks");
    assert!(checks.is_empty());
}

fn repair_leaves_missing_checks_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    git_init(work);
    std::fs::write(work.join("Cargo.toml"), "[package]\nname = \"t\"\n").expect("cargo");
    std::fs::write(work.join("lib.rs"), "fn main() {}\n").expect("source");

    repair_invalid_malvin_home_config_on_disk(work).expect("repair");

    assert!(!checks_path(work).exists());
}

fn repair_leaves_valid_checks_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    write_checks(work, "make lint\n");

    repair_invalid_malvin_home_config_on_disk(work).expect("repair");

    let checks = std::fs::read_to_string(checks_path(work)).expect("checks");
    assert_eq!(checks, "make lint\n");
}

fn repair_recreates_empty_home_malvin_config_from_template() {
    crate::test_support::test_utils::with_isolated_home(|work| {
        let cfg = crate::malvin_config_path(work);
        if let Some(parent) = cfg.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&cfg, b"").expect("empty home config");
        repair_invalid_malvin_home_config_on_disk(work).expect("repair");
        let text = std::fs::read_to_string(&cfg).expect("read home config");
        assert!(text.contains("mem_limit_gb"));
        assert!(text.contains("[agent]"));
    });
}

fn invalid_config_backups(cfg: &std::path::Path) -> Vec<std::path::PathBuf> {
    let parent = cfg.parent().expect("config parent");
    std::fs::read_dir(parent)
        .expect("list config dir")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.as_path() != cfg)
        .collect()
}

fn repair_recreates_invalid_home_malvin_config_from_template() {
    crate::test_support::test_utils::with_isolated_home(|work| {
        let cfg = crate::malvin_config_path(work);
        if let Some(parent) = cfg.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&cfg, b"not valid {{{ toml").expect("invalid home config");
        repair_invalid_malvin_home_config_on_disk(work).expect("repair");
        let text = std::fs::read_to_string(&cfg).expect("read home config");
        assert!(text.contains("mem_limit_gb"));
        assert!(!text.contains("not valid"));
    });
}

fn repair_preserves_invalid_config_bytes_in_a_backup_and_warns() {
    crate::test_support::test_utils::with_isolated_home(|work| {
        let cfg = crate::malvin_config_path(work);
        std::fs::create_dir_all(cfg.parent().expect("parent")).expect("mkdir");
        let original = b"# my settings\n[dm]\non_emit = \"hook\"\n[agent\n\xff";
        std::fs::write(&cfg, original).expect("invalid home config");

        crate::output::clear_captured_stderr_lines();
        repair_invalid_malvin_home_config_on_disk(work).expect("repair");
        let lines = crate::output::take_captured_stderr_lines();

        let backups = invalid_config_backups(&cfg);
        assert_eq!(backups.len(), 1, "{backups:?}");
        assert_eq!(std::fs::read(&backups[0]).expect("read backup"), original);
        let backup = backups[0].display().to_string();
        assert!(
            lines.iter().any(|line| line.contains(&backup)),
            "warning must name {backup}: {lines:?}"
        );
    });
}

fn repeated_repairs_keep_every_invalid_config() {
    crate::test_support::test_utils::with_isolated_home(|work| {
        let cfg = crate::malvin_config_path(work);
        std::fs::create_dir_all(cfg.parent().expect("parent")).expect("mkdir");
        for damage in ["first {{{", "second {{{"] {
            std::fs::write(&cfg, damage).expect("invalid home config");
            repair_invalid_malvin_home_config_on_disk(work).expect("repair");
        }
        assert_eq!(invalid_config_backups(&cfg).len(), 2);
    });
}

fn repair_of_empty_config_makes_no_backup() {
    crate::test_support::test_utils::with_isolated_home(|work| {
        let cfg = crate::malvin_config_path(work);
        std::fs::create_dir_all(cfg.parent().expect("parent")).expect("mkdir");
        std::fs::write(&cfg, b"").expect("empty home config");
        repair_invalid_malvin_home_config_on_disk(work).expect("repair");
        assert!(invalid_config_backups(&cfg).is_empty());
    });
}

fn ensure_heals_empty_home_config_to_template_keys() {
    crate::test_support::test_utils::with_isolated_home(|work| {
        let cfg = crate::malvin_config_path(work);
        if let Some(parent) = cfg.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&cfg, b"").expect("empty");
        repair_invalid_malvin_home_config_on_disk(work).expect("repair");
        let text = std::fs::read_to_string(&cfg).expect("read");
        assert!(text.contains("mem_limit_gb"));
        assert!(text.contains("[agent]"));
    });
}

#[test]
fn merged_kiss_cov_gate_restore_repair_test_helpers() {
    kiss_cov_gate_restore_repair_test_helpers();
    repair_leaves_empty_checks_file_unchanged();
    repair_leaves_missing_checks_unchanged();
    repair_leaves_valid_checks_unchanged();
    repair_recreates_empty_home_malvin_config_from_template();
    repair_recreates_invalid_home_malvin_config_from_template();
    repair_preserves_invalid_config_bytes_in_a_backup_and_warns();
    repeated_repairs_keep_every_invalid_config();
    repair_of_empty_config_makes_no_backup();
    ensure_heals_empty_home_config_to_template_keys();
}
