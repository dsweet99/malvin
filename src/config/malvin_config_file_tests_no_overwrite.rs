use super::{ensure_malvin_config_file, load_malvin_config, open_malvin_config};
use crate::terminal_palette::TerminalTheme;
use crate::test_support::test_utils::with_isolated_home;
use crate::workspace::workspace_paths::malvin_home_config_path;

#[test]
fn ensure_malvin_config_file_and_home_path() {
    with_isolated_home(|_| {
        open_malvin_config().expect("seed");
        let before = std::fs::read_to_string(malvin_home_config_path()).expect("read");
        ensure_malvin_config_file().expect("ensure");
        assert_eq!(
            before,
            std::fs::read_to_string(malvin_home_config_path()).expect("read")
        );
        let path = malvin_home_config_path();
        assert!(path.ends_with(".malvinconf/config.toml"));
        assert!(path.starts_with(crate::user_home_dir()));
    });
}

#[test]
fn open_malvin_config_never_changes_existing_user_mem_limit_gb() {
    with_isolated_home(|_| {
        let path = malvin_home_config_path();
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        let user_text = "mem_limit_gb = 6\ntheme = \"light\"\n";
        std::fs::write(&path, user_text).expect("write");
        let cfg = open_malvin_config().expect("open");
        assert_eq!(cfg.mem_limit_gb, 6);
        assert_eq!(cfg.theme, TerminalTheme::Light);
        assert_eq!(
            std::fs::read_to_string(&path).expect("read"),
            user_text,
            "user-edited config.toml must remain byte-identical"
        );
        let _ = open_malvin_config().expect("reopen");
        assert_eq!(std::fs::read_to_string(&path).expect("read"), user_text);
    });
}

#[test]
fn load_malvin_config_does_not_create_missing_file() {
    with_isolated_home(|_| {
        let path = malvin_home_config_path();
        assert!(!path.exists());
        let cfg = load_malvin_config();
        assert!(!path.exists());
        assert_eq!(
            cfg.agent.model.first().canonical(),
            crate::workspace::support_paths::DEFAULT_CLI_MODEL
        );
    });
}

#[test]
fn ensure_malvin_config_file_if_missing_creates_when_absent() {
    use super::ensure_malvin_config_file_if_missing;

    with_isolated_home(|_| {
        let path = malvin_home_config_path();
        assert!(!path.exists());
        ensure_malvin_config_file_if_missing().expect("create");
        assert!(path.is_file());
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(text.contains("mem_limit_gb"));
    });
}

#[test]
fn ensure_malvin_config_file_if_missing_is_noop_when_present() {
    use super::ensure_malvin_config_file_if_missing;

    with_isolated_home(|_| {
        open_malvin_config().expect("seed");
        let path = malvin_home_config_path();
        let before = std::fs::read_to_string(&path).expect("read");
        ensure_malvin_config_file_if_missing().expect("noop");
        assert_eq!(before, std::fs::read_to_string(&path).expect("read again"));
    });
}

#[test]
fn ensure_malvin_config_file_if_missing_heals_empty_file() {
    use super::ensure_malvin_config_file_if_missing;

    with_isolated_home(|_| {
        let path = malvin_home_config_path();
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, b"").expect("empty");
        ensure_malvin_config_file_if_missing().expect("heal");
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(!text.is_empty());
        assert!(text.contains("mem_limit_gb"));
    });
}

#[test]
fn open_malvin_config_heals_empty_file() {
    with_isolated_home(|_| {
        let path = malvin_home_config_path();
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, b"").expect("empty");
        let cfg = open_malvin_config().expect("open");
        assert!(cfg.mem_limit_gb > 0);
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(!text.is_empty());
        assert!(text.contains("mem_limit_gb"));
    });
}
