// Shared helpers for `coverage_kiss` smoke tests.


pub(super) fn kiss_test_tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

pub(super) fn kiss_write_same_body_files(dir: &Path, names: &[&str], body: &[u8]) {
    std::fs::create_dir_all(dir).expect("mkdir");
    for name in names {
        std::fs::write(dir.join(name), body).expect("write prompt");
    }
}

pub(super) const SMOKE_EMBEDDED_PROMPT_NAMES: &[&str] = &[
    "implement.md",
    "reviewers_spawn.md",
    "review_write.md",
    "kpop.md",
    "kpop_common.md",
    "kpop_block.md",
    "mbc2_pure.md",
    "mbc2.md",
    "concerns.md",
    "learn.md",
    crate::prompts::HEADER_MD,
    crate::prompts::DO_HEADER_MD,
    "coding_rules.md",
];

pub(super) fn kiss_smoke_prompt_store(prompts_dir: &Path) -> crate::prompts::PromptStore {
    kiss_write_same_body_files(prompts_dir, SMOKE_EMBEDDED_PROMPT_NAMES, b"body");
    let store = crate::prompts::PromptStore::with_root(prompts_dir.to_path_buf());
    store.ensure_defaults().expect("defaults");
    store.validate_required().expect("required");
    store
}

pub(super) fn kiss_smoke_render_context() -> HashMap<String, String> {
    HashMap::from([
        ("plan_path".to_string(), "p".to_string()),
        ("kpop_log_dir".to_string(), "./_kpop".to_string()),
        ("quality_gates".to_string(), String::new()),
    ])
}
