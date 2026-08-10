use super::{
    DO_HEADER_MD, WRITE_WRAPPER_MD, HEADER_MD, ROUTER_A_MD, ROUTER_B_MD, ROUTER_CODE_EXTRA_MD,
    ROUTER_SUMMARIZE_MD,
};

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

fn default_kpop_prompt(name: &str) -> Option<&'static str> {
    default_mbc2_prompt(name).or_else(|| match name {
        "kpop.md" | "kpop_common.md" => Some(include_str!("../../default_prompts/kpop_common.md")),
        "kpop_summarize.md" => Some(include_str!("../../default_prompts/kpop_summarize.md")),
        "kpop_block.md" => Some(include_str!("../../default_prompts/kpop_block.md")),
        _ => None,
    })
}

fn default_router_prompt(name: &str) -> Option<&'static str> {
    match name {
        ROUTER_A_MD => Some(include_str!("../../default_prompts/router_a.md")),
        ROUTER_B_MD => Some(include_str!("../../default_prompts/router_b.md")),
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
            DO_HEADER_MD => Some(include_str!("../../default_prompts/do_header.md")),
            WRITE_WRAPPER_MD => Some(include_str!("../../default_prompts/write_wrapper.md")),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::{default_constraints_prompt, default_file, default_kpop_prompt};

    #[test]
    fn default_file_covers_router_a_and_b() {
        assert!(default_file("router_a.md").is_some());
        assert!(default_file("router_b.md").is_some());
        assert!(default_file("router_summarize.md").is_some());
        assert!(default_constraints_prompt("init_constraints.md").is_some());
        assert!(default_kpop_prompt("kpop_common.md").is_some());
    }
}
