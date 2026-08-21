use std::io::{BufRead, Write};
use std::process::{Child, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::command_output_timeout::timeout_ms_from_env;

use super::model_list::parse_model_list_page;
use super::{DEFAULT_CODEX_LIST_MODELS_TIMEOUT_MS, ModelListPage, model_list_params};

pub(crate) struct CatalogChild {
    pub(crate) child: Child,
}

impl CatalogChild {
    pub(crate) const fn wrap(child: Child) -> Self {
        Self { child }
    }
}

impl Drop for CatalogChild {
    fn drop(&mut self) {
        reap_catalog_child(&mut self.child);
    }
}

pub(crate) fn list_models_from_child(child: &mut Child) -> Result<Vec<(String, String)>, String> {
    let (mut stdin, stdout) = take_model_server_pipes(child)?;
    let timeout = codex_list_models_timeout();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(read_all_model_pages(&mut stdin, stdout));
    });
    rx.recv_timeout(timeout).unwrap_or_else(|_| {
        Err(format!(
            "codex model/list timed out after {}ms",
            timeout.as_millis()
        ))
    })
}

pub(crate) fn spawn_codex_model_server() -> Result<Child, String> {
    let bin = super::resolve_codex_bin()?;
    let mut cmd = crate::malvin_sandbox::malvin_std_command(&bin);
    cmd.arg("app-server")
        .arg("--stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    cmd.spawn()
        .map_err(|e| format!("spawn codex app-server: {e}"))
}

pub(crate) fn reap_catalog_child(child: &mut Child) {
    crate::acp::signal_process_group(child.id(), 9);
    let _ = child.kill();
    let _ = child.wait();
}

#[must_use]
pub fn codex_list_models_timeout() -> Duration {
    timeout_ms_from_env(
        "MALVIN_CODEX_LIST_MODELS_TIMEOUT_MS",
        DEFAULT_CODEX_LIST_MODELS_TIMEOUT_MS,
    )
}

fn take_model_server_pipes(
    child: &mut Child,
) -> Result<(std::process::ChildStdin, std::process::ChildStdout), String> {
    let stdin = child.stdin.take().ok_or("codex stdin missing")?;
    let stdout = child.stdout.take().ok_or("codex stdout missing")?;
    Ok((stdin, stdout))
}

fn read_all_model_pages(
    stdin: &mut impl Write,
    stdout: impl std::io::Read,
) -> Result<Vec<(String, String)>, String> {
    send_model_list_requests(stdin, None)?;
    let mut stdout = std::io::BufReader::new(stdout);
    let mut all = Vec::new();
    loop {
        let page = read_model_list_page(&mut stdout)?;
        all.extend(page.models);
        let Some(next) = page.next_cursor else {
            return Ok(all);
        };
        send_model_list_request(stdin, Some(&next))?;
    }
}

fn send_model_list_requests(stdin: &mut impl Write, cursor: Option<&str>) -> Result<(), String> {
    let initialize = format!(
        "{}\n",
        serde_json::json!({
            "method": "initialize",
            "id": 1,
            "params": {
                "clientInfo": {
                    "name": "malvin",
                    "title": "Malvin",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        })
    );
    let initialized = "{\"method\":\"initialized\",\"params\":{}}\n";
    let model_list = format!(
        "{}\n",
        serde_json::json!({"method":"model/list","id":2,"params":model_list_params(cursor)})
    );
    stdin
        .write_all(format!("{initialize}{initialized}{model_list}").as_bytes())
        .and_then(|()| stdin.flush())
        .map_err(|error| format!("codex model/list write: {error}"))
}

fn send_model_list_request(stdin: &mut impl Write, cursor: Option<&str>) -> Result<(), String> {
    writeln!(
        stdin,
        "{}",
        serde_json::json!({"method":"model/list","id":2,"params":model_list_params(cursor)})
    )
    .and_then(|()| stdin.flush())
    .map_err(|e| format!("codex model/list write: {e}"))
}

fn read_model_list_page(stdout: &mut impl BufRead) -> Result<ModelListPage, String> {
    let mut line = String::new();
    loop {
        line.clear();
        if stdout.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
            return Err("codex model/list closed stdout".into());
        }
        let value: serde_json::Value =
            serde_json::from_str(&line).map_err(|e| format!("codex model/list JSON: {e}"))?;
        if value.get("id").and_then(serde_json::Value::as_u64) == Some(2) {
            return parse_model_list_page(&value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_pages_from_memory() {
        let mut stdin = Vec::new();
        let stdout = concat!(
            "{\"id\":1}\n",
            "{\"id\":2,\"result\":{\"data\":[{\"id\":\"a\"}],\"nextCursor\":\"n\"}}\n",
            "{\"id\":2,\"result\":{\"data\":[{\"id\":\"b\"}]}}\n"
        );
        let models = read_all_model_pages(&mut stdin, stdout.as_bytes()).unwrap();
        assert_eq!(models[0].0, "a");
        assert_eq!(models[1].0, "b");
        assert!(String::from_utf8_lossy(&stdin).contains("includeHidden"));
    }

    #[test]
    fn kiss_cov_catalog() {
        let _ = spawn_codex_model_server();
        let _ = codex_list_models_timeout();
    }

    #[test]
    fn catalog_page_skips_other_ids_and_closed_stdout() {
        let mut cursor = std::io::Cursor::new("{\"id\":9}\n{\"id\":2,\"result\":{\"data\":[]}}\n");
        let page = read_model_list_page(&mut cursor).unwrap();
        assert!(page.models.is_empty());
        let mut empty = std::io::Cursor::new("");
        assert!(
            read_model_list_page(&mut empty)
                .unwrap_err()
                .contains("closed stdout")
        );
    }

    #[cfg(unix)]
    #[test]
    fn catalog_child_drop_reaps_and_hang_times_out() {
        use std::os::unix::fs::PermissionsExt;
        use std::time::{Duration, Instant};
        let _lock = crate::test_utils::test_env_lock();
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("codex");
        std::fs::write(&p, "#!/bin/sh\nsleep 30\n").unwrap();
        let mut m = std::fs::metadata(&p).unwrap().permissions();
        m.set_mode(0o755);
        std::fs::set_permissions(&p, m).unwrap();
        crate::acp::with_env("MALVIN_CODEX", Some(p.to_str().unwrap()), || {
            crate::acp::with_env("MALVIN_CODEX_LIST_MODELS_TIMEOUT_MS", Some("200"), || {
                let started = Instant::now();
                let mut catalog = CatalogChild::wrap(spawn_codex_model_server().unwrap());
                let err = list_models_from_child(&mut catalog.child).expect_err("timeout");
                assert!(err.contains("timed out"), "got: {err}");
                assert!(started.elapsed() < Duration::from_secs(2));
                reap_catalog_child(&mut catalog.child);
                drop(catalog);
            });
        });
    }
}
