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
pub fn test_fake_command_path(command: &str) -> Option<PathBuf> {
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
#[allow(dead_code)]
const fn test_fake_command_path(_: &str) -> Option<PathBuf> {
    None
}

#[cfg(test)]
pub struct FakeCommandDirGuard {
    pub(crate) previous: Option<PathBuf>,
    pub(crate) thread_id: std::thread::ThreadId,
    pub(crate) _process_lock: Option<std::sync::MutexGuard<'static, ()>>,
}

#[cfg(test)]
fn restore_fake_command_dir_guard(guard: &mut FakeCommandDirGuard) {
    if guard.thread_id == std::thread::current().id() {
        TEST_FAKE_COMMAND_DIR.with(|dir| {
            *dir.borrow_mut() = guard.previous.take().and_then(|p| p.is_dir().then_some(p));
        });
    }
}

#[cfg(test)]
impl Drop for FakeCommandDirGuard {
    fn drop(&mut self) {
        restore_fake_command_dir_guard(self);
    }
}

#[cfg(test)]
pub fn set_fake_command_dir(path: &std::path::Path) -> FakeCommandDirGuard {
    let is_outermost = TEST_FAKE_COMMAND_DIR.with(|dir| dir.borrow().is_none());
    let process_lock = if is_outermost {
        Some(
            FAKE_COMMAND_DIR_MUTEX
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    } else {
        None
    };
    let previous = TEST_FAKE_COMMAND_DIR.with(|dir| {
        let mut guard = dir.borrow_mut();
        guard.replace(path.to_path_buf())
    });
    FakeCommandDirGuard {
        previous,
        thread_id: std::thread::current().id(),
        _process_lock: process_lock,
    }
}

#[cfg(test)]
pub fn run_command_for(command: &str) -> PathBuf {
    test_fake_command_path(command).unwrap_or_else(|| command.into())
}

#[cfg(not(test))]
pub fn apply_fake_path_if_present(_: &mut Command) {}

#[cfg(test)]
#[path = "command_support_tests.rs"]
mod command_support_tests;

#[cfg(all(test, windows))]
#[path = "windows_fake_command_path_tests.rs"]
mod windows_fake_command_path_tests;
