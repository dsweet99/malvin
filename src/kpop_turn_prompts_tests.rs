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
            "./.malvin/logs/run/request.md".to_string(),
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
        ("header.md", "<<hdr plan={{ plan_path }}>>\n"),
        ("kpop_common.md", "<<common>>\n"),
        ("mpc_block.md", "<<block req={{ user_request_path }}>>\n"),
        ("mbc2.md", "MBC2\n"),
    ] {
        std::fs::write(root.join(name), body).expect("write");
    }
    let store = PromptStore::with_root(root);
    store.ensure_defaults().expect("defaults");
    (tmp, store)
}

fn expected_kpop_block_output(
    store: &PromptStore,
    ctx: &WorkflowRenderContext,
    with_rules: bool,
) -> String {
    let map = ctx.as_map();
    let common = store
        .render_prompt_only("kpop_common.md", map)
        .expect("common");
    let body = store
        .render_prompt_only("mpc_block.md", map)
        .expect("block");
    if with_rules {
        let header = render_header(store, map).expect("header");
        format!(
            "{}\n\n{}\n\n{}",
            header.trim_end(),
            common.trim_end(),
            body.trim_end()
        )
    } else {
        format!("{}\n\n{}", common.trim_end(), body.trim_end())
    }
}

#[test]
fn render_turn_with_body_matches_kpop_engine_single_turn_without_header() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let request_path = "./.malvin/logs/run/request.md";
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
    let body = store
        .render_prompt_only("mpc_block.md", map)
        .expect("block");
    let expected = format!(
        "{}\n\n{}\n\n{}",
        header.trim_end(),
        common.trim_end(),
        body.trim_end()
    );
    assert_eq!(gate, expected);
    assert!(gate.contains(request_path));
    assert!(
        !gate.contains("budget for any KPOPs"),
        "gate prompt must not include hypothesis budget wording"
    );
}

#[test]
fn kpop_block_matches_independently_rendered_sections() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let mut prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        prepend_rules_once: true,
    };

    let first = prompts.kpop_block().expect("first kpop turn");
    assert_eq!(
        first,
        expected_kpop_block_output(&store, &base, true),
        "first turn should equal header + common + block with exact composition"
    );

    let second = prompts.kpop_block().expect("second kpop turn");
    assert_eq!(
        second,
        expected_kpop_block_output(&store, &base, false),
        "after prepend_rules_once is consumed, output should omit header"
    );
}

#[test]
fn kpop_block_without_prepend_rules_never_includes_header() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let mut prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        prepend_rules_once: false,
    };

    let out = prompts.kpop_block().expect("kpop turn");
    assert_eq!(
        out,
        expected_kpop_block_output(&store, &base, false),
        "prepend_rules_once=false should never prepend header"
    );
    let header = render_header(&store, base.as_map()).expect("header");
    assert!(
        !out.contains(header.trim()),
        "output must not contain rendered header fragment:\nheader={header:?}\nout={out:?}"
    );
}
