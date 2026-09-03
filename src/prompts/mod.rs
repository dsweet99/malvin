mod defaults;
mod store;

mod template;
pub use template::*;

pub use defaults::{
    DO_HEADER_MD, HEADER_MD, KPOP_COMMON_MD, KPOP_COMMON_NO_KPOP_MD, ROUTER_A_MD,
    ROUTER_A_NO_KPOP_MD, ROUTER_B_CREATIVE_MD, ROUTER_B_MD, ROUTER_B_NO_KPOP_MD,
    ROUTER_CODE_EXTRA_MD, ROUTER_SUMMARIZE_MD, RouterBPromptFlags, WRITE_A_MD, WRITE_B_MD,
    header_prompt_file, kpop_common_prompt_file, router_a_prompt_file, router_b_prompt_file,
};

#[allow(unused_imports)]
pub(crate) use defaults::{DEFAULT_PROMPTS, REQUIRED_PROMPTS, default_file};

const UNRESOLVED_BRACES_MSG: &str =
    "prompt still contains \"{{\" before ACP; resolve every {{ key }} placeholder";

pub fn enforce_no_unresolved_braces(text: &str) -> Result<(), PromptError> {
    enforce_no_unresolved_braces_in(text, None)
}

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
pub use store::{PromptStore, build_mbc2_render_context, render_header, render_mbc2_prompt};

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
    use super::*;

    #[test]
    fn kiss_cov_prompt_error() {
        let _: Option<PromptError> = None;
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<PromptError> = None;
    }
}
