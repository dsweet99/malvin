use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::local_endpoint::keyless_local_provider_is_listening;

const RUNTIME_MARKER: &str = "local_llm_runtime.json";
const RUNTIME_PATH_ENV: &str = "MALVIN_LOCAL_LLM_RUNTIME_PATH";
const OLLAMA_BIN_ENV: &str = "MALVIN_OLLAMA";
const SERVE_WAIT: Duration = Duration::from_secs(20);
const SERVE_POLL: Duration = Duration::from_millis(200);

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct RuntimeState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ollama_serve_pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    active_model: Option<String>,
}

fn runtime_path() -> PathBuf {
    if let Ok(path) = std::env::var(RUNTIME_PATH_ENV) {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    crate::workspace_paths::malvin_user_home_root().join(RUNTIME_MARKER)
}

fn load_state(path: &Path) -> RuntimeState {
    let Ok(body) = fs::read_to_string(path) else {
        return RuntimeState::default();
    };
    serde_json::from_str(&body).unwrap_or_default()
}

fn save_state(path: &Path, state: &RuntimeState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let body = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

fn clear_state(path: &Path) {
    let _ = fs::remove_file(path);
}

fn ollama_bin() -> String {
    std::env::var(OLLAMA_BIN_ENV)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "ollama".to_string())
}

fn process_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn stop_pid(pid: u32) {
    let _ = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn run_ollama_stop(model: &str) {
    let _ = Command::new(ollama_bin())
        .args(["stop", model])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn spawn_ollama_serve() -> Result<Child, String> {
    Command::new(ollama_bin())
        .arg("serve")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to start `ollama serve`: {e}"))
}

fn wait_until_listening(provider: &str, child: &mut Child) -> Result<(), String> {
    let deadline = Instant::now() + SERVE_WAIT;
    while Instant::now() < deadline {
        if keyless_local_provider_is_listening(provider) {
            return Ok(());
        }
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("`ollama serve` exited early ({status})"));
        }
        thread::sleep(SERVE_POLL);
    }
    let _ = child.kill();
    let _ = child.wait();
    Err("`ollama serve` did not become reachable in time".into())
}

pub(crate) fn ensure_local_provider_running(provider: &str) -> Result<(), String> {
    if keyless_local_provider_is_listening(provider) {
        return Ok(());
    }
    if !provider.eq_ignore_ascii_case("ollama") {
        return Err(format!(
            "local provider `{provider}` is not running; malvin can auto-start only `ollama`"
        ));
    }
    let path = runtime_path();
    let mut state = load_state(&path);
    let mut child = spawn_ollama_serve()?;
    wait_until_listening(provider, &mut child)?;
    state.ollama_serve_pid = Some(child.id());
    std::mem::forget(child);
    save_state(&path, &state)
}

pub(crate) fn note_local_model_in_use(model: &str) -> Result<(), String> {
    let path = runtime_path();
    let mut state = load_state(&path);
    state.active_model = Some(model.to_string());
    save_state(&path, &state)
}

pub fn housekeep_local_llms(needs_local: bool) {
    if needs_local {
        return;
    }
    let path = runtime_path();
    let state = load_state(&path);
    if state.active_model.is_none() && state.ollama_serve_pid.is_none() {
        return;
    }
    if let Some(model) = state.active_model.as_deref() {
        run_ollama_stop(model);
    }
    if let Some(pid) = state.ollama_serve_pid
        && process_alive(pid)
    {
        stop_pid(pid);
    }
    clear_state(&path);
}

#[must_use]
pub fn model_needs_local_llm(model: &crate::model_id::ParsedModel) -> bool {
    model
        .pi_provider_and_model()
        .is_some_and(|(provider, _)| pi::provider_metadata::provider_is_keyless_local(provider))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_id::parse_model_id;

    #[test]
    fn model_needs_local_llm_detects_ollama_and_skips_cursor() {
        let local = parse_model_id("rpi:ollama/tiny").expect("parse");
        let cursor = parse_model_id("cursor:auto").expect("parse");
        assert!(model_needs_local_llm(&local));
        assert!(!model_needs_local_llm(&cursor));
    }

    #[test]
    fn housekeep_clears_marker_and_skips_when_needed() {
        let tmp = tempfile::tempdir().expect("tmp");
        let marker = tmp.path().join("runtime.json");
        crate::acp::with_env(RUNTIME_PATH_ENV, Some(marker.to_str().unwrap()), || {
            let state = RuntimeState {
                ollama_serve_pid: Some(9_999_999),
                active_model: Some("toy".into()),
            };
            save_state(&marker, &state).expect("save");
            housekeep_local_llms(true);
            assert!(marker.is_file(), "must keep marker when local still needed");
            housekeep_local_llms(false);
            assert!(
                !marker.is_file(),
                "must clear marker after idle housekeeping"
            );
        });
    }

    #[test]
    fn ensure_non_ollama_errors_when_down() {
        let err = ensure_local_provider_running("llamacpp").expect_err("down");
        assert!(err.contains("auto-start only `ollama`"), "{err}");
    }
}
