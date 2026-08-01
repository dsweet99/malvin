//! Opt-in live `LlmTransport` integration tests (network / Metal GPU).
//!
//! ```text
//! MALVIN_LIVE_TRANSPORT=1 OPENROUTER_API_KEY=... cargo nextest run -E 'test(transport_live)' -- --ignored
//! MALVIN_LIVE_LOCAL=1 cargo nextest run -E 'test(transport_live)' -- --ignored
//! ```
//!
//! Local/GPU cases are Metal-only and stay disabled by default.

fn live_transport_prereqs_met() -> bool {
    std::env::var_os("MALVIN_LIVE_TRANSPORT").is_some_and(|v| v == "1")
        && std::env::var_os("OPENROUTER_API_KEY").is_some_and(|v| !v.is_empty())
}

fn live_local_prereqs_met() -> bool {
    std::env::var_os("MALVIN_LIVE_LOCAL").is_some_and(|v| v == "1")
}

#[test]
fn transport_live_tests_compile_and_skip_without_env() {
    assert!(
        !live_transport_prereqs_met()
            || std::env::var_os("OPENROUTER_API_KEY").is_some_and(|v| !v.is_empty())
    );
    let _ = live_local_prereqs_met();
}

#[tokio::test]
#[ignore = "live OpenRouter LlmTransport; MALVIN_LIVE_TRANSPORT=1 OPENROUTER_API_KEY=... cargo nextest run -E 'test(transport_live)' -- --ignored"]
async fn transport_live_openrouter_ensure_ready_and_complete() {
    if !live_transport_prereqs_met() {
        eprintln!("skip: set MALVIN_LIVE_TRANSPORT=1 and OPENROUTER_API_KEY to run");
        return;
    }
    let cfg = malvin::openrouter_transport::OpenRouterConfig::from_env(
        malvin::mini_agent::resolve_mini_model("openrouter:auto"),
    )
    .expect("openrouter config");
    let transport = malvin::llm_transport::OpenRouterTransport::new(cfg).expect("transport");
    let wrapped = malvin::llm_transport::LlmTransport::OpenRouter(transport);
    wrapped.ensure_ready().expect("ensure_ready");
    let messages = [malvin::llm_transport::ChatMessage {
        role: malvin::llm_transport::ChatRole::User,
        content: "Reply with exactly: pong".into(),
    }];
    let ok = wrapped.complete(&messages).await.expect("complete");
    assert!(!ok.content.trim().is_empty(), "empty live completion");
}

#[tokio::test]
#[ignore = "live Local LlmTransport (Metal); MALVIN_LIVE_LOCAL=1 cargo nextest run -E 'test(transport_live)' -- --ignored"]
async fn transport_live_local_ensure_ready_and_complete() {
    if !live_local_prereqs_met() {
        eprintln!("skip: set MALVIN_LIVE_LOCAL=1 to run real local/GPU transport");
        return;
    }
    let engine = malvin::local_llm::ensure_local_engine(
        "local:nemotron3_nano_4b",
        malvin::local_llm::DownloadPolicy::Allow,
    )
    .expect("ensure_local_engine");
    let wrapped =
        malvin::llm_transport::LlmTransport::Local(malvin::llm_transport::LocalLlmTransport::new(
            engine,
        ));
    wrapped.ensure_ready().expect("ensure_ready");
    let messages = [malvin::llm_transport::ChatMessage {
        role: malvin::llm_transport::ChatRole::User,
        content: "Say hi in one word.".into(),
    }];
    let ok = wrapped.complete(&messages).await.expect("local complete");
    assert!(!ok.content.trim().is_empty(), "empty local completion");
}
