use std::collections::HashMap;

use crate::kpop_turn_prompts::KpopTurnPrompts;
use crate::prompts::PromptStore;

#[test]
fn kpop_turn_prompts_render() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("prompts");
    std::fs::create_dir_all(&root).expect("mkdir");
    for name in [
        "kpop_common.md",
        "kpop_block.md",
        "mbc2_pure.md",
        "coding_rules.md",
    ] {
        std::fs::write(root.join(name), "body").expect("write");
    }
    let store = PromptStore::with_root(root);
    store.ensure_defaults().expect("defaults");
    let ctx = HashMap::from([("plan_path".to_string(), "p".to_string())]);
    let mut prompts = KpopTurnPrompts {
        store: &store,
        base: &ctx,
        request_text: "req",
        prepend_rules_once: true,
    };
    let _ = prompts.kpop_block(1, 0).expect("kpop");
    let _ = prompts.mbc2_pure().expect("mbc2");
}
