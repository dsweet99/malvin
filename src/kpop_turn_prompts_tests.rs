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
        ("exp_log".to_string(), "./.malvin/logs/run/_kpop/exp_log.md".to_string()),
        (
            "user_request_path".to_string(),
            "./.malvin/logs/run/user_request.md".to_string(),
        ),
        (
            "current_state".to_string(),
            "User: test\nRetry: not a retry".to_string(),
        ),
    ]))
}

fn kpop_turn_test_store() -> (tempfile::TempDir, PromptStore) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("prompts");
    std::fs::create_dir_all(&root).expect("mkdir");
    for (name, body) in [
        ("header.md", "<<hdr plan={{ plan_path }} req={{ user_request_path }}>>\n"),
        ("kpop_common.md", "<<common exp={{ exp_log }}>>\n"),
    ] {
        std::fs::write(root.join(name), body).expect("write");
    }
    let store = PromptStore::with_root(root);
    store.ensure_defaults().expect("defaults");
    (tmp, store)
}

fn expected_kpop_prompt_output(
    store: &PromptStore,
    ctx: &WorkflowRenderContext,
    with_rules: bool,
) -> String {
    let map = ctx.as_map();
    let common = store
        .render_prompt_only("kpop_common.md", map)
        .expect("common");
    if with_rules {
        let header = render_header(store, map).expect("header");
        format!("{}\n\n{}", header.trim_end(), common.trim_end())
    } else {
        common.trim_end().to_string()
    }
}

#[test]
fn kpop_engine_single_turn_prompt_is_header_plus_common() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let request_path = "./.malvin/logs/run/user_request.md";
    let exp_path = "./.malvin/logs/run/_kpop/exp_log.md";
    let prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        prepend_rules_once: false,
    };
    let gate = prompts.kpop_engine_single_turn_prompt().expect("gate prompt");
    let map = base.as_map();
    let header = store
        .render_prompt_only("header.md", map)
        .expect("header");
    let common = store
        .render_prompt_only("kpop_common.md", map)
        .expect("common");
    let expected = format!("{}\n\n{}", header.trim_end(), common.trim_end());
    assert_eq!(gate, expected);
    assert!(gate.contains(request_path));
    assert!(gate.contains(exp_path));
}

#[test]
fn kpop_prompt_matches_independently_rendered_sections() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let mut prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        prepend_rules_once: true,
    };

    let first = prompts.kpop_prompt().expect("first kpop turn");
    assert_eq!(
        first,
        expected_kpop_prompt_output(&store, &base, true),
        "first turn should equal header + common"
    );

    let second = prompts.kpop_prompt().expect("second kpop turn");
    assert_eq!(
        second,
        expected_kpop_prompt_output(&store, &base, false),
        "after prepend_rules_once is consumed, output should omit header"
    );
}

#[test]
fn kpop_prompt_without_prepend_rules_never_includes_header() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let mut prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        prepend_rules_once: false,
    };

    let out = prompts.kpop_prompt().expect("kpop turn");
    assert_eq!(
        out,
        expected_kpop_prompt_output(&store, &base, false),
        "prepend_rules_once=false should never prepend header"
    );
    let header = render_header(&store, base.as_map()).expect("header");
    assert!(
        !out.contains(header.trim()),
        "output must not contain rendered header fragment:\nheader={header:?}\nout={out:?}"
    );
}
