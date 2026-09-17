use super::super::cache_clock::unix_now_secs;
use super::refresh_pi_provider_caches_if_stale;

fn cursor_provider_is_excluded_from_live_fetch() {
    crate::acp::with_env("CURSOR_API_KEY", Some("test-key"), || {
        crate::test_utils::with_isolated_home(|_| {
            let live = refresh_pi_provider_caches_if_stale(true);
            assert!(
                !live.contains_key("cursor"),
                "cursor must not use Pi live fetch: {live:?}"
            );
        });
    });
}

fn live_fetch_skips_providers_without_api_key() {
    crate::test_utils::with_isolated_home(|_| {
        crate::acp::with_env("OPENAI_API_KEY", None, || {
            if !super::super::auth::is_provider_authenticated("openai") {
                return;
            }
            let live = refresh_pi_provider_caches_if_stale(true);
            assert!(
                !live.contains_key("openai"),
                "empty api key must skip live fetch: {live:?}"
            );
        });
    });
}

fn now_secs_returns_positive_epoch() {
    assert!(unix_now_secs() > 0);
}

fn resolve_provider_api_key_returns_empty_when_unconfigured() {
    crate::test_utils::with_isolated_home(|_| {
        crate::acp::with_env("OPENAI_API_KEY", None, || {
            if super::super::auth::is_provider_authenticated("openai") {
                return;
            }
            assert!(super::resolve_provider_api_key("openai").is_empty());
        });
    });
}
fn resolve_provider_api_key_reads_env_when_auth_missing() {
    crate::test_utils::with_isolated_home(|_| {
        crate::acp::with_env("OPENROUTER_API_KEY", Some("env-or-key"), || {
            assert_eq!(
                super::resolve_provider_api_key("openrouter").as_str(),
                "env-or-key"
            );
        });
    });
}

fn resolve_provider_api_key_uses_dummy_for_keyless_local() {
    crate::test_utils::with_isolated_home(|_| {
        assert_eq!(
            super::resolve_provider_api_key("ollama").as_str(),
            super::super::local_context::KEYLESS_LOCAL_API_KEY
        );
        assert_eq!(
            super::resolve_provider_api_key("llamacpp").as_str(),
            super::super::local_context::KEYLESS_LOCAL_API_KEY
        );
    });
}

fn parse_http_authority_host_port_covers_common_forms() {
    assert_eq!(
        super::super::local_endpoint::parse_http_authority_host_port("http://127.0.0.1:8080/v1"),
        Some(("127.0.0.1".into(), 8080))
    );
    assert_eq!(
        super::super::local_endpoint::parse_http_authority_host_port("http://localhost/v1"),
        Some(("localhost".into(), 80))
    );
    assert_eq!(
        super::super::local_endpoint::parse_http_authority_host_port("https://example.com/v1"),
        Some(("example.com".into(), 443))
    );
    assert_eq!(
        super::super::local_endpoint::parse_http_authority_host_port("http://[::1]:1234/v1"),
        Some(("::1".into(), 1234))
    );
    assert!(super::super::local_endpoint::parse_http_authority_host_port("not-a-url").is_none());
}

fn http_base_url_is_listening_detects_open_and_closed_ports() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let open = format!("http://127.0.0.1:{port}/v1");
    assert!(super::super::local_endpoint::http_base_url_is_listening(
        &open
    ));
    drop(listener);
    let closed = format!("http://127.0.0.1:{port}/v1");
    assert!(!super::super::local_endpoint::http_base_url_is_listening(
        &closed
    ));
}

fn refresh_skips_unreachable_keyless_local_providers() {
    crate::test_utils::with_isolated_home(|_| {
        let live = refresh_pi_provider_caches_if_stale(true);
        for provider in ["llamacpp", "mistralrs"] {
            if !super::super::local_endpoint::keyless_local_provider_is_listening(provider) {
                assert_eq!(
                    live.get(provider).map(std::vec::Vec::as_slice),
                    Some(&[][..]),
                    "unreachable {provider} must contribute empty live (suppress ghosts): {live:?}"
                );
            }
        }
    });
}

fn openai_compat_models_url_skips_non_openai_endpoints() {
    let google = pi::provider_metadata::provider_routing_defaults("google").expect("google");
    assert!(super::openai_compat_models_url(&google).is_none());
    let openrouter =
        pi::provider_metadata::provider_routing_defaults("openrouter").expect("openrouter");
    assert_eq!(
        super::openai_compat_models_url(&openrouter).as_deref(),
        Some("https://openrouter.ai/api/v1/models")
    );
}

#[test]
fn kiss_bundled_pi_sdk_models_refresh_tests() {
    cursor_provider_is_excluded_from_live_fetch();
    live_fetch_skips_providers_without_api_key();
    now_secs_returns_positive_epoch();
    resolve_provider_api_key_returns_empty_when_unconfigured();
    resolve_provider_api_key_reads_env_when_auth_missing();
    resolve_provider_api_key_uses_dummy_for_keyless_local();
    parse_http_authority_host_port_covers_common_forms();
    http_base_url_is_listening_detects_open_and_closed_ports();
    refresh_skips_unreachable_keyless_local_providers();
    openai_compat_models_url_skips_non_openai_endpoints();
}
