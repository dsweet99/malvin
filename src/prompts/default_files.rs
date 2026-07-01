use super::{DO_HEADER_MD, HEADER_MD};

fn default_constraints_prompt(name: &str) -> Option<&'static str> {
    match name {
        "tidy_constraints.md" => Some(include_str!("../../default_prompts/tidy_constraints.md")),
        "code_constraints.md" => Some(include_str!("../../default_prompts/code_constraints.md")),
        "init_constraints.md" => Some(include_str!("../../default_prompts/init_constraints.md")),
        "delight_constraints.md" => Some(include_str!("../../default_prompts/delight_constraints.md")),
        "explain_constraints.md" => Some(include_str!("../../default_prompts/explain_constraints.md")),
        "revise_constraints.md" => Some(include_str!("../../default_prompts/revise_constraints.md")),
        "mini_constraints.md" => Some(include_str!("../../default_prompts/mini_constraints.md")),
        _ => None,
    }
}

fn default_kpop_prompt(name: &str) -> Option<&'static str> {
    match name {
        "kpop.md" | "kpop_common.md" => Some(include_str!("../../default_prompts/kpop_common.md")),
        "kpop_block.md" => Some(include_str!("../../default_prompts/kpop_block.md")),
        "kpop_program.md" => Some(include_str!("../../default_prompts/kpop_program.md")),
        "kpop_program_creative.md" => {
            Some(include_str!("../../default_prompts/kpop_program_creative.md"))
        }
        "kpop_summarize.md" => Some(include_str!("../../default_prompts/kpop_summarize.md")),
        "mbc2.md" => Some(include_str!("../../default_prompts/mbc2.md")),
        _ => None,
    }
}

pub fn default_file(name: &str) -> Option<&'static str> {
    default_constraints_prompt(name)
        .or_else(|| default_kpop_prompt(name))
        .or_else(|| match name {
            HEADER_MD => Some(include_str!("../../default_prompts/header.md")),
            DO_HEADER_MD => Some(include_str!("../../default_prompts/do_header.md")),
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
        assert!(text.contains("--mini"));
    }

    #[test]
    fn default_constraints_prompt_embeds_code_and_tidy() {
        assert!(default_constraints_prompt("tidy_constraints.md").is_some());
        assert!(default_constraints_prompt("code_constraints.md").is_some());
        assert!(default_constraints_prompt("init_constraints.md").is_some());
        assert!(default_constraints_prompt("delight_constraints.md").is_some());
        assert!(default_constraints_prompt("explain_constraints.md").is_some());
        assert!(default_constraints_prompt("revise_constraints.md").is_some());
        assert!(default_constraints_prompt("missing.md").is_none());
        assert!(default_kpop_prompt("kpop_block.md").is_some());
        assert!(default_kpop_prompt("missing.md").is_none());
        assert!(default_file("code_constraints.md").is_some());
    }
}
