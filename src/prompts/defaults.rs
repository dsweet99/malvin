//! Embedded default prompt bodies (`default_prompts/`).

pub const REQUIRED_PROMPTS: &[&str] = &[
    "implement.md",
    "review_1.md",
    "review_2.md",
    "kpop.md",
    "concerns.md",
    "header.md",
    "coding_rules.md",
];

pub const DEFAULT_PROMPTS: &[&str] = &[
    "implement.md",
    "review_1.md",
    "review_2.md",
    "kpop.md",
    "mbc2.md",
    "concerns.md",
    "learn.md",
    "header.md",
    "coding_rules.md",
];

pub fn default_file(name: &str) -> Option<&'static str> {
    match name {
        "implement.md" => Some(include_str!("../../default_prompts/implement.md")),
        "review_1.md" => Some(include_str!("../../default_prompts/review_1.md")),
        "review_2.md" => Some(include_str!("../../default_prompts/review_2.md")),
        "kpop.md" => Some(include_str!("../../default_prompts/kpop.md")),
        "mbc2.md" => Some(include_str!("../../default_prompts/mbc2.md")),
        "concerns.md" => Some(include_str!("../../default_prompts/concerns.md")),
        "learn.md" => Some(include_str!("../../default_prompts/learn.md")),
        "header.md" => Some(include_str!("../../default_prompts/header.md")),
        "coding_rules.md" => Some(include_str!("../../default_prompts/coding_rules.md")),
        _ => None,
    }
}
