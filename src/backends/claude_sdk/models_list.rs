use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use serde_json::Value;

const CACHE_FILE: &str = "claude_models.json";
const LIST_SCRIPT: &str = "list_models.js";
const LIST_TIMEOUT: Duration = Duration::from_mins(1);
const DEFAULT_EFFORT: &str = "low|medium|high|xhigh|max";

#[must_use]
pub fn claude_models_cache_path() -> PathBuf {
    crate::workspace::workspace_paths::malvin_user_home_root().join(CACHE_FILE)
}

pub fn refresh_claude_models() -> Result<(), String> {
    let bridge = super::claude_sdk::bridge_path::resolve_bridge_js()?;
    let script = bridge.with_file_name(LIST_SCRIPT);
    if !script.is_file() {
        return Err(format!(
            "Claude model-list script missing: {}",
            script.display()
        ));
    }
    let node = crate::backends::cursor_sdk::node_resolve::resolve_node_bin()?;
    let mut cmd = Command::new(node);
    cmd.arg(script);
    let output = crate::command_output_timeout::command_output_with_timeout(
        cmd,
        LIST_TIMEOUT,
        "claude model list",
    )?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "claude model list failed: {}",
            err.trim().chars().take(240).collect::<String>()
        ));
    }
    let body = String::from_utf8_lossy(&output.stdout).into_owned();
    if parse_models(&body).is_empty() {
        return Err("claude model list returned no models".into());
    }
    write_cache(&body)
}

fn write_cache(body: &str) -> Result<(), String> {
    let path = claude_models_cache_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

#[must_use]
pub fn list_claude_display_models() -> Vec<(String, String)> {
    let cached = std::fs::read_to_string(claude_models_cache_path())
        .map(|body| parse_models(&body))
        .unwrap_or_default();
    if cached.is_empty() {
        builtin_models()
    } else {
        cached
    }
}

#[must_use]
pub fn builtin_models() -> Vec<(String, String)> {
    vec![
        (
            "sonnet".into(),
            format!("Claude Sonnet effort={DEFAULT_EFFORT}"),
        ),
        (
            "opus".into(),
            format!("Claude Opus effort={DEFAULT_EFFORT} fast=true|false"),
        ),
        (
            "haiku".into(),
            format!("Claude Haiku effort={DEFAULT_EFFORT}"),
        ),
    ]
}

fn parse_models(body: &str) -> Vec<(String, String)> {
    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(body) else {
        return Vec::new();
    };
    items.iter().filter_map(model_row).collect()
}

fn model_row(item: &Value) -> Option<(String, String)> {
    let id = item.get("value")?.as_str()?;
    if id.is_empty() || id == "default" {
        return None;
    }
    let name = item
        .get("displayName")
        .and_then(Value::as_str)
        .unwrap_or(id);
    let mut detail = format!("Claude {name}");
    if let Some(levels) = item.get("supportedEffortLevels").and_then(Value::as_array) {
        let levels: Vec<&str> = levels.iter().filter_map(Value::as_str).collect();
        if !levels.is_empty() {
            detail = format!("{detail} effort={}", levels.join("|"));
        }
    }
    if item.get("supportsFastMode").and_then(Value::as_bool) == Some(true) {
        detail.push_str(" fast=true|false");
    }
    Some((id.to_owned(), detail))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sdk_models_and_skips_default() {
        let body = r#"[{"value":"default","displayName":"Default"},
            {"value":"opus","displayName":"Opus 5.5","supportedEffortLevels":["low","max"],"supportsFastMode":true},
            {"value":"haiku","displayName":"Haiku"}]"#;
        let rows = parse_models(body);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0],
            (
                "opus".into(),
                "Claude Opus 5.5 effort=low|max fast=true|false".into()
            )
        );
        assert_eq!(rows[1], ("haiku".into(), "Claude Haiku".into()));
        assert!(parse_models("not json").is_empty());
    }

    #[test]
    fn cache_round_trip_and_fallback() {
        crate::test_support::test_utils::with_isolated_home(|_| {
            assert_eq!(list_claude_display_models(), builtin_models());
            write_cache(r#"[{"value":"fable","displayName":"Fable"}]"#).expect("write");
            assert_eq!(
                list_claude_display_models(),
                vec![("fable".to_string(), "Claude Fable".to_string())]
            );
            write_cache("[]").expect("write");
            assert_eq!(list_claude_display_models(), builtin_models());
        });
    }
}
