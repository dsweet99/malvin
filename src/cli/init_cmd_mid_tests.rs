#[cfg(test)]
mod tests {
    use std::process::Command;

    use crate::init_cmd::init_cmd_mid_core::{
        build_pre_commit_config, require_on_path, run_command_expect_success, write_shell_script,
        write_text_file,
    };
    use crate::init_cmd::init_cmd_workspace::ensure_malvin_workspace_layout;
    use crate::init_cmd::{
        emit_init_startup, parse_languages, resolve_init_root, Language, ADMIN_CHECK_UNTRACKED,
        TPL_GITIGNORE,
    };

    #[test]
    fn emit_init_startup_creates_malvin_run_under_root() {
        let tmp = tempfile::tempdir().unwrap();
        let artifacts = emit_init_startup(tmp.path(), false).unwrap();
        assert_eq!(artifacts.work_dir, tmp.path());
        assert!(
            artifacts.run_dir.starts_with(tmp.path().join(".malvin/logs")),
            "init run dir must live under .malvin/logs"
        );
        assert!(artifacts.run_dir.exists());
    }

    #[test]
    fn templates_are_nonempty() {
        assert!(!TPL_GITIGNORE.trim().is_empty());
        assert!(
            ADMIN_CHECK_UNTRACKED.starts_with("#!/bin/bash\n"),
            "check_untracked.sh must have a bash shebang for pre-commit exec"
        );
        assert!(ADMIN_CHECK_UNTRACKED.contains("check_untracked"));
        assert!(ADMIN_CHECK_UNTRACKED.contains("exclude-standard"));
    }

    #[test]
    fn parse_languages_accepts_valid_languages() {
        assert_eq!(
            parse_languages(&["python".into()]).unwrap(),
            vec![Language::Python]
        );
        assert_eq!(
            parse_languages(&["Python".into(), "rust".into()]).unwrap(),
            vec![Language::Python, Language::Rust]
        );
    }

    #[test]
    fn parse_languages_rejects_invalid() {
        assert!(parse_languages(&["javascript".into()]).is_err());
        assert!(parse_languages(&[]).is_err());
    }

    #[test]
    fn build_pre_commit_config_includes_correct_hooks() {
        let py = build_pre_commit_config(&[Language::Python]);
        assert!(py.contains("ruff"));
        assert!(!py.contains("clippy"));
        assert!(py.contains("kiss"));
        let both = build_pre_commit_config(&[Language::Python, Language::Rust]);
        assert!(both.contains("ruff"));
        assert!(both.contains("clippy"));
    }

    #[test]
    fn resolve_init_root_creates_nested_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("a").join("b");
        assert!(resolve_init_root(Some(nested.clone())).is_ok());
        assert!(nested.exists());
    }

    #[test]
    fn require_on_path_finds_existing_binary() {
        require_on_path("ls", "e").unwrap();
    }

    #[test]
    fn require_on_path_fails_for_missing_binary() {
        assert!(require_on_path("nonexistent_xyz_binary_12345", "e").is_err());
    }

    #[test]
    fn run_command_expect_success_detects_failure() {
        run_command_expect_success(&mut Command::new("true"), "ok").unwrap();
        assert!(run_command_expect_success(&mut Command::new("false"), "f").is_err());
    }

    #[test]
    fn write_text_file_respects_force_flag() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("sub").join("f.txt");
        write_text_file(&path, "hello", false).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
        std::fs::write(&path, "orig").unwrap();
        write_text_file(&path, "new", false).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "orig");
        write_text_file(&path, "new", true).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    }

    #[test]
    fn ensure_malvin_workspace_layout_writes_checks_advice_and_logs() {
        let tmp = tempfile::tempdir().unwrap();
        ensure_malvin_workspace_layout(tmp.path(), false, &[Language::Rust]).unwrap();
        assert!(tmp.path().join(".malvin/checks").is_file());
        assert!(tmp.path().join(crate::MALVIN_ADVICE_REL).is_file());
        assert!(tmp.path().join(crate::MALVIN_CONFIG_REL).is_file());
        assert!(tmp.path().join(".malvin/logs").is_dir());
        assert!(tmp.path().join("Cargo.toml").is_file());
    }

    #[test]
    fn ensure_malvin_workspace_layout_sanitizes_cargo_package_name() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("My-Project-2");
        std::fs::create_dir_all(&nested).unwrap();
        ensure_malvin_workspace_layout(&nested, false, &[Language::Rust]).unwrap();
        let toml = std::fs::read_to_string(nested.join("Cargo.toml")).unwrap();
        assert!(toml.contains("name = \"my_project_2\""));
    }

    #[test]
    #[cfg(unix)]
    fn write_shell_script_sets_executable_bit() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let script = tmp.path().join("s.sh");
        write_shell_script(&script, "#!/bin/sh", false).unwrap();
        assert!(std::fs::metadata(&script).unwrap().permissions().mode() & 0o111 != 0);
    }

}
