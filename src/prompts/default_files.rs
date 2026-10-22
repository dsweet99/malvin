use super::{HEADER_MD, ROUTER_CODE_EXTRA_MD, ROUTER_SUMMARIZE_MD};
use std::sync::OnceLock;

pub const ROUTER_A_MD: &str = "router_a.md";
pub const ROUTER_A_2_MD: &str = "router_a_2.md";
pub const ROUTER_A_AUDIT_MD: &str = "router_a_audit.md";
pub const ROUTER_B_MD: &str = "router_b.md";
pub const ROUTER_B_SATISFY_MD: &str = "router_b_satisfy.md";
pub const ROUTER_B_DONE_NOTE_MD: &str = "router_b_done_note.md";
pub const ROUTER_C_MD: &str = "router_c.md";

#[must_use]
pub const fn header_prompt_file() -> &'static str {
    HEADER_MD
}

#[must_use]
pub const fn router_a_prompt_file() -> &'static str {
    ROUTER_A_MD
}

#[must_use]
pub const fn router_a_2_prompt_file() -> &'static str {
    ROUTER_A_2_MD
}

#[must_use]
pub const fn router_a_audit_prompt_file() -> &'static str {
    ROUTER_A_AUDIT_MD
}

#[must_use]
pub const fn router_b_prompt_file() -> &'static str {
    ROUTER_B_MD
}

#[must_use]
pub const fn router_b_satisfy_prompt_file() -> &'static str {
    ROUTER_B_SATISFY_MD
}

fn default_constraints_prompt(name: &str) -> Option<&'static str> {
    match name {
        "init_constraints.md" => Some(include_str!("../../default_prompts/init_constraints.md")),
        _ => None,
    }
}

fn default_router_a_prompt(name: &str) -> Option<&'static str> {
    match name {
        ROUTER_A_MD => Some(include_str!("../../default_prompts/router_a.md")),
        ROUTER_A_2_MD => Some(include_str!("../../default_prompts/router_a_2.md")),
        ROUTER_A_AUDIT_MD => Some(include_str!("../../default_prompts/router_a_audit.md")),
        _ => None,
    }
}

fn default_router_b_fragment(name: &str) -> Option<&'static str> {
    match name {
        ROUTER_B_SATISFY_MD => Some(include_str!("../../default_prompts/router_b_satisfy.md")),
        ROUTER_B_DONE_NOTE_MD => Some(include_str!("../../default_prompts/router_b_done_note.md")),
        _ => None,
    }
}

fn default_router_prompt(name: &str) -> Option<&'static str> {
    match name {
        ROUTER_B_MD => Some(include_str!("../../default_prompts/router_b.md")),
        ROUTER_C_MD => Some(include_str!("../../default_prompts/router_c.md")),
        ROUTER_CODE_EXTRA_MD => Some(include_str!("../../default_prompts/router_code_extra.md")),
        ROUTER_SUMMARIZE_MD => Some(include_str!("../../default_prompts/router_summarize.md")),
        _ => default_router_a_prompt(name).or_else(|| default_router_b_fragment(name)),
    }
}

fn default_header_prompt() -> &'static str {
    static HEADER: OnceLock<String> = OnceLock::new();
    HEADER
        .get_or_init(|| {
            include_str!("../../default_prompts/header.md")
                .replace("- Coding / SWE", "- Technical problem-solving")
        })
        .as_str()
}

pub fn default_file(name: &str) -> Option<&'static str> {
    default_constraints_prompt(name)
        .or_else(|| default_router_prompt(name))
        .or_else(|| match name {
            HEADER_MD => Some(default_header_prompt()),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::{
        HEADER_MD, ROUTER_A_2_MD, ROUTER_A_AUDIT_MD, ROUTER_A_MD, ROUTER_B_DONE_NOTE_MD,
        ROUTER_B_MD, ROUTER_B_SATISFY_MD, ROUTER_C_MD, default_constraints_prompt, default_file,
        header_prompt_file, router_a_2_prompt_file, router_a_audit_prompt_file,
        router_a_prompt_file, router_b_prompt_file, router_b_satisfy_prompt_file,
    };

    #[test]
    fn default_file_covers_router_a_and_b() {
        assert!(default_file("router_a.md").is_some());
        assert!(default_file(ROUTER_A_2_MD).is_some());
        assert!(default_file("router_b.md").is_some());
        assert!(default_file(ROUTER_C_MD).is_some());
        assert!(default_file(ROUTER_A_AUDIT_MD).is_some());
        assert!(default_file(ROUTER_B_SATISFY_MD).is_some());
        assert!(default_file("router_summarize.md").is_some());
        assert!(default_file(ROUTER_B_DONE_NOTE_MD).is_some());
        assert!(default_constraints_prompt("init_constraints.md").is_some());
        assert!(default_file("router_a.md").is_some());
        shipped_prompts_spell_available_and_techniques();
    }

    fn shipped_prompts_spell_available_and_techniques() {
        let header = default_file(HEADER_MD).expect("header");
        assert!(
            !header.contains("availble"),
            "header.md misspells available"
        );
    }

    #[test]
    fn active_prompt_selectors_use_canonical_templates() {
        assert_eq!(header_prompt_file(), HEADER_MD);
        assert_eq!(router_a_prompt_file(), ROUTER_A_MD);
        assert_eq!(router_a_2_prompt_file(), ROUTER_A_2_MD);
        assert_eq!(router_a_audit_prompt_file(), ROUTER_A_AUDIT_MD);
        assert_eq!(router_b_prompt_file(), ROUTER_B_MD);
        assert_eq!(router_b_satisfy_prompt_file(), ROUTER_B_SATISFY_MD);
    }
}
