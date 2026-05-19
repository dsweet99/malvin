#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_init_startup_creates_malvin_run_under_root() {
        let tmp = tempfile::tempdir().unwrap();
        let artifacts = emit_init_startup(tmp.path(), false).unwrap();
        assert_eq!(artifacts.work_dir, tmp.path());
        assert!(
            artifacts.run_dir.starts_with(tmp.path().join("_malvin")),
            "init run dir must live under _malvin"
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
        assert!(!TPL_STYLE.trim().is_empty());
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
    #[cfg(unix)]
    fn write_shell_script_sets_executable_bit() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let script = tmp.path().join("s.sh");
        write_shell_script(&script, "#!/bin/sh", false).unwrap();
        assert!(std::fs::metadata(&script).unwrap().permissions().mode() & 0o111 != 0);
    }

    #[test]
    fn ensure_branch_is_main_renames_to_main() {
        let tmp = tempfile::tempdir().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(["commit", "--allow-empty", "-m", "i"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["branch", "-M", "main"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        ensure_branch_is_main(tmp.path()).unwrap();
        Command::new("git")
            .args(["branch", "-M", "master"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        ensure_branch_is_main(tmp.path()).unwrap();
        let branch = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&branch.stdout).trim(), "main");
    }
}
