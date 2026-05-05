//! Prompt templates sourced from embedded defaults, with optional custom root.

mod defaults;
mod store;
mod template;

pub use defaults::{DO_HEADER_MD, HEADER_MD};

#[allow(unused_imports)]
pub(crate) use defaults::{default_file, DEFAULT_PROMPTS, REQUIRED_PROMPTS};

/// # Errors
///
/// Returns [`PromptError`] when `text` still contains `{{`.
pub fn enforce_no_unresolved_braces(text: &str) -> Result<(), PromptError> {
    if text.contains("{{") {
        Err(PromptError(
            "prompt still contains \"{{\" before ACP; resolve every {{ key }} placeholder"
                .to_string(),
        ))
    } else {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct PromptError(pub String);

pub use store::{KpopPromptValidation, PromptStore, user_home_dir};

pub use template::{
    merge_header_and_coding_rules, merged_coding_rules, render_mbc2_for_scheduled_kpop_block,
    render_template, substitute_template,
};

#[cfg(test)]
mod embedded_defaults_tests;
#[cfg(test)]
#[allow(unsafe_code)]
mod tests;
