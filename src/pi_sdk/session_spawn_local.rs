use pi::sdk::SessionOptions;

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
                "You are a non-interactive CLI agent. For any shell/file action emit ONLY ",
                "a JSON tool call {\"name\":\"bash\",\"parameters\":{\"command\":\"...\"}} ",
                "(or read/write/edit/grep/find/ls). Never invent results. Never answer with ",
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

pub(crate) fn local_max_tool_iterations(mode: LocalAgentMode) -> usize {
    match mode {
        LocalAgentMode::KeylessTools => 40,
        LocalAgentMode::KeylessTextOnly => 1,
        LocalAgentMode::NonKeyless => SessionOptions::default().max_tool_iterations,
    }
}

fn keyless_local_tools_enabled(provider: &str, model: &str) -> bool {
    if !provider.eq_ignore_ascii_case("ollama") {
        return true;
    }
    super::local_llm_ollama::ollama_model_supports_tools(model).unwrap_or(true)
}

pub(crate) fn local_session_overrides(
    keyless: bool,
    provider: &str,
    model: &str,
) -> (Option<String>, Option<Vec<String>>, usize) {
    let mode = LocalAgentMode::for_provider_model(keyless, provider, model);
    (
        local_append_system_prompt(mode),
        local_enabled_tools(mode),
        local_max_tool_iterations(mode),
    )
}
