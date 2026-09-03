use super::{
    DO_HEADER_MD, HEADER_MD, KPOP_COMMON_MD, KPOP_COMMON_NO_KPOP_MD, ROUTER_CODE_EXTRA_MD,
    ROUTER_SUMMARIZE_MD, WRITE_A_MD, WRITE_B_MD,
};

pub const ROUTER_A_MD: &str = "router_a.md";
pub const ROUTER_A_NO_KPOP_MD: &str = "router_a_no_kpop.md";
pub const ROUTER_B_MD: &str = "router_b.md";
pub const ROUTER_B_CREATIVE_MD: &str = "router_b_creative.md";
pub const ROUTER_B_NO_KPOP_MD: &str = "router_b_no_kpop.md";

/// Flags that select among `router_b` prompt variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouterBPromptFlags {
    pub creative: bool,
    pub no_kpop: bool,
}

/// Active router header template.
#[must_use]
pub const fn header_prompt_file() -> &'static str {
    HEADER_MD
}

/// Active `kpop_common` template (`no_kpop` selects the stripped variant).
#[must_use]
pub const fn kpop_common_prompt_file(no_kpop: bool) -> &'static str {
    if no_kpop {
        KPOP_COMMON_NO_KPOP_MD
    } else {
        KPOP_COMMON_MD
    }
}

/// Active `router_a` template (`no_kpop` selects the stripped variant).
#[must_use]
pub const fn router_a_prompt_file(no_kpop: bool) -> &'static str {
    if no_kpop {
        ROUTER_A_NO_KPOP_MD
    } else {
        ROUTER_A_MD
    }
}

/// Active `router_b` template (`no_kpop` wins over `creative`).
#[must_use]
pub const fn router_b_prompt_file(flags: RouterBPromptFlags) -> &'static str {
    if flags.no_kpop {
        ROUTER_B_NO_KPOP_MD
    } else if flags.creative {
        ROUTER_B_CREATIVE_MD
    } else {
        ROUTER_B_MD
    }
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

fn default_router_prompt(name: &str) -> Option<&'static str> {
    match name {
        ROUTER_A_MD => Some(include_str!("../../default_prompts/router_a.md")),
        ROUTER_A_NO_KPOP_MD => Some(include_str!("../../default_prompts/router_a_no_kpop.md")),
        ROUTER_B_MD => Some(include_str!("../../default_prompts/router_b.md")),
        ROUTER_B_CREATIVE_MD => Some(include_str!("../../default_prompts/router_b_creative.md")),
        ROUTER_B_NO_KPOP_MD => Some(include_str!("../../default_prompts/router_b_no_kpop.md")),
        ROUTER_CODE_EXTRA_MD => Some(include_str!("../../default_prompts/router_code_extra.md")),
        ROUTER_SUMMARIZE_MD => Some(include_str!("../../default_prompts/router_summarize.md")),
        _ => None,
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
            WRITE_A_MD => Some(include_str!("../../default_prompts/write_a.md")),
            WRITE_B_MD => Some(include_str!("../../default_prompts/write_b.md")),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::{
        HEADER_MD, ROUTER_A_MD, ROUTER_A_NO_KPOP_MD, ROUTER_B_CREATIVE_MD, ROUTER_B_MD,
        ROUTER_B_NO_KPOP_MD, RouterBPromptFlags, default_constraints_prompt, default_file,
        header_prompt_file, kpop_common_prompt_file, router_a_prompt_file, router_b_prompt_file,
    };
    use crate::prompts::{KPOP_COMMON_MD, KPOP_COMMON_NO_KPOP_MD};

    #[test]
    fn default_file_covers_router_a_and_b() {
        assert!(default_file("router_a.md").is_some());
        assert!(default_file("router_b.md").is_some());
        assert!(default_file("router_b_creative.md").is_some());
        assert!(default_file("router_summarize.md").is_some());
        assert!(default_file("write_a.md").is_some());
        assert!(default_file("write_b.md").is_some());
        assert!(default_file("mbc2.md").is_some());
        assert!(default_file(KPOP_COMMON_MD).is_some());
        assert!(default_file(KPOP_COMMON_NO_KPOP_MD).is_some());
        assert!(default_file(ROUTER_A_NO_KPOP_MD).is_some());
        assert!(default_file(ROUTER_B_NO_KPOP_MD).is_some());
        assert!(default_constraints_prompt("init_constraints.md").is_some());
        assert!(default_file("router_a.md").is_some());
    }

    #[test]
    fn active_prompt_selectors_use_canonical_templates() {
        assert_eq!(header_prompt_file(), HEADER_MD);
        assert_eq!(kpop_common_prompt_file(false), KPOP_COMMON_MD);
        assert_eq!(kpop_common_prompt_file(true), KPOP_COMMON_NO_KPOP_MD);
        assert_eq!(router_a_prompt_file(false), ROUTER_A_MD);
        assert_eq!(router_a_prompt_file(true), ROUTER_A_NO_KPOP_MD);
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
            ROUTER_B_CREATIVE_MD
        );
        assert_eq!(
            router_b_prompt_file(RouterBPromptFlags {
                creative: false,
                no_kpop: true,
            }),
            ROUTER_B_NO_KPOP_MD
        );
        assert_eq!(
            router_b_prompt_file(RouterBPromptFlags {
                creative: true,
                no_kpop: true,
            }),
            ROUTER_B_NO_KPOP_MD
        );
    }
}
