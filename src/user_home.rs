//! Home directory resolution (crate-root leaf; no malvin internals).

use std::path::PathBuf;

#[must_use]
pub fn user_home_dir() -> PathBuf {
    if let Some(h) = std::env::var_os("HOME").filter(|s| !s.is_empty()) {
        return PathBuf::from(h);
    }
    if let Some(h) = std::env::var_os("USERPROFILE").filter(|s| !s.is_empty()) {
        return PathBuf::from(h);
    }
    std::env::temp_dir()
}

#[cfg(test)]
mod tests {
    use super::user_home_dir;

    #[test]
    fn user_home_dir_returns_nonempty_path() {
        assert!(!user_home_dir().as_os_str().is_empty());
    }
}
