use super::{
    DO_HEADER_MD, HEADER_MD, KPOP_COMMON_MD, KPOP_COMMON_NO_KPOP_MD, ROUTER_CODE_EXTRA_MD,
    ROUTER_SUMMARIZE_MD,
};

pub const ROUTER_A_MD: &str = "router_a.md";
pub const ROUTER_A_AUDIT_MD: &str = "router_a_audit.md";
pub const ROUTER_A_AUDIT_NO_KPOP_MD: &str = "router_a_audit_no_kpop.md";
pub const ROUTER_B_MD: &str = "router_b.md";
pub const ROUTER_B_SATISFY_MD: &str = "router_b_satisfy.md";
pub const ROUTER_B_SATISFY_BRIEF_MD: &str = "router_b_satisfy_brief.md";
pub const ROUTER_B_SATISFY_NO_KPOP_MD: &str = "router_b_satisfy_no_kpop.md";
pub const ROUTER_B_CREATIVE_LEAD_MD: &str = "router_b_creative_lead.md";
pub const ROUTER_B_DONE_NOTE_MD: &str = "router_b_done_note.md";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouterBPromptFlags {
    pub creative: bool,
    pub no_kpop: bool,
}

#[must_use]
pub const fn header_prompt_file() -> &'static str {
    HEADER_MD
}

#[must_use]
pub const fn kpop_common_prompt_file(no_kpop: bool) -> &'static str {
    if no_kpop {
        KPOP_COMMON_NO_KPOP_MD
    } else {
        KPOP_COMMON_MD
    }
}

#[must_use]
pub const fn router_a_prompt_file(_no_kpop: bool) -> &'static str {
    ROUTER_A_MD
}

#[must_use]
pub const fn router_a_audit_prompt_file(no_kpop: bool) -> &'static str {
    if no_kpop {
        ROUTER_A_AUDIT_NO_KPOP_MD
    } else {
        ROUTER_A_AUDIT_MD
    }
}

#[must_use]
pub const fn router_b_prompt_file(_flags: RouterBPromptFlags) -> &'static str {
    ROUTER_B_MD
}

#[must_use]
pub const fn router_b_satisfy_prompt_file(flags: RouterBPromptFlags) -> &'static str {
    if flags.no_kpop {
        ROUTER_B_SATISFY_NO_KPOP_MD
    } else if flags.creative {
        ROUTER_B_SATISFY_BRIEF_MD
    } else {
        ROUTER_B_SATISFY_MD
    }
}

#[must_use]
pub const fn router_b_uses_creative_lead(flags: RouterBPromptFlags) -> bool {
    flags.creative && !flags.no_kpop
}

#[must_use]
pub const fn router_b_uses_done_note(flags: RouterBPromptFlags) -> bool {
    !flags.no_kpop && !flags.creative
}

fn default_constraints_prompt(name: &str) -> Option<&'static str> {
    match name {
        "init_constraints.md" => Some(include_str!("../../default_prompts/init_constraints.md")),
        _ => None,
    }
}

fn default_mbc2_prompt(name: &str) -> Option<&'static str> {
    match name {
        "mbc2.md" => Some(include_str!("../../default_prompts/mbc2.md")),
        _ => None,
    }
}

fn default_router_a_prompt(name: &str) -> Option<&'static str> {
    match name {
        ROUTER_A_MD => Some(include_str!("../../default_prompts/router_a.md")),
        ROUTER_A_AUDIT_MD => Some(include_str!("../../default_prompts/router_a_audit.md")),
        ROUTER_A_AUDIT_NO_KPOP_MD => {
            Some(include_str!("../../default_prompts/router_a_audit_no_kpop.md"))
        }
        _ => None,
    }
}

fn default_router_b_fragment(name: &str) -> Option<&'static str> {
    match name {
        ROUTER_B_SATISFY_MD => Some(include_str!("../../default_prompts/router_b_satisfy.md")),
        ROUTER_B_SATISFY_BRIEF_MD => {
            Some(include_str!("../../default_prompts/router_b_satisfy_brief.md"))
        }
        ROUTER_B_SATISFY_NO_KPOP_MD => {
            Some(include_str!("../../default_prompts/router_b_satisfy_no_kpop.md"))
        }
        ROUTER_B_CREATIVE_LEAD_MD => {
            Some(include_str!("../../default_prompts/router_b_creative_lead.md"))
        }
        ROUTER_B_DONE_NOTE_MD => Some(include_str!("../../default_prompts/router_b_done_note.md")),
        _ => None,
    }
}

fn default_router_prompt(name: &str) -> Option<&'static str> {
    match name {
        ROUTER_B_MD => Some(include_str!("../../default_prompts/router_b.md")),
        ROUTER_CODE_EXTRA_MD => Some(include_str!("../../default_prompts/router_code_extra.md")),
        ROUTER_SUMMARIZE_MD => Some(include_str!("../../default_prompts/router_summarize.md")),
        _ => default_router_a_prompt(name).or_else(|| default_router_b_fragment(name)),
    }
}

pub fn default_file(name: &str) -> Option<&'static str> {
    default_constraints_prompt(name)
        .or_else(|| default_mbc2_prompt(name))
        .or_else(|| default_router_prompt(name))
        .or_else(|| match name {
            HEADER_MD => Some(include_str!("../../default_prompts/header.md")),
            KPOP_COMMON_MD => Some(include_str!("../../default_prompts/kpop_common.md")),
            KPOP_COMMON_NO_KPOP_MD => {
                Some(include_str!("../../default_prompts/kpop_common_no_kpop.md"))
            }
            DO_HEADER_MD => Some(include_str!("../../default_prompts/do_header.md")),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::{
        HEADER_MD, ROUTER_A_AUDIT_MD, ROUTER_A_AUDIT_NO_KPOP_MD, ROUTER_A_MD,
        ROUTER_B_CREATIVE_LEAD_MD, ROUTER_B_DONE_NOTE_MD, ROUTER_B_MD, ROUTER_B_SATISFY_BRIEF_MD,
        ROUTER_B_SATISFY_MD, ROUTER_B_SATISFY_NO_KPOP_MD, RouterBPromptFlags,
        default_constraints_prompt, default_file, header_prompt_file, kpop_common_prompt_file,
        router_a_audit_prompt_file, router_a_prompt_file, router_b_prompt_file,
        router_b_satisfy_prompt_file, router_b_uses_creative_lead, router_b_uses_done_note,
    };
    use crate::prompts::{KPOP_COMMON_MD, KPOP_COMMON_NO_KPOP_MD};

    #[test]
    fn default_file_covers_router_a_and_b() {
        assert!(default_file("router_a.md").is_some());
        assert!(default_file("router_b.md").is_some());
        assert!(default_file(ROUTER_A_AUDIT_MD).is_some());
        assert!(default_file(ROUTER_B_SATISFY_BRIEF_MD).is_some());
        assert!(default_file(ROUTER_B_CREATIVE_LEAD_MD).is_some());
        assert!(default_file("router_summarize.md").is_some());
        assert!(default_file("mbc2.md").is_some());
        assert!(default_file(KPOP_COMMON_MD).is_some());
        assert!(default_file(KPOP_COMMON_NO_KPOP_MD).is_some());
        assert!(default_file(ROUTER_A_AUDIT_NO_KPOP_MD).is_some());
        assert!(default_file(ROUTER_B_SATISFY_NO_KPOP_MD).is_some());
        assert!(default_file(ROUTER_B_DONE_NOTE_MD).is_some());
        assert!(default_constraints_prompt("init_constraints.md").is_some());
        assert!(default_file("router_a.md").is_some());
    }

    #[test]
    fn active_prompt_selectors_use_canonical_templates() {
        assert_eq!(header_prompt_file(), HEADER_MD);
        assert_eq!(kpop_common_prompt_file(false), KPOP_COMMON_MD);
        assert_eq!(kpop_common_prompt_file(true), KPOP_COMMON_NO_KPOP_MD);
        assert_eq!(router_a_prompt_file(false), ROUTER_A_MD);
        assert_eq!(router_a_prompt_file(true), ROUTER_A_MD);
        assert_eq!(router_a_audit_prompt_file(false), ROUTER_A_AUDIT_MD);
        assert_eq!(router_a_audit_prompt_file(true), ROUTER_A_AUDIT_NO_KPOP_MD);
        assert_eq!(
            router_b_prompt_file(RouterBPromptFlags {
                creative: false,
                no_kpop: false,
            }),
            ROUTER_B_MD
        );
        assert_eq!(
            router_b_prompt_file(RouterBPromptFlags {
                creative: true,
                no_kpop: false,
            }),
            ROUTER_B_MD
        );
        assert_eq!(
            router_b_satisfy_prompt_file(RouterBPromptFlags {
                creative: false,
                no_kpop: false,
            }),
            ROUTER_B_SATISFY_MD
        );
        assert_eq!(
            router_b_satisfy_prompt_file(RouterBPromptFlags {
                creative: true,
                no_kpop: false,
            }),
            ROUTER_B_SATISFY_BRIEF_MD
        );
        assert_eq!(
            router_b_satisfy_prompt_file(RouterBPromptFlags {
                creative: false,
                no_kpop: true,
            }),
            ROUTER_B_SATISFY_NO_KPOP_MD
        );
        assert_eq!(
            router_b_satisfy_prompt_file(RouterBPromptFlags {
                creative: true,
                no_kpop: true,
            }),
            ROUTER_B_SATISFY_NO_KPOP_MD
        );
        assert!(router_b_uses_creative_lead(RouterBPromptFlags {
            creative: true,
            no_kpop: false,
        }));
        assert!(!router_b_uses_creative_lead(RouterBPromptFlags {
            creative: true,
            no_kpop: true,
        }));
        assert!(router_b_uses_done_note(RouterBPromptFlags {
            creative: false,
            no_kpop: false,
        }));
        assert!(!router_b_uses_done_note(RouterBPromptFlags {
            creative: true,
            no_kpop: false,
        }));
    }
}
