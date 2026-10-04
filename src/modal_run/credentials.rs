use std::collections::BTreeMap;
use std::path::Path;

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

pub(super) fn env_set(key: &str) -> bool {
    std::env::var(key).is_ok_and(|v| !v.trim().is_empty())
}

#[must_use]
pub fn modal_credentials_present(home: &Path) -> bool {
    (env_set("MODAL_TOKEN_ID") && env_set("MODAL_TOKEN_SECRET")) || home.join(".modal.toml").is_file()
}
