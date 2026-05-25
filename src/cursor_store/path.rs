use std::path::{Path, PathBuf};

pub fn find_store_path(cursor_dir: &Path, session_id: &str) -> Option<PathBuf> {
    let primary = cursor_dir
        .join("acp-sessions")
        .join(session_id)
        .join("store.db");
    if primary.is_file() {
        return Some(primary);
    }
    find_legacy_store_path(cursor_dir, session_id)
}

pub(crate) fn find_legacy_store_path(cursor_dir: &Path, session_id: &str) -> Option<PathBuf> {
    let chats = cursor_dir.join("chats");
    let entries = std::fs::read_dir(chats).ok()?;
    for entry in entries.flatten() {
        let candidate = entry.path().join(session_id).join("store.db");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
