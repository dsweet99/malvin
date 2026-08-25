#[cfg(test)]
mod command_support_unit_tests {
    use super::super::{RepoGateCommandFailure, RepoGateFailure, run_command_failure};

    #[test]
    fn run_command_failure_captures_streams() {
        let output = std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: b"stdout-bytes".to_vec(),
            stderr: b"stderr-bytes".to_vec(),
        };
        let RepoGateFailure::Command(RepoGateCommandFailure {
            command,
            stdout,
            stderr,
            ..
        }) = run_command_failure("malvin kiss", &output)
        else {
            panic!("expected command failure");
        };
        assert_eq!(command, "malvin kiss");
        assert!(stdout.contains("stdout-bytes"));
        assert!(stderr.contains("stderr-bytes"));
    }
}

#[cfg(all(test, unix))]
mod stale_fake_command_path_tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    use super::super::{
        FakeCommandDirGuard, TEST_FAKE_COMMAND_DIR, fake_command_dir_for_path_env,
        restore_fake_command_dir_guard, run_command_for, set_fake_command_dir,
    };

    #[test]
    fn restore_fake_command_dir_guard_restores_previous() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut guard: FakeCommandDirGuard = set_fake_command_dir(tmp.path());
        restore_fake_command_dir_guard(&mut guard);
        assert_eq!(fake_command_dir_for_path_env(), None);
    }

    #[test]
    fn test_fake_command_path_none_without_fake_dir() {
        assert_eq!(super::super::test_fake_command_path("kiss"), None);
    }

    #[test]
    fn fake_command_dir_guard_restores_on_drop() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().to_path_buf();
        {
            let _guard = set_fake_command_dir(&p);
            assert_eq!(fake_command_dir_for_path_env(), Some(p));
        }
        assert_eq!(fake_command_dir_for_path_env(), None);
    }

    #[test]
    fn nested_fake_command_dir_guards_restore_stack() {
        let tmp1 = tempfile::tempdir().unwrap();
        let tmp2 = tempfile::tempdir().unwrap();
        let p1 = tmp1.path().to_path_buf();
        let p2 = tmp2.path().to_path_buf();
        let guard1 = set_fake_command_dir(&p1);
        let guard2 = set_fake_command_dir(&p2);
        assert_eq!(fake_command_dir_for_path_env(), Some(p2));
        drop(guard2);
        assert_eq!(fake_command_dir_for_path_env(), Some(p1));
        drop(guard1);
        assert_eq!(fake_command_dir_for_path_env(), None);
    }

    #[test]
    fn removed_fake_dir_is_cleared_and_command_falls_back_to_name() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().to_path_buf();
        let kiss = p.join("kiss");
        fs::write(&kiss, "#!/bin/sh\nexit 0\n").unwrap();
        let mut perms = fs::metadata(&kiss).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&kiss, perms).unwrap();
        let _g = set_fake_command_dir(&p);
        assert_eq!(fake_command_dir_for_path_env(), Some(p.clone()));
        let mut cmd = Command::new("kiss");
        super::super::apply_fake_path_if_present(&mut cmd);
        assert_eq!(run_command_for("kiss"), kiss);
        std::mem::drop(tmp);
        assert_eq!(run_command_for("kiss"), std::path::PathBuf::from("kiss"));
        TEST_FAKE_COMMAND_DIR.with(|d| assert!(d.borrow().is_none()));
    }
}
