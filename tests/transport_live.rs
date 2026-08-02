//! Opt-in live `LlmTransport` integration tests (network / Metal GPU).
//!
//! ```text
//! MALVIN_LIVE_TRANSPORT=1 OPENROUTER_API_KEY=... cargo nextest run -E 'test(transport_live)' -- --ignored
//! MALVIN_LIVE_LOCAL=1 cargo nextest run -E 'test(transport_live)' -- --ignored
//! ```
//!
//! Local/GPU cases are Metal-only and stay disabled by default.

#![cfg(unix)]

mod common;

use common::require_openrouter_key_when_gate_set;

fn live_transport_gate_set() -> bool {
    std::env::var_os("MALVIN_LIVE_TRANSPORT").is_some_and(|v| v == "1")
}

fn live_local_gate_set() -> bool {
    std::env::var_os("MALVIN_LIVE_LOCAL").is_some_and(|v| v == "1")
}

#[test]
fn transport_live_tests_compile_and_skip_without_env() {
    let _ = (live_transport_gate_set(), live_local_gate_set());
}

#[tokio::test]
#[ignore = "live OpenRouter LlmTransport; MALVIN_LIVE_TRANSPORT=1 OPENROUTER_API_KEY=... cargo nextest run -E 'test(transport_live)' -- --ignored"]
async fn transport_live_openrouter_ensure_ready_and_complete() {
    if !live_transport_gate_set() {
        return;
    }
    require_openrouter_key_when_gate_set("MALVIN_LIVE_TRANSPORT");
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
    if !live_local_gate_set() {
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
