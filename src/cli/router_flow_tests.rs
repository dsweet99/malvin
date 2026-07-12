use clap::Parser;
use std::collections::HashMap;

use crate::flow_prompt_join_test_helpers::{
    assert_dual_workflow_header_join, assert_header_user_join, flow_test_artifacts,
    flow_test_artifacts_no_checks,
};
use crate::router_flow::router_flow_prompt::{
    build_router_b_prompt, build_router_c_prompt, build_router_coder_run, build_router_coder_run_with_store,
    combine_router_acp_prompt_header_and_user, combine_router_prompt_file_and_user,
    combine_router_raw_header_and_user, prepare_router_prompt_store,
};
use crate::prompts::{
    HEADER_MD, PromptStore, ROUTER_A_1_MD, ROUTER_A_2_MD, ROUTER_B_SIMPLE_MD, ROUTER_C_MD,
};

fn write_router_mock_prompt_files(prompt_root: &std::path::Path) {
    write_prompt(prompt_root, HEADER_MD, "CODING_HDR\n");
    write_prompt(prompt_root, ROUTER_A_1_MD, "ROUTER_HDR\n");
    write_prompt(prompt_root, ROUTER_A_2_MD, "ROUTER_A_2_HDR\n");
    write_prompt(prompt_root, ROUTER_B_SIMPLE_MD, "ROUTER_B_HDR\n");
    write_prompt(
        prompt_root,
        crate::prompts::ROUTER_B_COMPLEX_MD,
        "ROUTER_B_COMPLEX_HDR\n",
    );
    write_prompt(prompt_root, ROUTER_C_MD, "ROUTER_C_HDR\n");
    write_prompt(prompt_root, "kpop_common.md", "");
}

fn write_prompt(prompt_root: &std::path::Path, name: &str, body: &str) {
    std::fs::write(prompt_root.join(name), body).unwrap_or_else(|_| panic!("write {name}"));
}

fn mock_router_prompt_store(tmp: &tempfile::TempDir) -> PromptStore {
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir");
    write_router_mock_prompt_files(&prompt_root);
    PromptStore::with_root(prompt_root)
}

#[test]
fn combine_router_prompt_file_and_user_joins_rendered_template_and_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir");
    std::fs::write(prompt_root.join(HEADER_MD), "TMPL\n").expect("tmpl");
    let store = PromptStore::with_root(prompt_root);
    let ctx = crate::prompt_stratification::WorkflowRenderContext::from(HashMap::from([
        ("k".into(), "v".into()),
    ]));
    let (combined, header, user) =
        combine_router_prompt_file_and_user(&store, "BODY\n", HEADER_MD, &ctx).expect("combine");
    assert_eq!(header, "TMPL");
    assert_eq!(user, "BODY");
    assert_header_user_join(&combined, "TMPL", "BODY");
}

#[test]
fn prepare_router_prompt_store_loads_default_templates() {
    let store = prepare_router_prompt_store().expect("store");
    assert!(store.validate_exists(HEADER_MD).is_ok());
    assert!(store.validate_exists(ROUTER_A_1_MD).is_ok());
    assert!(store.validate_exists(ROUTER_A_2_MD).is_ok());
    assert!(store.validate_exists(ROUTER_B_SIMPLE_MD).is_ok());
    assert!(store.validate_exists(crate::prompts::ROUTER_B_COMPLEX_MD).is_ok());
    assert!(store.validate_exists(ROUTER_C_MD).is_ok());
}

#[test]
fn build_router_coder_run_succeeds_without_checks_in_non_git_workspace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts_no_checks(&tmp);
    let run = build_router_coder_run(&artifacts, "USER_TOKEN").expect("run");
    assert!(run.combined.contains("Know thyself"));
    assert!(run.combined.contains("COMPLEXITY_SCORE"));
    assert!(
        !run.combined.contains("CODING_TASK"),
        "router_a_1 must not ask for CODING_TASK"
    );
    assert!(
        run.combined.contains("Context Prep"),
        "router prompt must include standard header content"
    );
    assert!(
        !run.combined.contains("{{"),
        "router prompt must expand all template placeholders without checks"
    );
    assert_eq!(run.combined.matches("USER_TOKEN").count(), 1);
}

#[test]
fn build_router_coder_run_combines_both_headers_and_user() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = mock_router_prompt_store(&tmp);
    let run = build_router_coder_run_with_store(&store, &artifacts, "USER_TOKEN\n\n").expect("run");
    assert_dual_workflow_header_join(&run.combined, "CODING_HDR", "ROUTER_HDR", "USER_TOKEN");
    let (trace_header, trace_user) = &run.header_user_for_trace;
    assert_header_user_join(trace_header, "CODING_HDR", "ROUTER_HDR");
    assert_eq!(trace_user, "USER_TOKEN");
}

#[test]
fn build_router_coder_run_default_store_produces_dual_headers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let run = build_router_coder_run(&artifacts, "USER_TOKEN").expect("run");
    assert!(run.combined.contains("Know thyself"));
    assert!(run.combined.contains("COMPLEXITY_SCORE"));
    assert!(
        !run.combined.contains("CODING_TASK"),
        "router_a_1 must not ask for CODING_TASK"
    );
    assert!(
        run.combined.contains("Context Prep"),
        "router prompt must include standard header content"
    );
    assert!(
        run.combined.contains("User:"),
        "router prompt must render current_state from workflow context"
    );
    assert!(
        !run.combined.contains("{{"),
        "router prompt must expand all template placeholders"
    );
    assert_eq!(run.combined.matches("USER_TOKEN").count(), 1);
}

#[test]
fn build_router_coder_run_allows_user_request_with_double_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let user = "Expand {{ code_checks }} and {{ code_extra }} in templates.";
    let run = build_router_coder_run(&artifacts, user).expect("run");
    assert!(run.combined.contains(user));
}

#[test]
fn combine_router_acp_prompt_joins_rendered_header_and_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let store = mock_router_prompt_store(&tmp);
    let artifacts = flow_test_artifacts(&tmp);
    let (combined, header, user) =
        combine_router_acp_prompt_header_and_user(&store, &artifacts, "USER_TOKEN").expect("combine");
    assert_eq!(header, "CODING_HDR");
    assert_eq!(user, "USER_TOKEN");
    assert_header_user_join(&combined, "CODING_HDR", "USER_TOKEN");
}

#[test]
fn combine_router_raw_header_and_user_joins_rendered_router_header_and_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir");
    std::fs::write(prompt_root.join(ROUTER_A_1_MD), "ROUTER_TOKEN\n").expect("router_a_1");
    let artifacts = flow_test_artifacts(&tmp);
    let store = PromptStore::with_root(prompt_root);
    let (combined, header, user) =
        combine_router_raw_header_and_user(&store, &artifacts, "USER_RAW_TOKEN\n\n")
            .expect("combine");
    assert_eq!(header, "ROUTER_TOKEN");
    assert_eq!(user, "USER_RAW_TOKEN");
    assert_header_user_join(&combined, "ROUTER_TOKEN", "USER_RAW_TOKEN");
}

#[test]
fn router_coder_run_exposes_combined_and_trace_split() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let run = build_router_coder_run(&artifacts, "TRACE_USER").expect("run");
    assert!(!run.combined.is_empty());
    let (trace_header, trace_user) = run.header_user_for_trace;
    assert!(trace_header.contains("Know thyself"));
    assert_eq!(trace_user, "TRACE_USER");
}

#[cfg(test)]
#[path = "router_flow_prompt_tests.rs"]
mod router_flow_prompt_tests;

#[test]
fn build_router_b_prompt_renders_without_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_b_prompt(&store, &artifacts, ROUTER_B_SIMPLE_MD, false).expect("router_b");
    assert!(!body.contains("CONTINUE_ROUTER"));
    assert!(!body.contains("{{"));
    let router_c = build_router_c_prompt(&store, &artifacts).expect("router_c");
    assert!(router_c.contains("CONTINUE_ROUTER"));
    assert!(!router_c.contains("{{"));
}

#[test]
fn cli_accepts_default_route_request() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "route this task"]).expect("parse");
    assert!(cli.command.is_none());
    assert_eq!(cli.request.as_deref(), Some("route this task"));
}

#[test]
fn router_client_uses_kpop_style_agent_io_not_do_style() {
    use crate::agent_backend::build_agent_backend;
    use crate::cli::{SharedOpts, WorkflowCliOptions};

    let shared = SharedOpts {
        model: crate::config::DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tenacious: false,
        no_tee: true,
        no_markdown: false,
        verbose: false,
        max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
        doc: false,
        name: None,
        mini_max_bash_turns: 32,
        mini_max_http_turns: 32,
        mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_transport_retries: crate::support_paths::DEFAULT_MAX_MINI_TRANSPORT_RETRIES,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
    };
    let backend = build_agent_backend(
        &shared,
        WorkflowCliOptions { force: false },
        shared.acp_stdout_markdown_enabled(),
        "router",
    )
    .expect("backend");
    let io = match backend {
        crate::agent_backend::AgentBackend::Acp(c) => c.io,
        crate::agent_backend::AgentBackend::Mini(c) => c.io,
    };
    assert!(
        !io.raw_output,
        "bare route must use styled logging, not do-style raw_output"
    );
    assert!(io.show_thoughts_on_stdout);
    assert!(io.emit_stdout_markdown);
}
