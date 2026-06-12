use super::{
    ensure_malvin_config_file, load_malvin_config, open_malvin_config,
};
use crate::terminal_palette::TerminalTheme;
use crate::test_utils::with_isolated_home;
use crate::workspace_paths::{malvin_config_path, malvin_home_config_path};

#[test]
fn ensure_malvin_config_file_and_home_path() {
    with_isolated_home(|work| {
        open_malvin_config(work).expect("seed");
        let before = std::fs::read_to_string(malvin_config_path(work)).expect("read");
        ensure_malvin_config_file(work).expect("ensure");
        assert_eq!(
            before,
            std::fs::read_to_string(malvin_config_path(work)).expect("read")
        );
        let path = malvin_home_config_path();
        assert!(path.ends_with(".malvin_home/config.toml"));
        assert!(path.starts_with(crate::user_home_dir()));
    });
}

#[test]
fn open_malvin_config_never_changes_existing_user_mem_limit_gb() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        let user_text = "mem_limit_gb = 6\ntheme = \"light\"\n";
        std::fs::write(&path, user_text).expect("write");
        let cfg = open_malvin_config(work).expect("open");
        assert_eq!(cfg.mem_limit_gb, 6);
        assert_eq!(cfg.theme, TerminalTheme::Light);
        assert_eq!(
            std::fs::read_to_string(&path).expect("read"),
            user_text,
            "user-edited config.toml must remain byte-identical"
        );
        let _ = open_malvin_config(work).expect("reopen");
        assert_eq!(
            std::fs::read_to_string(&path).expect("read"),
            user_text
        );
    });
}

#[test]
fn load_malvin_config_does_not_create_missing_file() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        assert!(!path.exists());
        let cfg = load_malvin_config(work);
        assert!(!path.exists());
        assert_eq!(cfg.agent.max_hypotheses, super::DEFAULT_MAX_HYPOTHESES);
    });
}
