//! Embedded default prompt bodies (`default_prompts/`).

pub const HEADER_MD: &str = "header.md";
pub const DO_HEADER_MD: &str = "do_header.md";

pub const REQUIRED_PROMPTS: &[&str] = &[
    "check_plan.md",
    "implement.md",
    "review_1.md",
    "review_2.md",
    "concerns.md",
    HEADER_MD,
    "coding_rules.md",
];

pub const DEFAULT_PROMPTS: &[&str] = &[
    "check_plan.md",
    "implement.md",
    "review_1.md",
    "review_2.md",
    "kpop.md",
    "kpop_common.md",
    "kpop_block.md",
    "mbc2_pure.md",
    "mbc2.md",
    "concerns.md",
    "learn.md",
    HEADER_MD,
    DO_HEADER_MD,
    "coding_rules.md",
];

pub fn default_file(name: &str) -> Option<&'static str> {
    match name {
        "check_plan.md" => Some(include_str!("../../default_prompts/check_plan.md")),
        "implement.md" => Some(include_str!("../../default_prompts/implement.md")),
        "review_1.md" => Some(include_str!("../../default_prompts/review_1.md")),
        "review_2.md" => Some(include_str!("../../default_prompts/review_2.md")),
        "kpop.md" | "kpop_common.md" => {
            Some(include_str!("../../default_prompts/kpop_common.md"))
        }
        "kpop_block.md" => Some(include_str!("../../default_prompts/kpop_block.md")),
        "mbc2_pure.md" => Some(include_str!("../../default_prompts/mbc2_pure.md")),
        "mbc2.md" => Some(include_str!("../../default_prompts/mbc2.md")),
        "concerns.md" => Some(include_str!("../../default_prompts/concerns.md")),
        "learn.md" => Some(include_str!("../../default_prompts/learn.md")),
        HEADER_MD => Some(include_str!("../../default_prompts/header.md")),
        DO_HEADER_MD => Some(include_str!("../../default_prompts/do_header.md")),
        "coding_rules.md" => Some(include_str!("../../default_prompts/coding_rules.md")),
        _ => None,
    }
}
