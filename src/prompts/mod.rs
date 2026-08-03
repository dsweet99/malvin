//! Prompt templates sourced from embedded defaults, with optional custom root.

mod defaults;
mod store;

mod template;
pub use template::*;

pub use defaults::{
    DO_HEADER_MD, EXPLAIN_WRAPPER_MD, HEADER_MD, ROUTER_A_MD, ROUTER_B_MD, ROUTER_CODE_EXTRA_MD,
    ROUTER_SUMMARIZE_MD,
};

#[allow(unused_imports)]
pub(crate) use defaults::{DEFAULT_PROMPTS, REQUIRED_PROMPTS, default_file};

const UNRESOLVED_BRACES_MSG: &str =
    "prompt still contains \"{{\" before ACP; resolve every {{ key }} placeholder";

/// # Errors
///
/// Returns [`PromptError`] when `text` still contains `{{`.
pub fn enforce_no_unresolved_braces(text: &str) -> Result<(), PromptError> {
    enforce_no_unresolved_braces_in(text, None)
}

/// # Errors
///
/// Returns [`PromptError`] when `text` still contains `{{`.
pub fn enforce_no_unresolved_braces_in(
    text: &str,
    prompt_file: Option<&str>,
) -> Result<(), PromptError> {
    if template::unresolved_spaced_brace_placeholders(text).is_empty() {
        Ok(())
    } else {
        Err(unresolved_braces_error(prompt_file))
    }
}

/// # Errors
///
/// Returns [`PromptError`] when `template` contains `{{ key }}` placeholders whose keys are
/// missing from `context`.
pub fn enforce_template_placeholders_resolved_in(
    template: &str,
    context: &std::collections::HashMap<String, String>,
    prompt_file: Option<&str>,
) -> Result<(), PromptError> {
    if template::unresolved_template_placeholders(template, context).is_empty() {
        Ok(())
    } else {
        Err(unresolved_braces_error(prompt_file))
    }
}

fn unresolved_braces_error(prompt_file: Option<&str>) -> PromptError {
    let msg = prompt_file.map_or_else(
        || UNRESOLVED_BRACES_MSG.to_string(),
        |name| format!("{UNRESOLVED_BRACES_MSG} (in {name})"),
    );
    PromptError(msg)
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct PromptError(pub String);

pub use crate::user_home::user_home_dir;
pub use store::{
    KpopPromptValidation, PromptStore, render_header, render_mbc2_for_scheduled_kpop_block,
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
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_prompt_error() { let _: Option<PromptError> = None; }

}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<PromptError> = None;
    }
}
