use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;

use super::local_context::LOCAL_LLM_BASE_URL_ENV;
use super::local_endpoint::keyless_local_provider_is_listening;
use super::local_llm_paths::ollama_bin;

pub(crate) const SERVE_WAIT: Duration = Duration::from_secs(20);
const SERVE_POLL: Duration = Duration::from_millis(200);
const SHOW_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Deserialize)]
struct OllamaShowResponse {
    #[serde(default)]
    capabilities: Vec<String>,
}

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

#[must_use]
pub(crate) fn capabilities_include_tools(capabilities: &[String]) -> bool {
    capabilities
        .iter()
        .any(|c| c.eq_ignore_ascii_case("tools"))
}

fn trim_openai_v1_suffix(base: &str) -> String {
    let trimmed = base.trim().trim_end_matches('/');
    trimmed
        .strip_suffix("/v1")
        .unwrap_or(trimmed)
        .to_string()
}

fn ollama_api_root() -> String {
    if let Ok(value) = std::env::var(LOCAL_LLM_BASE_URL_ENV) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trim_openai_v1_suffix(trimmed);
        }
    }
    if let Some(defaults) = pi::provider_metadata::provider_routing_defaults("ollama") {
        return trim_openai_v1_suffix(defaults.base_url);
    }
    "http://127.0.0.1:11434".to_string()
}

fn parse_ollama_show_capabilities(body: &str) -> Option<Vec<String>> {
    serde_json::from_str::<OllamaShowResponse>(body)
        .ok()
        .map(|parsed| parsed.capabilities)
}

async fn fetch_ollama_show_capabilities_async(model: &str) -> Option<Vec<String>> {
    let url = format!("{}/api/show", ollama_api_root());
    let client = pi::http::client::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .timeout(SHOW_TIMEOUT)
        .json(&serde_json::json!({ "name": model }))
        .ok()?
        .send()
        .await
        .ok()?;
    if !(200..300).contains(&response.status()) {
        return None;
    }
    let body = response.text().await.ok()?;
    parse_ollama_show_capabilities(&body)
}

#[must_use]
pub(crate) fn ollama_model_supports_tools(model: &str) -> Option<bool> {
    let Ok(runtime) = asupersync::runtime::RuntimeBuilder::current_thread().build() else {
        return None;
    };
    let caps = runtime.block_on(fetch_ollama_show_capabilities_async(model))?;
    Some(capabilities_include_tools(&caps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_non_ollama_errors_when_down() {
        let err = ensure_provider_running("llamacpp").expect_err("down");
        assert!(err.contains("auto-start only `ollama`"), "{err}");
    }

    #[test]
    fn capabilities_include_tools_is_case_insensitive() {
        assert!(!capabilities_include_tools(&[]));
        assert!(!capabilities_include_tools(&[
            "completion".into(),
            "insert".into()
        ]));
        assert!(capabilities_include_tools(&[
            "completion".into(),
            "Tools".into()
        ]));
    }

    #[test]
    fn parse_show_capabilities_reads_tools_flag() {
        let phi = r#"{"capabilities":["completion"]}"#;
        let qwen = r#"{"capabilities":["completion","tools","insert"]}"#;
        assert!(!capabilities_include_tools(
            &parse_ollama_show_capabilities(phi).expect("phi")
        ));
        assert!(capabilities_include_tools(
            &parse_ollama_show_capabilities(qwen).expect("qwen")
        ));
        assert!(parse_ollama_show_capabilities("not-json").is_none());
    }

    #[test]
    fn trim_openai_v1_suffix_strips_trailing_v1() {
        assert_eq!(
            trim_openai_v1_suffix("http://127.0.0.1:11434/v1/"),
            "http://127.0.0.1:11434"
        );
        assert_eq!(
            trim_openai_v1_suffix("http://host:11434"),
            "http://host:11434"
        );
    }
}
