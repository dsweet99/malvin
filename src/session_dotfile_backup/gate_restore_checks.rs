//! Shared helpers for interpreting `.malvin/checks` bytes during gate restore.

use super::DotfileBackupState;

pub(super) fn substantive_check_lines(bytes: &[u8]) -> Vec<String> {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Vec::new();
    };
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

pub(super) fn is_bare_kiss_check_bytes(bytes: &[u8]) -> bool {
    substantive_check_lines(bytes) == ["kiss".to_string()]
}

pub(super) fn is_invalid_bare_kiss_checks(state: &DotfileBackupState) -> bool {
    match state {
        DotfileBackupState::Present(payload) => is_bare_kiss_check_bytes(&payload.bytes),
        DotfileBackupState::Missing => false,
    }
}

pub(super) fn is_bare_kiss_checks(state: &DotfileBackupState) -> bool {
    let bytes = match state {
        DotfileBackupState::Present(payload) => payload.bytes.as_slice(),
        DotfileBackupState::Missing => return false,
    };
    let lines = substantive_check_lines(bytes);
    lines == ["kiss".to_string()]
        || lines == [crate::repo_gates::KISS_CHECK_COMMAND.to_string()]
}

#[cfg(test)]
mod tests {
    use super::{
        is_bare_kiss_check_bytes, is_bare_kiss_checks, is_invalid_bare_kiss_checks,
        substantive_check_lines,
    };
    use crate::session_dotfile_backup::{DotfileBackupPayload, DotfileBackupState};

    #[test]
    fn substantive_check_lines_skips_comments_and_blank_lines() {
        assert_eq!(
            substantive_check_lines(b"# kiss\n\nkiss check\n"),
            vec!["kiss check".to_string()]
        );
    }

    #[test]
    fn substantive_check_lines_returns_empty_for_non_utf8() {
        assert!(substantive_check_lines(b"\xff\xfe").is_empty());
    }

    #[test]
    fn is_bare_kiss_check_bytes_detects_exact_bare_kiss_line() {
        assert!(is_bare_kiss_check_bytes(b"kiss\n"));
        assert!(!is_bare_kiss_check_bytes(b"kiss check\n"));
    }

    #[test]
    fn is_invalid_bare_kiss_checks_false_for_missing_state() {
        assert!(!is_invalid_bare_kiss_checks(&DotfileBackupState::Missing));
    }

    #[test]
    fn is_invalid_bare_kiss_checks_true_for_bare_kiss_present_state() {
        let state = DotfileBackupState::Present(DotfileBackupPayload {
            backup_path: std::path::PathBuf::from("/tmp/test"),
            bytes: b"kiss\n".to_vec(),
        });
        assert!(is_invalid_bare_kiss_checks(&state));
    }

    #[test]
    fn is_bare_kiss_checks_detects_bare_kiss_and_kiss_check_command() {
        let bare = DotfileBackupState::Present(DotfileBackupPayload {
            backup_path: std::path::PathBuf::from("/tmp/test"),
            bytes: b"kiss\n".to_vec(),
        });
        let kiss_check = DotfileBackupState::Present(DotfileBackupPayload {
            backup_path: std::path::PathBuf::from("/tmp/test"),
            bytes: format!("{}\n", crate::repo_gates::KISS_CHECK_COMMAND).into_bytes(),
        });
        let full = DotfileBackupState::Present(DotfileBackupPayload {
            backup_path: std::path::PathBuf::from("/tmp/test"),
            bytes: b"kiss check\nruff check\n".to_vec(),
        });
        assert!(is_bare_kiss_checks(&bare));
        assert!(is_bare_kiss_checks(&kiss_check));
        assert!(!is_bare_kiss_checks(&DotfileBackupState::Missing));
        assert!(!is_bare_kiss_checks(&full));
    }
}
