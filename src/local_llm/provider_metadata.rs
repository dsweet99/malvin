use std::path::PathBuf;

pub(crate) struct LocalProvider {
    pub(crate) id: &'static str,
    pub(crate) aliases: &'static [&'static str],
    pub(crate) base_url: &'static str,
    pub(crate) api: &'static str,
}

const LOCAL_PROVIDERS: &[LocalProvider] = &[
    LocalProvider {
        id: "ollama",
        aliases: &[],
        base_url: "http://127.0.0.1:11434/v1",
        api: "openai-completions",
    },
    LocalProvider {
        id: "llamacpp",
        aliases: &["llama-cpp", "llama.cpp", "llama-server"],
        base_url: "http://127.0.0.1:8080/v1",
        api: "openai-completions",
    },
    LocalProvider {
        id: "mistralrs",
        aliases: &["mistral.rs", "mistral-rs"],
        base_url: "http://127.0.0.1:1234/v1",
        api: "openai-completions",
    },
];

const PROVIDER_ENV_KEYS: &[(&str, &[&str])] = &[
    (
        "anthropic",
        &["ANTHROPIC_API_KEY", "ANTHROPIC_OAUTH_TOKEN", "ANTHROPIC_AUTH_TOKEN"],
    ),
    ("openai", &["OPENAI_API_KEY"]),
    ("deepseek", &["DEEPSEEK_API_KEY"]),
    ("google", &["GEMINI_API_KEY"]),
    ("mistral", &["MISTRAL_API_KEY"]),
    ("groq", &["GROQ_API_KEY"]),
    ("cerebras", &["CEREBRAS_API_KEY"]),
    ("xai", &["XAI_API_KEY"]),
    ("openrouter", &["OPENROUTER_API_KEY"]),
    ("zai", &["ZAI_API_KEY"]),
    ("fireworks", &["FIREWORKS_API_KEY"]),
    ("together", &["TOGETHER_API_KEY"]),
    ("moonshotai", &["MOONSHOT_API_KEY"]),
    ("minimax", &["MINIMAX_API_KEY"]),
    ("huggingface", &["HF_TOKEN"]),
];

fn local_provider(provider: &str) -> Option<&'static LocalProvider> {
    let provider = provider.trim();
    LOCAL_PROVIDERS.iter().find(|p| {
        p.id.eq_ignore_ascii_case(provider)
            || p.aliases.iter().any(|a| a.eq_ignore_ascii_case(provider))
    })
}

#[must_use]
pub fn provider_is_keyless_local(provider: &str) -> bool {
    local_provider(provider).is_some()
}

#[must_use]
pub(crate) fn local_provider_defaults(provider: &str) -> Option<&'static LocalProvider> {
    local_provider(provider)
}

#[must_use]
pub(crate) fn provider_auth_env_keys(provider: &str) -> Option<&'static [&'static str]> {
    let provider = provider.trim();
    PROVIDER_ENV_KEYS
        .iter()
        .find(|(id, _)| id.eq_ignore_ascii_case(provider))
        .map(|(_, keys)| *keys)
}

#[must_use]
pub(crate) fn provider_ids_match(left: &str, right: &str) -> bool {
    let (left, right) = (left.trim(), right.trim());
    if left.eq_ignore_ascii_case(right) {
        return true;
    }
    match (local_provider(left), local_provider(right)) {
        (Some(a), Some(b)) => a.id == b.id,
        _ => false,
    }
}

#[must_use]
pub(crate) fn pi_agent_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("PI_CODING_AGENT_DIR").filter(|v| !v.is_empty()) {
        return PathBuf::from(dir);
    }
    std::env::var_os("HOME")
        .map_or_else(|| PathBuf::from("."), PathBuf::from)
        .join(".pi")
        .join("agent")
}

#[must_use]
pub(crate) fn pi_models_json_path() -> PathBuf {
    pi_agent_dir().join("models.json")
}

#[must_use]
pub(crate) fn pi_auth_json_path() -> PathBuf {
    pi_agent_dir().join("auth.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyless_local_matches_aliases_only_for_local_servers() {
        for id in ["ollama", "llamacpp", "llama.cpp", "MistralRS", "mistral-rs"] {
            assert!(provider_is_keyless_local(id), "{id}");
        }
        for id in ["openai", "ollama-cloud", "lmstudio", "nope"] {
            assert!(!provider_is_keyless_local(id), "{id}");
        }
        assert!(provider_ids_match("llama-server", "llamacpp"));
        assert!(!provider_ids_match("ollama", "llamacpp"));
    }

    #[test]
    fn env_keys_cover_common_providers() {
        assert_eq!(provider_auth_env_keys("openai"), Some(&["OPENAI_API_KEY"][..]));
        assert!(provider_auth_env_keys("ollama").is_none());
    }

    #[test]
    fn agent_dir_honors_env_override() {
        let _lock = crate::test_support::test_utils::test_env_lock();
        crate::agent_process::with_env("PI_CODING_AGENT_DIR", Some("/tmp/pi-agent-x"), || {
            assert_eq!(pi_models_json_path(), PathBuf::from("/tmp/pi-agent-x/models.json"));
            assert_eq!(pi_auth_json_path(), PathBuf::from("/tmp/pi-agent-x/auth.json"));
        });
    }
}
