use std::io::{BufRead, Write};
use std::path::PathBuf;

pub const CODEX_MISSING_HINT: &str = "codex backend requires the codex binary on PATH (or MALVIN_CODEX); install Codex CLI separately.";

#[must_use]
pub fn codex_missing_binary_message() -> String {
    CODEX_MISSING_HINT.to_string()
}

pub fn resolve_codex_bin() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("MALVIN_CODEX") {
        let path = PathBuf::from(path);
        if !path.is_file() {
            return Err(format!(
                "MALVIN_CODEX points to a missing file ({}); {CODEX_MISSING_HINT}",
                path.display()
            ));
        }
        if !codex_path_is_executable(&path) {
            return Err(format!(
                "MALVIN_CODEX is not executable ({}); {CODEX_MISSING_HINT}",
                path.display()
            ));
        }
        return Ok(path);
    }
    crate::support_paths::lookup_bin_on_path("codex").ok_or_else(codex_missing_binary_message)
}

#[must_use]
pub(crate) fn codex_path_is_executable(path: &std::path::Path) -> bool {
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

pub fn list_codex_models() -> Result<Vec<(String, String)>, String> {
    let mut child = spawn_codex_model_server()?;
    let (mut stdin, stdout) = take_model_server_pipes(&mut child)?;
    send_model_list_requests(&mut stdin)?;
    let result = read_model_list_response(stdout)?;
    let _ = child.kill();
    Ok(result)
}

fn spawn_codex_model_server() -> Result<std::process::Child, String> {
    let bin = resolve_codex_bin()?;
    std::process::Command::new(bin)
        .arg("app-server")
        .arg("--stdio")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn codex app-server: {e}"))
}

fn take_model_server_pipes(
    child: &mut std::process::Child,
 ) -> Result<(std::process::ChildStdin, std::process::ChildStdout), String> {
    let stdin = child.stdin.take().ok_or("codex stdin missing")?;
    let stdout = child.stdout.take().ok_or("codex stdout missing")?;
    Ok((stdin, stdout))
}

fn send_model_list_requests(stdin: &mut impl Write) -> Result<(), String> {
    writeln!(stdin, r#"{{"method":"initialize","id":1,"params":{{"clientInfo":{{"name":"malvin","title":"Malvin","version":"{}"}}}}}}"#, env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string())?;
    writeln!(stdin, r#"{{"method":"initialized","params":{{}}}}"#).map_err(|e| e.to_string())?;
    writeln!(stdin, r#"{{"method":"model/list","id":2,"params":{{"limit":100,"includeHidden":false}}}}"#).map_err(|e| e.to_string())?;
    stdin.flush().map_err(|e| e.to_string())
}

fn read_model_list_response(stdout: impl std::io::Read) -> Result<Vec<(String, String)>, String> {
    let mut stdout = std::io::BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        if stdout.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
            return Err("codex model/list closed stdout".into());
        }
        let value: serde_json::Value = serde_json::from_str(&line).map_err(|e| format!("codex model/list JSON: {e}"))?;
        if value.get("id").and_then(serde_json::Value::as_u64) == Some(2) {
            return parse_model_list_response(&value);
        }
    }
}

fn parse_model_list_response(value: &serde_json::Value) -> Result<Vec<(String, String)>, String> {
    if let Some(error) = value.get("error") {
        return Err(format!("codex model/list: {error}"));
    }
    let rows = value.pointer("/result/data").and_then(serde_json::Value::as_array).ok_or("codex model/list response missing data")?;
    Ok(rows.iter().filter_map(|row| Some((row.get("id")?.as_str()?.to_owned(), row.get("displayName").and_then(serde_json::Value::as_str).unwrap_or_default().to_owned()))).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[cfg(unix)]
    #[test]
    fn test_list_codex_models() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("codex");
        std::fs::write(
            &p,
            "#!/bin/sh\nprintf '%s\\n' '{\"id\":1,\"result\":{}}' '{\"id\":2,\"result\":{\"data\":[{\"id\":\"gpt-test\",\"displayName\":\"Test\"}]}}'\n",
        )
        .unwrap();
        let mut m = std::fs::metadata(&p).unwrap().permissions();
        m.set_mode(0o755);
        std::fs::set_permissions(&p, m).unwrap();
        crate::acp::with_env("MALVIN_CODEX", Some(p.to_str().unwrap()), || {
            assert_eq!(list_codex_models().unwrap(), vec![("gpt-test".into(), "Test".into())]);
        });
    }

    #[test]
    fn test_codex_missing_binary_message() {
        assert!(codex_missing_binary_message().contains("MALVIN_CODEX"));
    }

    #[cfg(unix)]
    #[test]
    fn codex_path_is_executable_checks_file_mode() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("codex");
        std::fs::write(&p, "").unwrap();
        let mut permissions = std::fs::metadata(&p).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&p, permissions).unwrap();
        assert!(codex_path_is_executable(&p));
        let mut permissions = std::fs::metadata(&p).unwrap().permissions();
        permissions.set_mode(0o644);
        std::fs::set_permissions(&p, permissions).unwrap();
        assert!(!codex_path_is_executable(&p));
    }

    #[cfg(unix)]
    #[test]
    fn test_codex_path_is_executable() {
        let _ = codex_path_is_executable;
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("codex");
        std::fs::write(&p, "#!/bin/sh\n").unwrap();
        let mut m = std::fs::metadata(&p).unwrap().permissions();
        m.set_mode(0o755);
        std::fs::set_permissions(&p, m).unwrap();
        assert!(codex_path_is_executable(&p));
        let mut m = std::fs::metadata(&p).unwrap().permissions();
        m.set_mode(0o644);
        std::fs::set_permissions(&p, m).unwrap();
        assert!(!codex_path_is_executable(&p));
        assert!(!codex_path_is_executable(&d.path().join("missing")));
    }
}
