use std::collections::HashMap;

use crate::kpop_turn_prompts::KpopTurnPrompts;
use crate::prompts::PromptStore;

fn kpop_turn_test_context() -> HashMap<String, String> {
    HashMap::from([
        ("plan_path".to_string(), "p".to_string()),
        ("advice_path".to_string(), "./.malvin/advice.md".to_string()),
        ("exp_log".to_string(), "./.malvin/logs/run/_kpop/exp_log.md".to_string()),
        (
            "current_state".to_string(),
            "User: test\nRetry: not a retry".to_string(),
        ),
    ])
}

#[test]
fn kpop_turn_prompts_render() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("prompts");
    std::fs::create_dir_all(&root).expect("mkdir");
    for name in [
        "kpop_common.md",
        "kpop_block.md",
        "mbc2.md",
        "coding_rules.md",
    ] {
        std::fs::write(root.join(name), "body").expect("write");
    }
    let store = PromptStore::with_root(root);
    store.ensure_defaults().expect("defaults");
    let ctx = kpop_turn_test_context();
    let mut prompts = KpopTurnPrompts {
        store: &store,
        base: &ctx,
        request_text: "req",
        prepend_rules_once: true,
    };
    let _ = prompts.kpop_block(1, 0).expect("kpop");
}
