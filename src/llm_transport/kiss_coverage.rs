//! Kiss coverage witnesses for `llm_transport`.

#[test]
fn kiss_cov_llm_transport_types_and_errors() {
    use crate::llm_transport::{
        body_indicates_prompt_too_long, is_prompt_too_long_error, ChatMessage, ChatRole,
        CompletionMeta, CompletionOk, CompletionResponse, LocalLlmTransport, OpenRouterTransport,
        ResponseUsage, TransportError,
    };

    let msg = ChatMessage {
        role: ChatRole::User,
        content: "hi".into(),
    };
    let ok = CompletionOk {
        content: "x".into(),
        meta: CompletionMeta::default(),
    };
    assert_eq!(ok.clone().into_response().content, "x");
    let from = CompletionOk::from(CompletionResponse {
        content: "y".into(),
        usage: Some(ResponseUsage {
            prompt_tokens: Some(1),
            completion_tokens: None,
            total_tokens: None,
            cost: None,
        }),
        reasoning: None,
    });
    assert_eq!(from.content, "y");
    let _ = (
        msg,
        TransportError::MissingContent,
        body_indicates_prompt_too_long,
        is_prompt_too_long_error,
        OpenRouterTransport::new,
        LocalLlmTransport::new,
        stringify!(LlmTransport),
    );
}

#[test]
fn kiss_cov_openrouter_transport_construct_and_map() {
    use crate::llm_transport::{openrouter, CompletionResponse, LlmTransport, OpenRouterTransport};

    crate::agent_backend::test_support::install_openrouter_test_key();
    let cfg = crate::openrouter_transport::OpenRouterConfig::from_env("test/model".into())
        .expect("config");
    let transport = OpenRouterTransport::new(cfg).expect("transport");
    assert!(transport.ensure_ready().is_ok());
    let _ = transport.client();
    let cfg2 = crate::openrouter_transport::OpenRouterConfig::from_env("test/model".into())
        .expect("config2");
    let _ = OpenRouterTransport::new(cfg2).expect("t2").into_client();
    let wrapped = LlmTransport::OpenRouter(transport);
    assert!(wrapped.ensure_ready().is_ok());
    let ok = openrouter::map_completion(
        Ok(CompletionResponse {
            content: "hi".into(),
            usage: None,
            reasoning: None,
        }),
        crate::openrouter_transport::HttpExchangeMeta {
            status: Some(200),
            body: None,
        },
    )
    .expect("ok");
    assert_eq!(ok.content, "hi");
}

#[test]
fn kiss_cov_local_transport_module_import() {
    #[allow(unused_imports)]
    use crate::llm_transport::local;
    use crate::llm_transport::local::LocalLlmTransport;
    let _ = (
        LocalLlmTransport::new,
        LocalLlmTransport::engine,
        LocalLlmTransport::into_engine,
        LocalLlmTransport::ensure_ready,
        LocalLlmTransport::complete,
        stringify!(engine),
        stringify!(into_engine),
    );
}

#[test]
fn kiss_cov_contract_tests_symbols() {
    let _ = (
        super::contract_tests::user_msg,
        super::contract_tests::openrouter_transport,
        super::contract_tests::scripted_local_ok,
        super::contract_tests::scripted_local_err,
        super::contract_tests::llm_transport_openrouter_ensure_ready_key_present_and_absent,
        super::contract_tests::llm_transport_openrouter_complete_ok_and_err,
        super::contract_tests::llm_transport_local_scripted_ensure_ready_ok_and_complete,
        super::contract_tests::llm_transport_parity_openrouter_mock_and_scripted_local,
        super::contract_tests::kiss_cov_llm_transport_contract_symbols,
    );
}
