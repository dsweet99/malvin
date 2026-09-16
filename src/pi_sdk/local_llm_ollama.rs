use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::local_endpoint::keyless_local_provider_is_listening;
use super::local_llm_paths::ollama_bin;

pub(crate) const SERVE_WAIT: Duration = Duration::from_secs(20);
const SERVE_POLL: Duration = Duration::from_millis(200);

pub(crate) fn process_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

pub(crate) fn stop_pid(pid: u32) {
    let _ = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

pub(crate) fn run_ollama_stop(model: &str) {
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

pub(crate) fn ensure_provider_running(provider: &str) -> Result<Option<u32>, String> {
    if keyless_local_provider_is_listening(provider) {
        return Ok(None);
    }
    if !provider.eq_ignore_ascii_case("ollama") {
        return Err(format!(
            "local provider `{provider}` is not running; malvin can auto-start only `ollama`"
        ));
    }
    let mut child = spawn_ollama_serve()?;
    wait_until_listening(provider, &mut child)?;
    let pid = child.id();
    std::mem::forget(child);
    Ok(Some(pid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_non_ollama_errors_when_down() {
        let err = ensure_provider_running("llamacpp").expect_err("down");
        assert!(err.contains("auto-start only `ollama`"), "{err}");
    }
}
