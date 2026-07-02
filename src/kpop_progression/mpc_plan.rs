use std::path::Path;

/// Clears a stale `DONE` marker from the mpc plan file before transport/gate retry.
///
/// Read/write failures are ignored so retry can proceed with prior on-disk state.
pub fn strip_mpc_plan_done_on_disk(path: &Path) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    if text.trim() == "DONE" {
        let _ = std::fs::write(path, "");
    }
}

/// True when `path` exists and its trimmed contents are exactly `DONE`.
///
/// # Errors
///
/// Returns `Err` when the file cannot be read.
pub fn mpc_plan_declares_done(path: &Path) -> Result<bool, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => {
            return Err(format!(
                "failed to read mpc plan at {}: {e}",
                path.display()
            ));
        }
    };
    Ok(text.trim() == "DONE")
}

#[cfg(test)]
mod tests {
    use super::{mpc_plan_declares_done, strip_mpc_plan_done_on_disk};

    #[test]
    fn strip_mpc_plan_done_on_disk_clears_done_only() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("mpc_plan.md");
        std::fs::write(&path, "DONE\n").expect("write");
        strip_mpc_plan_done_on_disk(&path);
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "");
        std::fs::write(&path, "# plan\n").expect("write plan");
        strip_mpc_plan_done_on_disk(&path);
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "# plan\n");
    }

    #[test]
    fn mpc_plan_declares_done_requires_exact_done() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("mpc_plan.md");
        assert!(!mpc_plan_declares_done(&path).expect("missing file read"));
        std::fs::write(&path, "DONE\n").expect("write");
        assert!(mpc_plan_declares_done(&path).expect("read"));
        std::fs::write(&path, "DONE ").expect("write");
        assert!(mpc_plan_declares_done(&path).expect("read"));
        std::fs::write(&path, "DONE\nextra\n").expect("write");
        assert!(!mpc_plan_declares_done(&path).expect("read"));
        std::fs::write(&path, "NOT DONE\n").expect("write");
        assert!(!mpc_plan_declares_done(&path).expect("read"));
    }
}
