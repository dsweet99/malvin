use super::{run_router_models, should_fallback};
use crate::cli::router_flow::{new_router_client, prepare_router_prompt_store};
use crate::cli::router_flow_loop::RouterAgentLoopInput;
use crate::cli::{RouterOpts, SharedOpts};
use malvin::test_support::test_utils::{SavedEnvVars, block_on_test_async, with_isolated_home};

const BRIDGE: &str = r#"
import fs from 'node:fs';
import readline from 'node:readline';
let model;
let turns = 0;
const emit = obj => process.stdout.write(JSON.stringify(obj) + '\n');
for await (const line of readline.createInterface({input: process.stdin})) {
    const req = JSON.parse(line);
    if (req.op === 'create' || req.op === 'resume') {
        model = req.model;
        fs.appendFileSync(process.env.FALLBACK_ATTEMPTS, model + '\n');
        emit({event: 'ok', agentId: 'test-agent'});
    } else if (req.op === 'send') {
        turns += 1;
        if (model.startsWith('poor') || (model === 'late-poor' && turns === 2)) {
            emit({event: 'fatal', message: 'Insufficient funds: ' + model, retryable: false});
        } else if (model === 'limited') {
            emit({event: 'fatal', message: 'codex RPC error: {"codexErrorInfo":"usageLimitExceeded","message":"You\u2019ve hit your usage limit."}', retryable: false});
        } else if (model === 'invalid') {
            emit({event: 'fatal', message: 'cannot use this model', retryable: false});
        } else {
            emit({event: 'run_done', status: 'finished', result: 'MALVIN_DONE', durationMs: 1});
        }
    } else if (req.op === 'close') {
        break;
    }
}
"#;

fn with_bridge(body: impl FnOnce(&std::path::Path)) {
    malvin::backends::cursor_sdk::node_resolve::resolve_node_bin().expect("node");
    with_isolated_home(|work| {
        let _env = SavedEnvVars::capture(&[
            "MALVIN_CURSOR_SDK_BRIDGE",
            "CURSOR_API_KEY",
            "MALVIN_TEST_NO_REAL_AGENT",
            "FALLBACK_ATTEMPTS",
        ]);
        let bridge = work.join("bridge.mjs");
        std::fs::write(&bridge, BRIDGE).unwrap();
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("MALVIN_CURSOR_SDK_BRIDGE", bridge);
            std::env::set_var("CURSOR_API_KEY", "test-key");
            std::env::set_var("FALLBACK_ATTEMPTS", work.join("attempts"));
        }
        malvin::test_support::test_utils::enable_test_fast_teardown();
        body(work);
    });
}

fn run_models(work: &std::path::Path, models: &str) -> Result<(), String> {
    let mut shared = SharedOpts::test_defaults();
    shared.model = malvin::config::malvin_config_file::parse_model_list_cli_arg(models).unwrap();
    let artifacts = malvin::artifacts::create_run_artifacts_from_text_opts(
        "fallback test",
        Some(work),
        malvin::workspace::run_id::RunDirOptions::default(),
    )
    .unwrap();
    let store = prepare_router_prompt_store().unwrap();
    let mut client = new_router_client(&shared);
    block_on_test_async(run_router_models(RouterAgentLoopInput {
        client: &mut client,
        artifacts: &artifacts,
        prompt_store: &store,
        shared: &shared,
        router: &RouterOpts::test_defaults(),
        max_loops: 1,
    }))
}

#[test]
fn funds_failure_tries_next_model_and_each_task_restarts_at_first() {
    with_bridge(|work| {
        for _ in 0..2 {
            run_models(work, "cursor:poor,cursor:good,cursor:unused").unwrap();
        }
        assert_eq!(
            std::fs::read_to_string(work.join("attempts")).unwrap(),
            "poor\ngood\npoor\ngood\n"
        );
    });
}

#[test]
fn exhausted_models_return_last_error() {
    with_bridge(|work| {
        assert!(
            run_models(work, "cursor:poor,cursor:poor-again")
                .unwrap_err()
                .contains("Insufficient funds: poor-again")
        );
        assert_eq!(
            std::fs::read_to_string(work.join("attempts")).unwrap(),
            "poor\npoor-again\n"
        );
    });
}

#[test]
fn other_failures_and_single_model_do_not_fallback() {
    with_bridge(|work| {
        assert!(
            run_models(work, "cursor:invalid,cursor:good")
                .unwrap_err()
                .contains("cannot use this model")
        );
        assert!(
            run_models(work, "cursor:poor")
                .unwrap_err()
                .contains("Insufficient funds")
        );
        assert_eq!(
            std::fs::read_to_string(work.join("attempts")).unwrap(),
            "invalid\npoor\n"
        );
    });
    assert!(!should_fallback(
        "ABORT: operator stopped; Insufficient funds"
    ));
    assert!(!should_fallback(
        "Insufficient funds; gitignore restore: failed"
    ));
    assert!(!should_fallback(
        "Insufficient funds; workspace session restore failed"
    ));
    assert!(!should_fallback("ABORT: You've hit your usage limit"));
    assert!(!should_fallback("rate limit exceeded"));
    assert!(should_fallback(
        "codex RPC error: {\"codexErrorInfo\":\"usageLimitExceeded\",\"message\":\"You\u{2019}ve hit your usage limit.\"}"
    ));
    assert!(should_fallback("You're out of usage"));
    assert!(should_fallback("You've hit your usage limit"));
    assert!(should_fallback("Upgrade your plan to continue"));
}

#[test]
fn usage_limit_tries_next_model() {
    with_bridge(|work| {
        run_models(work, "cursor:limited,cursor:good").unwrap();
        let err = run_models(work, "cursor:limited").unwrap_err();
        assert!(err.contains("usageLimitExceeded"), "{err}");
        assert_eq!(
            std::fs::read_to_string(work.join("attempts")).unwrap(),
            "limited\ngood\nlimited\n"
        );
    });
}

#[test]
fn first_model_success_does_not_try_backups() {
    with_bridge(|work| {
        run_models(work, "cursor:good,cursor:unused").unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join("attempts")).unwrap(),
            "good\n"
        );
    });
}

#[test]
fn payment_failure_after_initial_turn_still_falls_back() {
    with_bridge(|work| {
        run_models(work, "cursor:late-poor,cursor:good").unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join("attempts")).unwrap(),
            "late-poor\ngood\n"
        );
    });
}
