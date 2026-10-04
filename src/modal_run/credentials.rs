use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::model_id::{CODEX_PREFIX, CURSOR_PREFIX, PI_PREFIX};

pub const FORWARDED_ENV: &[&str] = &[
    "CURSOR_API_KEY",
    "CURSOR_AGENT_API_KEY",
    "AGENT_API_KEY",
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
    "OPENROUTER_API_KEY",
];

#[must_use]
pub fn forwarded_env(lookup: impl Fn(&str) -> Option<String>) -> BTreeMap<String, String> {
    FORWARDED_ENV
        .iter()
        .filter_map(|k| {
            let v = lookup(k)?;
            (!v.trim().is_empty()).then(|| ((*k).to_string(), v))
        })
        .collect()
}

fn env_set(key: &str) -> bool {
    std::env::var(key).is_ok_and(|v| !v.trim().is_empty())
}

#[must_use]
pub fn modal_credentials_present(home: &Path) -> bool {
    (env_set("MODAL_TOKEN_ID") && env_set("MODAL_TOKEN_SECRET")) || home.join(".modal.toml").is_file()
}

#[must_use]
pub fn file_logins(model: &str, home: &Path) -> Vec<PathBuf> {
    let candidates: Vec<PathBuf> = if model.starts_with(CODEX_PREFIX) {
        vec![home.join(".codex/auth.json")]
    } else if model.starts_with(PI_PREFIX) {
        let dir = std::env::var_os("PI_CODING_AGENT_DIR")
            .filter(|v| !v.is_empty())
            .map_or_else(|| home.join(".pi/agent"), PathBuf::from);
        vec![dir.join("auth.json"), dir.join("models.json")]
    } else {
        Vec::new()
    };
    candidates.into_iter().filter(|p| p.is_file()).collect()
}

pub fn preflight(model: &str, home: &Path) -> Result<(), String> {
    if !modal_credentials_present(home) {
        return Err(
            "malvin --modal needs Modal credentials. Run `modal setup`, or set MODAL_TOKEN_ID and MODAL_TOKEN_SECRET."
                .to_string(),
        );
    }
    if model.starts_with(CURSOR_PREFIX) && crate::cursor_sdk::effective_sdk_api_key().is_none() {
        return Err(
            "malvin --modal with a cursor: model needs CURSOR_API_KEY (or CURSOR_AGENT_API_KEY / AGENT_API_KEY); `agent login` does not reach the Sandbox."
                .to_string(),
        );
    }
    if model.starts_with(CODEX_PREFIX) && !env_set("OPENAI_API_KEY") && file_logins(model, home).is_empty() {
        return Err(
            "malvin --modal with a codex: model needs OPENAI_API_KEY or ~/.codex/auth.json."
                .to_string(),
        );
    }
    Ok(())
}
