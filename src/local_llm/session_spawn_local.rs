const LOCAL_ENABLED_TOOLS: &[&str] = &["read", "bash", "edit", "write", "grep", "find", "ls"];

const LOCAL_TEXT_ONLY_APPEND: &str = concat!(
    "You are a non-interactive CLI agent. Answer in plain text only. ",
    "You have no tools; do not invent shell output, file contents, or tool results."
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LocalAgentMode {
    NonKeyless,
    KeylessTools,
    KeylessTextOnly,
}

impl LocalAgentMode {
    pub(crate) fn for_provider_model(keyless: bool, provider: &str, model: &str) -> Self {
        if !keyless {
            return Self::NonKeyless;
        }
        if keyless_local_tools_enabled(provider, model) {
            Self::KeylessTools
        } else {
            Self::KeylessTextOnly
        }
    }
}

pub(crate) fn local_append_system_prompt(mode: LocalAgentMode) -> Option<String> {
    match mode {
        LocalAgentMode::NonKeyless => None,
        LocalAgentMode::KeylessTools => Some(
            concat!(
                "You are a non-interactive CLI agent. For any shell/file action, reply with ",
                "ONLY a tool call to one of the provided tools (bash, read, write, edit, grep, ",
                "find, ls), in the exact tool-call format your instructions specify, with no ",
                "text before or after it. Never invent results. Never answer with ",
                "markdown ```bash fences. Stay in the workspace unless a temp path is named. ",
                "For HTTP(S) downloads prefer curl -L -o FILE URL. ",
                "To count files prefer bash with find . -type f | wc -l; do not install packages."
            )
            .to_string(),
        ),
        LocalAgentMode::KeylessTextOnly => Some(LOCAL_TEXT_ONLY_APPEND.to_string()),
    }
}

pub(crate) fn local_enabled_tools(mode: LocalAgentMode) -> Option<Vec<String>> {
    match mode {
        LocalAgentMode::NonKeyless => None,
        LocalAgentMode::KeylessTools => Some(
            LOCAL_ENABLED_TOOLS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        ),
        LocalAgentMode::KeylessTextOnly => Some(Vec::new()),
    }
}

fn keyless_local_tools_enabled(provider: &str, model: &str) -> bool {
    if !provider.eq_ignore_ascii_case("ollama") {
        return true;
    }
    super::local_llm_ollama::ollama_model_supports_tools(model).unwrap_or(false)
}

#[must_use]
pub(crate) fn local_cli_args(provider: &str, model: &str) -> Vec<String> {
    let keyless = super::provider_metadata::provider_is_keyless_local(provider);
    local_cli_args_for_mode(LocalAgentMode::for_provider_model(keyless, provider, model))
}

pub(crate) fn local_cli_args_for_mode(mode: LocalAgentMode) -> Vec<String> {
    let mut args = Vec::new();
    match local_enabled_tools(mode) {
        Some(tools) if tools.is_empty() => args.push("--no-tools".to_string()),
        Some(tools) => {
            args.push("--tools".to_string());
            args.push(tools.join(","));
        }
        None => {}
    }
    if let Some(text) = local_append_system_prompt(mode) {
        args.push("--append-system-prompt".to_string());
        args.push(text);
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cloud_providers_get_no_extra_flags_and_keyless_tool_mode_allows_core_tools() {
        {
            assert!(local_cli_args("openai", "gpt-4o").is_empty());
        }
        {
            let args = local_cli_args_for_mode(LocalAgentMode::KeylessTools);
            assert_eq!(args[0], "--tools");
            assert_eq!(args[1], "read,bash,edit,write,grep,find,ls");
            assert_eq!(args[2], "--append-system-prompt");
            assert!(args[3].contains("tool call"));
        }
    }
    #[test]
    fn keyless_text_only_mode_disables_tools_and_non_ollama_local_providers_use_tools() {
        {
            let args = local_cli_args_for_mode(LocalAgentMode::KeylessTextOnly);
            assert_eq!(args[0], "--no-tools");
            assert_eq!(args[2], LOCAL_TEXT_ONLY_APPEND);
        }
        {
            assert_eq!(
                LocalAgentMode::for_provider_model(true, "llamacpp", "m"),
                LocalAgentMode::KeylessTools
            );
            assert_eq!(
                LocalAgentMode::for_provider_model(false, "openai", "m"),
                LocalAgentMode::NonKeyless
            );
        }
    }
}
