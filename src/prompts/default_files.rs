use super::{
    DO_HEADER_MD, EXPLAIN_WRAPPER_MD, HEADER_MD, ROUTER_CODE_EXTRA_MD, ROUTER_KPOP_GROUP_MD,
    ROUTER_REQUIREMENTS_MD, ROUTER_SUMMARIZE_MD, ROUTER_WORK_MD,
};

fn default_constraints_prompt(name: &str) -> Option<&'static str> {
    match name {
        "code_constraints.md" => Some(include_str!("../../default_prompts/code_constraints.md")),
        "init_constraints.md" => Some(include_str!("../../default_prompts/init_constraints.md")),
        "mini_constraints.md" => Some(include_str!("../../default_prompts/mini_constraints.md")),
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
        "kpop_program.md" => Some(include_str!("../../default_prompts/kpop_program.md")),
        "kpop_summarize.md" => Some(include_str!("../../default_prompts/kpop_summarize.md")),
        "kpop_block.md" => Some(include_str!("../../default_prompts/kpop_block.md")),
        _ => None,
    })
}

fn default_router_prompt(name: &str) -> Option<&'static str> {
    match name {
        ROUTER_REQUIREMENTS_MD => {
            Some(include_str!("../../default_prompts/router_requirements.md"))
        }
        ROUTER_KPOP_GROUP_MD => Some(include_str!("../../default_prompts/router_kpop_group.md")),
        ROUTER_WORK_MD => Some(include_str!("../../default_prompts/router_work.md")),
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
            EXPLAIN_WRAPPER_MD => Some(include_str!("../../default_prompts/explain_wrapper.md")),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::{default_constraints_prompt, default_file, default_kpop_prompt};

    #[test]
    fn default_file_mini_constraints_embedded() {
        let text = default_file("mini_constraints.md").expect("mini_constraints");
        assert!(text.contains("```bash"));
        assert!(text.contains("openrouter:"));
    }

    #[test]
    fn default_constraints_prompt_embeds_code_and_init() {
        assert!(default_constraints_prompt("code_constraints.md").is_some());
        assert!(default_constraints_prompt("init_constraints.md").is_some());
        assert!(default_constraints_prompt("missing.md").is_none());
        assert!(default_kpop_prompt("mbc2.md").is_some());
        assert!(default_kpop_prompt("missing.md").is_none());
        assert!(default_file("code_constraints.md").is_some());
    }
}
