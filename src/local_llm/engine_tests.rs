//! Unit tests for [`super`] (`LocalCompletionEngine`).

use super::*;
use crate::llm_transport::{ChatMessage, ChatRole};
use crate::local_llm::DownloadPolicy;
use crate::local_llm::registry::require_known_local_slug;

#[test]
fn local_slug_requires_local_prefix() {
    assert_eq!(
        local_slug("local:qwen35_9b_q4").expect("ok"),
        "qwen35_9b_q4"
    );
    assert!(local_slug("openrouter:x").is_err());
}

#[test]
fn ensure_local_engine_rejects_non_local_ids() {
    assert!(ensure_local_engine("openrouter:x", DownloadPolicy::Deny).is_err());
}

#[test]
fn require_mem_limit_for_local_mentions_config_key() {
    let spec = require_known_local_slug("qwen35_9b_q4").expect("spec");
    match require_mem_limit_for_local(spec) {
        Ok(()) => assert!(
            crate::mem_limit_config::load_mem_limit_gb(
                &std::env::current_dir().unwrap_or_else(|_| ".".into())
            ) >= spec.min_mem_limit_gb
        ),
        Err(e) => {
            assert!(e.contains("mem_limit_gb"));
            assert!(e.contains(&spec.min_mem_limit_gb.to_string()));
        }
    }
}

#[test]
fn messages_to_turns_maps_roles() {
    assert_eq!(role_name(ChatRole::System), "system");
    assert_eq!(role_name(ChatRole::User), "user");
    assert_eq!(role_name(ChatRole::Assistant), "assistant");
    let turns = messages_to_turns(&[
        ChatMessage {
            role: ChatRole::System,
            content: "s".into(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "u".into(),
        },
    ]);
    assert_eq!(turns[0].role, "system");
    assert_eq!(turns[1].role, "user");
}

#[test]
fn map_complete_result_ok_and_err() {
    let (ok, meta) = map_complete_result(Ok("hi".into()));
    assert_eq!(ok.expect("ok").content, "hi");
    assert_eq!(meta.status, Some(200));
    let (err, meta) = map_complete_result(Err("boom".into()));
    assert!(err.is_err());
    assert_eq!(meta.status, Some(500));
}

#[test]
fn scripted_local_engine_constructors_are_named_for_kiss() {
    let _ = (
        stringify!(scripted_ok),
        stringify!(scripted_err),
        stringify!(LocalEngineInner),
        stringify!(Scripted),
    );
    let ok = LocalCompletionEngine::scripted_ok("slug", "content");
    assert_eq!(ok.model_slug, "slug");
    let err = LocalCompletionEngine::scripted_err("slug", "boom");
    assert_eq!(err.model_slug, "slug");
}

#[allow(unsafe_code)]
fn restore_env_after(key: &str, value: Option<&str>, body: impl FnOnce()) {
    unsafe {
        let saved = std::env::var_os(key);
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
        body();
        match saved {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
}

#[test]
fn local_max_tokens_from_env_defaults_and_overrides() {
    let _ = super::local_max_tokens_from_env;
    let _ = LocalCompletionEngine::complete;
    restore_env_after("MALVIN_LOCAL_MAX_TOKENS", None, || {
        assert_eq!(local_max_tokens_from_env(), DEFAULT_MAX_TOKENS);
    });
    restore_env_after("MALVIN_LOCAL_MAX_TOKENS", Some("1234"), || {
        assert_eq!(local_max_tokens_from_env(), 1234);
    });
    restore_env_after("MALVIN_LOCAL_MAX_TOKENS", Some("not-a-number"), || {
        assert_eq!(local_max_tokens_from_env(), DEFAULT_MAX_TOKENS);
    });
}
