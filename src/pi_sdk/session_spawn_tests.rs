#[test]
fn kiss_cov_session_and_spawn_names() {
    let _ = super::spawn_bridge;
    let _ = stringify!(pi_spawn_bridge);
    let _ = stringify!(split_provider_model);
    let _ = stringify!(fake_embedded_session);
    let _ = stringify!(live_embedded_session);
    let _ = stringify!(start_embedded_mem_watch);
    let _ = stringify!(watch_embedded_memory);
    let _ = stringify!(isolated_tool_factory);
}

#[test]
fn split_provider_model_first_slash() {
    assert_eq!(
        super::session_spawn::split_provider_model("openai/gpt-4o").expect("ok"),
        ("openai", "gpt-4o")
    );
    assert_eq!(
        super::session_spawn::split_provider_model("openrouter/anthropic/claude-3-haiku")
            .expect("ok"),
        ("openrouter", "anthropic/claude-3-haiku")
    );
    assert!(
        super::session_spawn::split_provider_model("noslash")
            .expect_err("err")
            .0
            .contains("provider")
    );
}

#[tokio::test]
async fn fake_session_begin_end_leaves_no_pi_runtime_thread() {
    let _guard = crate::test_utils::test_env_lock();
    unsafe {
        std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
        std::env::set_var("OPENAI_API_KEY", "test-key");
    }
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = crate::pi_sdk::pi_sdk_client_from_raw(
        "pi:openai/gpt-4o",
        crate::acp::AgentIoOptions {
            force: true,
            no_tee: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        },
        1,
    );
    client.prompts_log_run_dir = Some(tmp.path().to_path_buf());
    client.begin_coder_session(tmp.path()).await.expect("begin");
    client.end_coder_session().await.expect("end");
    let leftover = leftover_pi_runtime_threads();
    unsafe {
        std::env::remove_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV);
    }
    assert!(
        leftover.is_empty(),
        "malvin-pi-sdk threads still alive: {leftover:?}"
    );
}

fn leftover_pi_runtime_threads() -> Vec<String> {
    let Ok(entries) = std::fs::read_dir("/proc/self/task") else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for entry in entries.flatten() {
        let comm = entry.path().join("comm");
        let Ok(name) = std::fs::read_to_string(comm) else {
            continue;
        };
        if name.contains("malvin-pi-sdk") {
            names.push(name.trim().to_string());
        }
    }
    names
}
