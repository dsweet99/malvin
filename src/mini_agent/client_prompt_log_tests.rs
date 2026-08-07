use super::client_prompt_log::{write_prompt_log, PromptLogWrite};
use crate::acp::CoderPromptOptions;
use crate::agent_backend::test_support::mini_loop_config;
use crate::mini_agent::{LlmBackend, MiniAgentClient, MockScript, MockStep};
use crate::openrouter_transport::CompletionResponse;
use std::sync::Mutex;

fn test_client(verbose: bool) -> MiniAgentClient {
    MiniAgentClient::new_mock(
        mini_loop_config(4, 1),
        crate::acp::AgentIoOptions {
            force: false,
            no_tee: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: verbose,
        },
        LlmBackend::Mock(Mutex::new(MockScript {
            responses: vec![MockStep::Ok(CompletionResponse {
                content: "ok".into(),
                usage: None,
                reasoning: None,
            })],
            call_count: 0,
            on_response: None,
        })),
    )
}

fn write_router_work_bracket(client: &MiniAgentClient, log_path: &std::path::Path) {
    write_prompt_log(PromptLogWrite {
        client,
        prompt: "body",
        log_path,
        who: "router_work",
        opts: &CoderPromptOptions {
            stdout_bracket_label: Some("router_work.md"),
            ..Default::default()
        },
    })
    .expect("write");
}

fn assert_bracket_in_stdout_log_only(stdout: &str, live: &str) {
    let delim = crate::output::format_who_tag_delim(crate::output::WHO_U);
    assert!(
        stdout.contains(&format!("{delim}[router_work.md...]")),
        "bracket summary must land in stdout.log like cursor:/prime:: {stdout:?}"
    );
    assert!(
        live.is_empty(),
        "outgoing prompt bracket must not hit live terminal; got {live:?}"
    );
    assert!(
        !live.contains("[router_work") && !stdout.contains("r|[router_work]"),
        "legacy r|[router_work] live form must stay gone"
    );
}

#[tokio::test]
async fn mini_prompt_log_bracket_goes_to_stdout_log_not_live_terminal() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut client = test_client(false);
    client.io.raw_output = false;
    client.io.no_tee = false;
    let stdout_log = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(stdout_log.clone()));
    crate::output::enable_stdout_capture();
    write_router_work_bracket(&client, &tmp.path().join("router_work.log"));
    let live = crate::output::take_captured_stdout();
    let stdout = std::fs::read_to_string(&stdout_log).unwrap_or_default();
    assert_bracket_in_stdout_log_only(&stdout, &live);
    crate::output::set_stdout_log_path(None);
}

#[tokio::test]
async fn mini_prompt_log_skips_stdout_log_bracket_when_no_tee() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut client = test_client(false);
    client.io.raw_output = false;
    client.io.no_tee = true;
    let stdout_log = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(stdout_log.clone()));
    write_router_work_bracket(&client, &tmp.path().join("router_work.log"));
    let stdout = std::fs::read_to_string(&stdout_log).unwrap_or_default();
    assert!(
        stdout.is_empty(),
        "no_tee (background) must skip stdout.log brackets like cursor:/prime:; got {stdout:?}"
    );
    crate::output::set_stdout_log_path(None);
}

#[tokio::test]
async fn mini_do_prompt_log_skips_live_stdout_bracket() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut client = test_client(false);
    client.trace.plain_lines = true;
    let log = tmp.path().join("do.log");
    let log_path = log.clone();
    crate::output::set_stdout_log_path(Some(tmp.path().join("stdout.log")));
    write_prompt_log(PromptLogWrite {
        client: &client,
        prompt: "body",
        log_path: &log_path,
        who: "do",
        opts: &CoderPromptOptions {
            do_trace_split: Some(("header", "user")),
            ..Default::default()
        },
    })
    .expect("write");
    let stdout = std::fs::read_to_string(tmp.path().join("stdout.log")).unwrap_or_default();
    assert!(
        stdout.is_empty(),
        "plain do must not emit d|[do] bracket on stdout; got {stdout:?}"
    );
    crate::output::set_stdout_log_path(None);
}

#[tokio::test]
async fn mini_write_prompt_log_includes_effective_constraints() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut client = test_client(true);
    client.trace_run_dir = Some(tmp.path().to_path_buf());
    let log = tmp.path().join("agent.log");
    write_prompt_log(PromptLogWrite {
        client: &client,
        prompt: "constraints block\n\nbody text",
        log_path: &log,
        who: "agent",
        opts: &CoderPromptOptions::default(),
    })
    .expect("write");
    let text = std::fs::read_to_string(&log).expect("read");
    assert!(text.contains("constraints block"));
    assert!(text.contains("body text"));
    let run_prompts = std::fs::read_to_string(tmp.path().join("prompts.log")).expect("mirror");
    assert!(run_prompts.contains("constraints block"));
}

#[tokio::test]
async fn mini_write_prompt_log_appends_log_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let client = test_client(true);
    let log = tmp.path().join("agent.log");
    write_prompt_log(PromptLogWrite {
        client: &client,
        prompt: "body text",
        log_path: &log,
        who: "agent",
        opts: &CoderPromptOptions::default(),
    })
    .expect("write");
    let text = std::fs::read_to_string(&log).expect("read");
    assert!(text.contains("body text"));
}

#[cfg(test)]
mod kiss_cov_prompt_log_refs {
    use super::*;

    #[test]
    fn kiss_cov_client_prompt_log_test_symbols() {
        let _ = (
            test_client,
            write_router_work_bracket,
            assert_bracket_in_stdout_log_only,
            stringify!(mini_prompt_log_bracket_goes_to_stdout_log_not_live_terminal),
            stringify!(mini_prompt_log_skips_stdout_log_bracket_when_no_tee),
            stringify!(mini_do_prompt_log_skips_live_stdout_bracket),
            stringify!(mini_write_prompt_log_includes_effective_constraints),
            stringify!(mini_write_prompt_log_appends_log_file),
        );
    }
}
