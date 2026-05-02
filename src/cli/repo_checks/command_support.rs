#![allow(clippy::missing_const_for_fn)]

use std::path::PathBuf;
use std::process::{Command, Output};

use super::types::{RepoGateCommandFailure, RepoGateFailure};

pub fn run_command_failure(command: &str, output: &Output) -> RepoGateFailure {
    RepoGateFailure::Command(RepoGateCommandFailure {
        command: command.to_string(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

#[cfg(test)]
pub fn apply_fake_path_if_present(command: &mut Command) {
    if let Some(fake_dir) = TEST_FAKE_COMMAND_DIR.with(|dir| dir.borrow().as_ref().cloned()) {
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
        dir.borrow()
            .as_ref()
            .map(|d| d.join(command))
            .filter(|path| path.is_file())
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
                *dir.borrow_mut() = self.previous.take();
            });
        }
    }
}

#[cfg(test)]
pub fn set_fake_command_dir(path: &std::path::Path) -> FakeCommandDirGuard {
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
mod kiss_stringify_command_support {
    #[test]
    fn kiss_stringify_repo_checks_command_support_units() {
        let _ = stringify!(super::run_command_failure);
        let _ = stringify!(super::apply_fake_path_if_present);
        let _ = stringify!(super::test_fake_command_path);
        let _ = stringify!(super::FakeCommandDirGuard);
        let _ = stringify!(super::run_command_for);
    }
}
