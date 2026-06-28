//! User prompt push for the inner bash-fence loop.

use malvin_mini::{ChatMessage, ChatRole};

use super::loop_types::{LoopDriverConfig, LoopDriverSession};

pub(crate) fn push_user_prompt(
    session: &mut LoopDriverSession,
    config: &LoopDriverConfig,
    user_prompt: &str,
) {
    let content = if !session.constraints_prepended && !config.mini_constraints.is_empty() {
        session.constraints_prepended = true;
        let model_line = if session.llm_model_slug.is_empty() {
            String::new()
        } else {
            format!(
                "\n\nYour OpenRouter model slug is `{}`. When asked which LLM you are, name this slug.",
                session.llm_model_slug
            )
        };
        format!("{}{}\n\n{}", config.mini_constraints, model_line, user_prompt)
    } else {
        user_prompt.to_string()
    };
    session.messages.push(ChatMessage {
        role: ChatRole::User,
        content,
    });
}
