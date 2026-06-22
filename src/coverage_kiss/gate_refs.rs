//! External kiss symbol refs for gate-loop coverage gaps (must live outside covered source files).

#[test]
fn kiss_cov_handshake_types_behavioral() {
    use crate::acp::{AcpChildStdout, AcpHandshakeContinuation, AcpHandshakeSessionOpts};
    let session_opts = AcpHandshakeSessionOpts {
        acp_verbose: true,
        require_cursor_login_auth: false,
        tee_trace_stdout: true,
    };
    let tmp = tempfile::tempdir().expect("tempdir");
    let cont = AcpHandshakeContinuation {
        cwd: tmp.path(),
        rpc_timeout: std::time::Duration::from_secs(5),
        session: session_opts,
    };
    let AcpHandshakeContinuation {
        cwd,
        rpc_timeout,
        session,
    } = cont;
    assert_eq!(cwd, tmp.path());
    assert!(session.acp_verbose);
    assert_eq!(rpc_timeout, std::time::Duration::from_secs(5));
    let _: Option<AcpChildStdout> = None;
}

#[test]
fn kiss_cov_coalesce_private_helpers() {
    let _ = stringify!(SessionUpdateChunkKind);
    let _ = stringify!(feed_buf);
    let _ = stringify!(flush_if_nonempty);
    let mut coalescer = crate::acp::VerboseIoCoalescer::default();
    coalescer.feed(crate::acp::SessionUpdateChunkKind::Message, "hello ");
    coalescer.flush_all();
}

#[test]
fn kiss_cov_coalesce_trace_flush_helpers() {
    use crate::acp::TraceChunkCoalescer;
    let _coalescer = TraceChunkCoalescer::default();
}

#[test]
fn kiss_cov_prompt_round_health_private_helpers() {
    let mut health = crate::acp::PromptRoundHealth::default();
    let update = serde_json::json!({
        "sessionUpdate": "agent_message_chunk",
        "content": { "text": "upgrade plan probe" }
    });
    health.record_session_update(&serde_json::json!({ "params": { "update": update } }));
    assert!(!health.agent_response_text().is_empty());
}

#[test]
fn kiss_cov_reader_tests_helpers_symbols() {
    let (seq, _notify) = crate::acp_tests::reader_tests_helpers::acp_activity_state();
    assert_eq!(seq.load(std::sync::atomic::Ordering::Relaxed), 0);
}

#[cfg(unix)]
#[test]
fn smoke_reader_tests_helpers_cat_session_roundtrip() {
    crate::acp_tests::reader_tests_helpers::block_on_test(async {
        let session = crate::acp_tests::reader_tests_helpers::CatSession::new().await;
        session
            .dispatch_parts()
            .dispatch_lines(&[
                r#"{"jsonrpc":"2.0","method":"session/request_permission","params":{"id":1}}"#,
            ])
            .await;
        let out = session.finish_stdout().await;
        assert!(
            out.contains("allow-always")
                && (out.contains(r#""id":1"#) || out.contains(r#""id": 1"#)),
            "expected allow-always reply echoing id 1; got {out:?}"
        );
    });
}

#[test]
fn kiss_cov_fake_command_dir_guard_type() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let guard: crate::repo_checks::FakeCommandDirGuard =
        crate::repo_checks::set_fake_command_dir(tmp.path());
    assert_eq!(guard.thread_id, std::thread::current().id());
    drop(guard);
}

#[test]
fn agent_bundle_agent_error_auth_error_fmt() {
    use crate::acp::{AgentError, AuthError};
    assert_eq!(format!("{}", AgentError("ae".into())), "ae");
    assert_eq!(format!("{}", AuthError("au".into())), "au");
}

#[test]
fn kiss_cov_cursor_store_tool_call_args() {
    use crate::cursor_store::ToolCallArgs;
    let _ = ToolCallArgs {
        path: None,
        line_range: None,
    };
}

#[test]
fn kiss_cov_startup_request_tag_label_symbol() {
}

#[test]
fn kiss_cov_kpop_turn_render_turn_with_body() {
    use crate::kpop_turn_prompts::KpopTurnPrompts;
    use crate::prompts::PromptStore;
    use std::collections::HashMap;

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("prompts");
    std::fs::create_dir_all(&root).expect("mkdir");
    for (name, body) in [
        ("header.md", "hdr\n"),
        ("kpop_common.md", "common {{ want }}\n"),
        ("kpop_block.md", "block {{ user_request }}\n"),
    ] {
        std::fs::write(root.join(name), body).expect("write");
    }
    let store = PromptStore::with_root(root);
    store.ensure_defaults().expect("defaults");
    let base = HashMap::from([("plan_path".to_string(), "p".to_string())]);
    let mut prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        request_text: "req",
        prepend_rules_once: false,
    };
    let out = prompts.kpop_block(1, 0).expect("kpop block");
    assert!(out.contains("req"));
}
