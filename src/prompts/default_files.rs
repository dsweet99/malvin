use super::{
    DO_HEADER_MD, INSPIRE_SUMMARIZE_MD, WRITE_A_MD, WRITE_B_MD, HEADER_MD, ROUTER_A_MD,
    ROUTER_B_CREATIVE_MD, ROUTER_B_MD, ROUTER_CODE_EXTRA_MD, ROUTER_SUMMARIZE_MD,
};

pub const KPOP_COMMON_MD: &str = "kpop_common.md";
pub const KPOP_COMMON_FAKE_MD: &str = "kpop_common_fake.md";
pub const HEADER_NOKPOP_MD: &str = "header_nokpop.md";
pub const ROUTER_A_NOKPOP_MD: &str = "router_a_nokpop.md";
pub const ROUTER_B_NOKPOP_MD: &str = "router_b_nokpop.md";
pub const ROUTER_B_CREATIVE_NOKPOP_MD: &str = "router_b_creative_nokpop.md";

/// Active router header template (`KPop` ablated; full `KPop` templates are not selectable).
#[must_use]
pub const fn header_prompt_file() -> &'static str {
    HEADER_NOKPOP_MD
}

/// Active `KPop`-common slot template (stub; full method is not selectable).
#[must_use]
pub const fn kpop_common_prompt_file() -> &'static str {
    KPOP_COMMON_FAKE_MD
}

/// Active `router_a` template (`KPop` ablated).
#[must_use]
pub const fn router_a_prompt_file() -> &'static str {
    ROUTER_A_NOKPOP_MD
}

/// Active `router_b` template (`KPop` ablated; `creative` selects the creative variant).
#[must_use]
pub const fn router_b_prompt_file(creative: bool) -> &'static str {
    if creative {
        ROUTER_B_CREATIVE_NOKPOP_MD
    } else {
        ROUTER_B_NOKPOP_MD
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
        INSPIRE_SUMMARIZE_MD => {
            Some(include_str!("../../default_prompts/inspire_summarize.md"))
        }
        _ => None,
    }
}

fn default_kpop_prompt(name: &str) -> Option<&'static str> {
    default_mbc2_prompt(name).or_else(|| match name {
        "kpop.md" | KPOP_COMMON_MD => Some(include_str!("../../default_prompts/kpop_common.md")),
        KPOP_COMMON_FAKE_MD => Some(include_str!("../../default_prompts/kpop_common_fake.md")),
        _ => None,
    })
}

fn default_router_prompt(name: &str) -> Option<&'static str> {
    match name {
        ROUTER_A_MD => Some(include_str!("../../default_prompts/router_a.md")),
        ROUTER_A_NOKPOP_MD => Some(include_str!("../../default_prompts/router_a_nokpop.md")),
        ROUTER_B_MD => Some(include_str!("../../default_prompts/router_b.md")),
        ROUTER_B_NOKPOP_MD => Some(include_str!("../../default_prompts/router_b_nokpop.md")),
        ROUTER_B_CREATIVE_MD => Some(include_str!("../../default_prompts/router_b_creative.md")),
        ROUTER_B_CREATIVE_NOKPOP_MD => {
            Some(include_str!("../../default_prompts/router_b_creative_nokpop.md"))
        }
        ROUTER_CODE_EXTRA_MD => {
            Some(include_str!("../../default_prompts/router_code_extra.md"))
        }
        ROUTER_SUMMARIZE_MD => Some(include_str!("../../default_prompts/router_summarize.md")),
        _ => None,
    }
}

pub fn default_file(name: &str) -> Option<&'static str> {
    default_constraints_prompt(name)
        .or_else(|| default_kpop_prompt(name))
        .or_else(|| default_router_prompt(name))
        .or_else(|| match name {
            HEADER_MD => Some(include_str!("../../default_prompts/header.md")),
            HEADER_NOKPOP_MD => Some(include_str!("../../default_prompts/header_nokpop.md")),
            DO_HEADER_MD => Some(include_str!("../../default_prompts/do_header.md")),
            WRITE_A_MD => Some(include_str!("../../default_prompts/write_a.md")),
            WRITE_B_MD => Some(include_str!("../../default_prompts/write_b.md")),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::{
        default_constraints_prompt, default_file, default_kpop_prompt, header_prompt_file,
        kpop_common_prompt_file, router_a_prompt_file, router_b_prompt_file, HEADER_NOKPOP_MD,
        KPOP_COMMON_FAKE_MD, ROUTER_A_NOKPOP_MD, ROUTER_B_CREATIVE_NOKPOP_MD, ROUTER_B_NOKPOP_MD,
    };

    #[test]
    fn default_file_covers_router_a_and_b() {
        assert!(default_file("router_a.md").is_some());
        assert!(default_file("router_b.md").is_some());
        assert!(default_file("router_b_creative.md").is_some());
        assert!(default_file("router_summarize.md").is_some());
        assert!(default_file("write_a.md").is_some());
        assert!(default_file("write_b.md").is_some());
        assert!(default_file("inspire_summarize.md").is_some());
        assert!(default_constraints_prompt("init_constraints.md").is_some());
        assert!(default_kpop_prompt("kpop_common.md").is_some());
    }

    #[test]
    fn active_prompt_selectors_always_use_nokpop_templates() {
        assert_eq!(header_prompt_file(), HEADER_NOKPOP_MD);
        assert_eq!(kpop_common_prompt_file(), KPOP_COMMON_FAKE_MD);
        assert_eq!(router_a_prompt_file(), ROUTER_A_NOKPOP_MD);
        assert_eq!(router_b_prompt_file(false), ROUTER_B_NOKPOP_MD);
        assert_eq!(router_b_prompt_file(true), ROUTER_B_CREATIVE_NOKPOP_MD);
    }
}
