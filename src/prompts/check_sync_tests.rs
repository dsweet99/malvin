use super::{DEFAULT_PROMPTS, REQUIRED_PROMPTS, default_file};

#[test]
fn default_file_contains_check_sync_prompt() {
    assert!(default_file("check_sync.md").is_some());
}

#[test]
fn check_sync_prompt_is_shipped_in_default_prompt_lists() {
    assert!(
        DEFAULT_PROMPTS.contains(&"check_sync.md"),
        "expected check_sync in DEFAULT_PROMPTS"
    );
    assert!(
        REQUIRED_PROMPTS.contains(&"check_sync.md"),
        "expected check_sync in REQUIRED_PROMPTS"
    );
}
