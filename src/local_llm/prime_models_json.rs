//! Temporary `models.json` so Prime can resolve `local/local/<slug>` to a sidecar.

use std::path::Path;

use serde_json::json;

/// Writes a Prime `models.json` pointing provider `local` at `base_url`.
///
/// # Errors
///
/// Returns an error when the file cannot be written.
pub fn write_prime_local_models_json(
    path: &Path,
    base_url: &str,
    model_id: &str,
    display_name: &str,
) -> Result<(), String> {
    let doc = json!({
        "providers": {
            "local": {
                "baseUrl": base_url,
                "api": "openai-completions",
                "apiKey": "local",
                "compat": {
                    "supportsDeveloperRole": false,
                    "supportsReasoningEffort": false,
                    "supportsUsageInStreaming": false,
                    "maxTokensField": "max_tokens"
                },
                "models": [{
                    "id": model_id,
                    "name": display_name,
                    "reasoning": false
                }]
            }
        }
    });
    let text = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| format!("write models.json: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_models_json_contains_provider_and_model() {
        let dir = tempfile::tempdir().expect("td");
        let path = dir.path().join("models.json");
        write_prime_local_models_json(
            &path,
            "http://127.0.0.1:9/v1",
            "local/qwen35_9b_q4",
            "Qwen",
        )
        .expect("write");
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(text.contains("\"local\""));
        assert!(text.contains("local/qwen35_9b_q4"));
        assert!(text.contains("http://127.0.0.1:9/v1"));
    }
}
