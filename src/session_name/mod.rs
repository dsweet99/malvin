//! Per-user session name registry: one live malvin process per `--name`.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::alnum_id::random_alnum;
use crate::user_home_dir;

const NAMES_SUBDIR: &str = "names";
const AUTO_NAME_LEN: usize = 5;
const AUTO_NAME_MAX_DRAWS: usize = 16;
const ACQUIRE_MAX_ATTEMPTS: usize = 4;

#[must_use]
pub fn names_registry_root() -> PathBuf {
    user_home_dir().join(".malvin").join(NAMES_SUBDIR)
}

#[must_use]
pub fn name_path(name: &str) -> PathBuf {
    names_registry_root().join(name)
}

#[must_use]
pub fn parse_holder_pid(contents: &str) -> Option<u32> {
    contents.trim().parse::<u32>().ok()
}

pub fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("session name must not be empty".into());
    }
    if name.len() > 64 {
        return Err("session name must be at most 64 characters".into());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(format!(
            "invalid session name {name:?}: must not contain path separators or .."
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err(format!(
            "invalid session name {name:?}: use letters, digits, and . _ - only"
        ));
    }
    Ok(())
}

fn remove_name_path_best_effort(path: &Path) {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return;
    };
    if metadata.is_dir() {
        let _ = std::fs::remove_dir_all(path);
    } else {
        let _ = std::fs::remove_file(path);
    }
}

enum NameFileState {
    Absent,
    Cleared,
    Holder(u32),
}

fn inspect_name_file(path: &Path) -> NameFileState {
    if !path.exists() {
        return NameFileState::Absent;
    }
    let Ok(metadata) = std::fs::metadata(path) else {
        remove_name_path_best_effort(path);
        return NameFileState::Cleared;
    };
    if !metadata.is_file() {
        remove_name_path_best_effort(path);
        return NameFileState::Cleared;
    }
    let Ok(contents) = std::fs::read_to_string(path) else {
        let _ = std::fs::remove_file(path);
        return NameFileState::Cleared;
    };
    parse_holder_pid(&contents).map_or_else(
        || {
            let _ = std::fs::remove_file(path);
            NameFileState::Cleared
        },
        NameFileState::Holder,
    )
}

fn live_peer_error(name: &str, holder_pid: u32, path: &Path) -> String {
    format!(
        "session name {name:?} held by pid {holder_pid} at {}; another malvin process is already running with this name",
        path.display()
    )
}

#[cfg(unix)]
fn reconcile_foreign_holder(name: &str, holder_pid: u32, path: &Path) -> Result<(), String> {
    if crate::acp::pid_alive(holder_pid) {
        return Err(live_peer_error(name, holder_pid, path));
    }
    let _ = std::fs::remove_file(path);
    Ok(())
}

#[cfg(not(unix))]
fn reconcile_foreign_holder(_name: &str, _holder_pid: u32, path: &Path) -> Result<(), String> {
    let _ = std::fs::remove_file(path);
    Ok(())
}

/// Clear stale or abandoned name files; return `Err` when a live foreign holder remains.
pub fn clear_stale_name_file(name: &str) -> Result<(), String> {
    let path = name_path(name);
    match inspect_name_file(&path) {
        NameFileState::Absent | NameFileState::Cleared => Ok(()),
        NameFileState::Holder(holder_pid) if holder_pid == std::process::id() => Ok(()),
        NameFileState::Holder(holder_pid) => reconcile_foreign_holder(name, holder_pid, &path),
    }
}

fn write_holder_pid(path: &Path, pid: u32) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    writeln!(file, "{pid}")
}

fn ensure_names_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn try_acquire_name_lock(
    name: &str,
    write_holder: impl FnOnce(&Path) -> std::io::Result<()>,
) -> Result<SessionNameGuard, String> {
    let path = name_path(name);
    ensure_names_dir(&path)?;
    for _ in 0..ACQUIRE_MAX_ATTEMPTS {
        clear_stale_name_file(name)?;
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(_) => match write_holder(&path) {
                Ok(()) => return Ok(SessionNameGuard { name: name.to_string() }),
                Err(e) => {
                    let _ = std::fs::remove_file(&path);
                    return Err(e.to_string());
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    clear_stale_name_file(name)?;
    Err(format!(
        "session name {name:?} is already held at {}",
        path.display()
    ))
}

fn lock_name_file(name: &str) -> Result<SessionNameGuard, String> {
    try_acquire_name_lock(name, |path| write_holder_pid(path, std::process::id()))
}

pub fn acquire_name(name: &str) -> Result<SessionNameGuard, String> {
    validate_name(name)?;
    lock_name_file(name)
}

#[cfg(test)]
pub(crate) fn acquire_name_with_write(
    name: &str,
    write: impl FnOnce(&Path) -> std::io::Result<()>,
) -> Result<SessionNameGuard, String> {
    validate_name(name)?;
    try_acquire_name_lock(name, write)
}

pub fn generate_auto_name_with(
    mut draw: impl FnMut(usize) -> String,
) -> Result<(String, SessionNameGuard), String> {
    for draw_index in 0..AUTO_NAME_MAX_DRAWS {
        let name = draw(draw_index);
        if clear_stale_name_file(&name).is_err() {
            continue;
        }
        if let Ok(guard) = lock_name_file(&name) {
            return Ok((name, guard));
        }
    }
    Err(
        "failed to allocate a unique auto-generated session name after 16 attempts; retry or pass --name"
            .into(),
    )
}

pub fn generate_auto_name() -> Result<(String, SessionNameGuard), String> {
    generate_auto_name_with(|_| random_alnum(AUTO_NAME_LEN))
}

pub fn acquire_session_name(opt_name: Option<&str>) -> Result<(String, SessionNameGuard), String> {
    opt_name.map_or_else(generate_auto_name, |name| {
        acquire_name(name).map(|guard| (name.to_string(), guard))
    })
}

pub fn release_name(name: &str) {
    let path = name_path(name);
    if let Ok(contents) = std::fs::read_to_string(&path) {
        if parse_holder_pid(&contents) == Some(std::process::id()) {
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// Cross-process guard: one live malvin process per session name.
pub fn assert_no_peer_name_lock(name: &str) -> Result<(), String> {
    clear_stale_name_file(name)
}

#[derive(Debug)]
pub struct SessionNameGuard {
    name: String,
}

impl SessionNameGuard {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for SessionNameGuard {
    fn drop(&mut self) {
        release_name(&self.name);
    }
}

#[cfg(test)]
mod tests;
