#![allow(clippy::missing_const_for_fn)]

use std::path::PathBuf;
use std::process::{Command, Output};
#[cfg(test)]
use std::sync::Mutex;

use super::types::{RepoGateCommandFailure, RepoGateFailure};

#[cfg(test)]
static FAKE_COMMAND_DIR_MUTEX: Mutex<()> = Mutex::new(());

pub fn run_command_failure(command: &str, output: &Output) -> RepoGateFailure {
    RepoGateFailure::Command(RepoGateCommandFailure {
        command: command.to_string(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

#[cfg(test)]
fn fake_command_dir_for_path_env() -> Option<PathBuf> {
    TEST_FAKE_COMMAND_DIR.with(|dir| {
        let mut borrowed = dir.borrow_mut();
        match borrowed.as_ref() {
            None => None,
            Some(p) if !p.is_dir() => {
                *borrowed = None;
                None
            }
            Some(p) => Some(p.clone()),
        }
    })
}

#[cfg(test)]
pub fn apply_fake_path_if_present(command: &mut Command) {
    if let Some(fake_dir) = fake_command_dir_for_path_env() {
        let separator = if cfg!(windows) { ';' } else { ':' };
        let path = std::env::var("PATH").unwrap_or_default();
        let mut path_with_fake = fake_dir.display().to_string();
        path_with_fake.push(separator);
        path_with_fake.push_str(&path);
        command.env("PATH", path_with_fake);
    }
}

#[cfg(test)]
thread_local! {
    static TEST_FAKE_COMMAND_DIR: std::cell::RefCell<Option<PathBuf>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn test_fake_command_path(command: &str) -> Option<PathBuf> {
    TEST_FAKE_COMMAND_DIR.with(|dir| {
        let d = {
            let mut borrowed = dir.borrow_mut();
            match borrowed.as_ref() {
                None => return None,
                Some(p) if !p.is_dir() => {
                    *borrowed = None;
                    return None;
                }
                Some(p) => p.clone(),
            }
        };
        let path = d.join(command);
        path.is_file().then_some(path)
    })
}

#[cfg(not(test))]
const fn test_fake_command_path(_: &str) -> Option<PathBuf> {
    None
}

#[cfg(test)]
pub struct FakeCommandDirGuard {
    pub(crate) previous: Option<PathBuf>,
    thread_id: std::thread::ThreadId,
}

#[cfg(test)]
impl Drop for FakeCommandDirGuard {
    fn drop(&mut self) {
        if self.thread_id == std::thread::current().id() {
            TEST_FAKE_COMMAND_DIR.with(|dir| {
                *dir.borrow_mut() = self.previous.take().and_then(|p| p.is_dir().then_some(p));
            });
        }
    }
}

#[cfg(test)]
pub fn set_fake_command_dir(path: &std::path::Path) -> FakeCommandDirGuard {
    let _mutex_guard = FAKE_COMMAND_DIR_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let previous = TEST_FAKE_COMMAND_DIR.with(|dir| {
        let mut guard = dir.borrow_mut();
        guard.replace(path.to_path_buf())
    });
    FakeCommandDirGuard {
        previous,
        thread_id: std::thread::current().id(),
    }
}

pub fn run_command_for(command: &str) -> PathBuf {
    test_fake_command_path(command).unwrap_or_else(|| command.into())
}

#[cfg(not(test))]
pub fn apply_fake_path_if_present(_: &mut Command) {}

#[cfg(test)]
mod command_support_unit_tests {
    use super::{RepoGateCommandFailure, RepoGateFailure, run_command_failure};

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

#[cfg(all(test, windows))]
mod windows_fake_command_path_tests {
    use std::fs;
    use std::process::Command;

    use super::{apply_fake_path_if_present, fake_command_dir_for_path_env, run_command_for, set_fake_command_dir};

    #[test]
    fn fake_command_dir_resolves_batch_command() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().to_path_buf();
        let kiss = p.join("kiss.bat");
        fs::write(&kiss, "@echo off\r\nexit /b 0\r\n").unwrap();
        let _guard = set_fake_command_dir(&p);
        assert_eq!(fake_command_dir_for_path_env(), Some(p.clone()));
        let mut cmd = Command::new("kiss");
        apply_fake_path_if_present(&mut cmd);
        assert_eq!(run_command_for("kiss"), kiss);
    }
}

#[cfg(all(test, unix))]
mod stale_fake_command_path_tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    use super::{TEST_FAKE_COMMAND_DIR, run_command_for, set_fake_command_dir};

    #[test]
    fn test_fake_command_path_none_without_fake_dir() {
        assert_eq!(super::test_fake_command_path("kiss"), None);
    }

    #[test]
    fn fake_command_dir_guard_restores_on_drop() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().to_path_buf();
        {
            let _guard = set_fake_command_dir(&p);
            assert_eq!(super::fake_command_dir_for_path_env(), Some(p));
        }
        assert_eq!(super::fake_command_dir_for_path_env(), None);
    }

    #[test]
    fn nested_fake_command_dir_guards_restore_stack() {
        let tmp1 = tempfile::tempdir().unwrap();
        let tmp2 = tempfile::tempdir().unwrap();
        let p1 = tmp1.path().to_path_buf();
        let p2 = tmp2.path().to_path_buf();
        let guard1: super::FakeCommandDirGuard = set_fake_command_dir(&p1);
        let guard2 = set_fake_command_dir(&p2);
        assert_eq!(super::fake_command_dir_for_path_env(), Some(p2));
        drop(guard2);
        assert_eq!(super::fake_command_dir_for_path_env(), Some(p1));
        drop(guard1);
        assert_eq!(super::fake_command_dir_for_path_env(), None);
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
        assert_eq!(super::fake_command_dir_for_path_env(), Some(p.clone()));
        let mut cmd = Command::new("kiss");
        super::apply_fake_path_if_present(&mut cmd);
        assert_eq!(run_command_for("kiss"), kiss);
        std::mem::drop(tmp);
        assert_eq!(run_command_for("kiss"), std::path::PathBuf::from("kiss"));
        TEST_FAKE_COMMAND_DIR.with(|d| assert!(d.borrow().is_none()));
    }
}
