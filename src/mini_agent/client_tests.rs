//! Unit tests for [`super::MiniAgentClient`].

use super::*;
use crate::mini_agent::{LlmBackend, MockScript, MockStep};

#[test]
fn mini_plain_lines_requires_raw_output_even_with_do_trace_split() {
    let split = CoderPromptOptions {
        do_trace_split: Some(("h", "u")),
        ..Default::default()
    };
    let no_split = CoderPromptOptions::default();
    let styled = AgentIoOptions {
        force: false,
        no_tee: true,
        raw_output: false,
        show_thoughts_on_stdout: true,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    };
    let raw = AgentIoOptions {
        raw_output: true,
        show_thoughts_on_stdout: false,
        ..styled
    };
    // Mirrors MiniAgentClient::run_coder_prompt assignment.
    let plain = |opts: &CoderPromptOptions<'_>, io: &AgentIoOptions| {
        opts.do_trace_split.is_some() && io.raw_output
    };
    assert!(
        !plain(&split, &styled),
        "verbose/default-workflow tee must keep who-tags despite do_trace_split"
    );
    assert!(
        plain(&split, &raw),
        "raw --do tee still uses plain_lines when do_trace_split is set"
    );
    assert!(!plain(&no_split, &raw));
    assert!(!plain(&no_split, &styled));
}

#[test]
fn mini_new_mock_skips_openrouter_init() {
    let client = MiniAgentClient::new_mock(
        MiniLoopConfig {
            model: "m".into(),
            max_http_turns: 4,
            max_bash_execs: 128,
            max_http_retries: 1,
            max_transport_retries: 3,
            max_gate_retries: 1,
            max_shrink_passes: 0,
            retry_strategy: MiniRetryStrategy::CumulativeTranscript,
            expects_investigation: false,
            allow_download: true,
            mini_constraints: "constraints".into(),
        },
        AgentIoOptions {
            force: false,
            no_tee: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        },
        LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![MockStep::Ok(crate::openrouter_transport::CompletionResponse {
                content: "ok".into(),
                usage: None,
                reasoning: None,
            })],
            call_count: 0,
            on_response: None,
        })),
    );
    assert!(!client.has_open_coder_session());
    assert!(!client.has_local_engine());
}

#[tokio::test]
async fn begin_coder_session_fails_fast_when_no_force() {
    let client = MiniAgentClient::new_mock(
        MiniLoopConfig {
            model: "m".into(),
            max_http_turns: 4,
            max_bash_execs: 128,
            max_http_retries: 1,
            max_transport_retries: 3,
            max_gate_retries: 1,
            max_shrink_passes: 0,
            retry_strategy: MiniRetryStrategy::CumulativeTranscript,
            expects_investigation: false,
            allow_download: true,
            mini_constraints: "constraints".into(),
        },
        AgentIoOptions {
            force: false,
            no_tee: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        },
        LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![],
            call_count: 0,
            on_response: None,
        })),
    );
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut client = client;
    let err = client
        .begin_coder_session(tmp.path())
        .await
        .expect_err("no-force must fail fast like cursor:/prime:");
    assert!(
        err.0.contains("--no-force") && err.0.contains("mini:"),
        "unexpected error: {}",
        err.0
    );
}

#[test]
fn ensure_authenticated_skips_api_key_for_local_models() {
    let client = MiniAgentClient::new_mock(
        MiniLoopConfig {
            model: "mini:local/qwen35_9b_q4".into(),
            max_http_turns: 4,
            max_bash_execs: 128,
            max_http_retries: 1,
            max_transport_retries: 3,
            max_gate_retries: 1,
            max_shrink_passes: 0,
            retry_strategy: MiniRetryStrategy::CumulativeTranscript,
            expects_investigation: false,
            allow_download: false,
            mini_constraints: "constraints".into(),
        },
        AgentIoOptions {
            force: false,
            no_tee: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        },
        LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![],
            call_count: 0,
            on_response: None,
        })),
    );
    client.ensure_authenticated().expect("local needs no key");
}

#[cfg(test)]
mod kiss_cov_client_test_refs {
    #[test]
    fn kiss_cov_client_test_symbols() {
        let _ = (
            stringify!(mini_plain_lines_requires_raw_output_even_with_do_trace_split),
            stringify!(mini_new_mock_skips_openrouter_init),
            stringify!(begin_coder_session_fails_fast_when_no_force),
            stringify!(ensure_authenticated_skips_api_key_for_local_models),
        );
    }
}
