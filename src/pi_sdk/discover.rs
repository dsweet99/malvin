use std::path::{Path, PathBuf};
use std::process::Command;

pub const PI_MISSING_HINT: &str = "pi backend requires the pi binary on PATH (or MALVIN_PI). Install pi_agent_rust’s pi CLI; malvin does not bundle it.";

pub const PI_MIN_VERSION: (u32, u32, u32) = (0, 1, 23);

#[must_use]
pub fn pi_missing_binary_message() -> String {
    PI_MISSING_HINT.to_string()
}

pub fn resolve_pi_bin() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("MALVIN_PI") {
        let path = PathBuf::from(path);
        if !path.is_file() {
            return Err(format!(
                "MALVIN_PI points to a missing file ({}); {PI_MISSING_HINT}",
                path.display()
            ));
        }
        if !pi_path_is_executable(&path) {
            return Err(format!(
                "MALVIN_PI is not executable ({}); {PI_MISSING_HINT}",
                path.display()
            ));
        }
        return Ok(path);
    }
    crate::support_paths::lookup_bin_on_path("pi").ok_or_else(pi_missing_binary_message)
}

#[must_use]
pub(crate) fn pi_path_is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

pub fn pi_version_ok(bin: &Path) -> Result<(), String> {
    let output = Command::new(bin)
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("failed to run `{} --version`: {e}", bin.display()))?;
    if !output.status.success() {
        return Err(format!(
            "`{} --version` exited with {}; {PI_MISSING_HINT}",
            bin.display(),
            output.status
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let ver = parse_pi_version(&text).ok_or_else(|| {
        format!(
            "could not parse `{} --version` output (got {:?}); need pi >= {}.{}.{}",
            bin.display(),
            text.trim(),
            PI_MIN_VERSION.0,
            PI_MIN_VERSION.1,
            PI_MIN_VERSION.2
        )
    })?;
    if ver < PI_MIN_VERSION {
        return Err(format!(
            "pi {}.{}.{} is too old (need >= {}.{}.{}); {PI_MISSING_HINT}",
            ver.0, ver.1, ver.2, PI_MIN_VERSION.0, PI_MIN_VERSION.1, PI_MIN_VERSION.2
        ));
    }
    Ok(())
}

#[must_use]
pub(crate) fn parse_pi_version(text: &str) -> Option<(u32, u32, u32)> {
    let rest = text.lines().next()?.trim().strip_prefix("pi ")?;
    let ver_token = rest.split_whitespace().next()?;
    parse_semver_triple(ver_token)
}

fn parse_semver_triple(ver_token: &str) -> Option<(u32, u32, u32)> {
    let mut parts = ver_token.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = match parts.next() {
        None => 0,
        Some(p) => leading_u32(p)?,
    };
    Some((major, minor, patch))
}
fn leading_u32(s: &str) -> Option<u32> {
    let digits: String = s.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::pi_path_is_executable;

    #[cfg(unix)]
    #[test]
    fn pi_path_is_executable_checks_file_mode() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("pi");
        std::fs::write(&p, "").unwrap();
        let mut permissions = std::fs::metadata(&p).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&p, permissions).unwrap();
        assert!(pi_path_is_executable(&p));
        let mut permissions = std::fs::metadata(&p).unwrap().permissions();
        permissions.set_mode(0o644);
        std::fs::set_permissions(&p, permissions).unwrap();
        assert!(!pi_path_is_executable(&p));
    }
}
