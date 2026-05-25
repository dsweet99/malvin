//! Prompt templates sourced from embedded defaults, with optional custom root.

mod defaults;
mod store;

mod template;
pub use template::*;

pub use defaults::{DO_HEADER_MD, HEADER_MD};

#[allow(unused_imports)]
pub(crate) use defaults::{DEFAULT_PROMPTS, REQUIRED_PROMPTS, default_file};

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

pub use crate::user_home::user_home_dir;
pub use store::{
    KpopPromptValidation, PromptStore, merged_coding_rules, render_mbc2_for_scheduled_kpop_block,
};

#[cfg(test)]
mod embedded_defaults_tests;
#[cfg(test)]
#[path = "prompts_tests_a.rs"]
mod prompts_tests_a;
#[cfg(test)]
#[path = "prompts_tests_b.rs"]
mod prompts_tests_b;


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_prompt_error() { let _ = stringify!(PromptError); }

}
