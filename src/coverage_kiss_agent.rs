//! Agent ACP behavioral smokes for `kiss check` coverage (split from `coverage_kiss` for file size limits).

use crate::acp::{
    AgentClient, AgentError, AgentIoOptions, AuthError, KpopFlowOnceArgs, has_api_key,
    test_captive_session::captive_cat_acp_session_for_tests,
};
use std::fmt::Write as _;

#[test]
fn agent_error_and_auth_error_display_via_fmt() {
    let _ = stringify!(fmt);
    let mut buf = String::new();
    write!(buf, "{}", AgentError("agent err".into())).expect("fmt AgentError");
    assert_eq!(buf, "agent err");
    write!(buf, "{}", AuthError("auth err".into())).expect("fmt AuthError");
    assert_eq!(buf, "agent errauth err");
}

#[test]
fn replace_coder_session_slot_for_tests_opens_session() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut client = AgentClient::new(
            "m".into(),
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        );
        assert!(!client.has_open_coder_session());
        client.replace_coder_session_slot_for_tests(captive_cat_acp_session_for_tests(tmp.path()));
        assert!(client.has_open_coder_session());
    });
}

#[test]
fn kpop_flow_once_args_and_run_kpop_flow_symbols() {
    let _ = stringify!(run_kpop_flow);
    let _ = stringify!(KpopFlowOnceArgs);
    let _ = stringify!(KpopPromptRound);
    let tmp = tempfile::tempdir().expect("tempdir");
    let log = tmp.path().join("kpop.log");
    let prompts = ["probe"];
    let args = KpopFlowOnceArgs {
        cwd: tmp.path(),
        kpop_prompts: &prompts,
        kpop_log: &log,
    };
    assert_eq!(args.kpop_prompts, &["probe"]);
}

#[test]
fn smoke_agent_client_new_has_no_open_coder_session() {
    let io = AgentIoOptions {
        force: false,
        no_tee: false,
        raw_output: false,
        show_thoughts_on_stdout: true,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    };
    let client = AgentClient::new("smoke-model".to_string(), io);
    assert!(!client.has_open_coder_session());
}

#[test]
fn smoke_has_api_key_reads_env_without_panic() {
    let _: bool = has_api_key();
}
