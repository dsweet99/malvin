use std::collections::HashMap;

use crate::kpop_turn_prompts::KpopTurnPrompts;
use crate::prompt_stratification::WorkflowRenderContext;
use crate::prompts::{PromptStore, render_header};

fn kpop_turn_test_context() -> WorkflowRenderContext {
    WorkflowRenderContext::from(HashMap::from([
        (
            "plan_path".to_string(),
            "plan/with $ and\nmulti-line path".to_string(),
        ),
        ("advice_path".to_string(), "./.malvin/advice.md".to_string()),
        ("logs_dir".to_string(), "./.malvin/logs/run42".to_string()),
        ("workspace_dir".to_string(), "./.malvin/logs/run42".to_string()),
        ("exp_log".to_string(), "./.malvin/logs/run/_kpop/exp_log.md".to_string()),
        (
            "user_request_path".to_string(),
            "./.malvin/logs/run/user_request.md".to_string(),
        ),
        (
            "current_state".to_string(),
            "User: test\nRetry: not a retry".to_string(),
        ),
        ("git_extra".to_string(), String::new()),
    ]))
}

fn kpop_turn_test_store() -> (tempfile::TempDir, PromptStore) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("prompts");
    std::fs::create_dir_all(&root).expect("mkdir");
    for (name, body) in [
        ("header.md", "<<hdr plan={{ plan_path }} req={{ user_request_path }}>>\n"),
        ("kpop_common.md", "<<common exp={{ exp_log }}>>\n"),
        ("kpop_block.md", "<<block want={{ want }} user={{ user_request }}>>\n"),
    ] {
        std::fs::write(root.join(name), body).expect("write");
    }
    let store = PromptStore::with_root(root);
    store.ensure_defaults().expect("defaults");
    (tmp, store)
}

#[test]
fn kpop_engine_single_turn_prompt_is_header_common_and_block() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let request_path = "./.malvin/logs/run/user_request.md";
    let exp_path = "./.malvin/logs/run/_kpop/exp_log.md";
    let prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        request_text: "investigate cache",
        prepend_rules_once: false,
    };
    let gate = prompts.kpop_engine_single_turn_prompt(5).expect("gate prompt");
    assert!(gate.contains(request_path));
    assert!(gate.contains(exp_path));
    assert!(gate.contains("investigate cache"));
    assert!(gate.contains("<<block want=5"));
}

#[test]
fn kpop_block_prepends_header_once_then_common_and_block() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let mut prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        request_text: "brief text",
        prepend_rules_once: true,
    };

    let first = prompts.kpop_block(3, 0).expect("first kpop turn");
    let header = render_header(&store, base.as_map()).expect("header");
    assert!(first.contains(header.trim()));
    assert!(first.contains("brief text"));

    let second = prompts.kpop_block(3, 0).expect("second kpop turn");
    assert!(!second.contains(header.trim()));
    assert!(second.contains("brief text"));
}

#[test]
fn kpop_block_without_prepend_rules_never_includes_header() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let mut prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        request_text: "brief",
        prepend_rules_once: false,
    };

    let out = prompts.kpop_block(2, 1).expect("kpop turn");
    let header = render_header(&store, base.as_map()).expect("header");
    assert!(!out.contains(header.trim()));
    assert!(out.contains("brief"));
}

#[test]
fn kpop_block_allows_user_request_with_double_braces() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let request = "Expand {{ code_extra }} in router_b_* templates.";
    let prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        request_text: request,
        prepend_rules_once: false,
    };
    let gate = prompts.kpop_engine_single_turn_prompt(5).expect("gate prompt");
    assert!(gate.contains(request));
}
