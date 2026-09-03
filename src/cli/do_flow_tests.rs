use clap::Parser;
use std::collections::HashMap;

use crate::config::DEFAULT_CLI_MODEL;
use crate::do_flow::do_flow_prompt::{
    build_do_coder_run_with_store, combine_do_acp_prompt_header_and_user,
    combine_do_prompt_file_and_user, combine_do_raw_header_and_user, prepare_do_prompt_store,
};
use crate::flow_prompt_join_test_helpers::{
    assert_header_user_join, flow_test_artifacts,
    flow_test_artifacts_no_checks,
};
use crate::prompt_stratification::WorkflowRenderContext;
use crate::prompts::{DO_HEADER_MD, HEADER_MD, PromptStore};

fn mock_do_prompt_store(tmp: &tempfile::TempDir) -> PromptStore {
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir");
    std::fs::write(prompt_root.join(HEADER_MD), "CODING_HDR\n").expect("header");
    std::fs::write(prompt_root.join(DO_HEADER_MD), "DO_HDR\n").expect("do_header");
    PromptStore::with_root(prompt_root)
}

fn combine_do_prompt_file_and_user_joins_rendered_template_and_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir");
    std::fs::write(prompt_root.join(HEADER_MD), "TMPL\n").expect("tmpl");
    let store = PromptStore::with_root(prompt_root);
    let ctx = WorkflowRenderContext::from(HashMap::from([("k".into(), "v".into())]));
    let (combined, header, user) =
        combine_do_prompt_file_and_user(&store, "BODY\n", HEADER_MD, &ctx).expect("combine");
    assert_eq!(header, "TMPL");
    assert_eq!(user, "BODY");
    assert_header_user_join(&combined, "TMPL", "BODY");
}

fn prepare_do_prompt_store_loads_default_templates() {
    let store = prepare_do_prompt_store().expect("store");
    assert!(store.validate_exists(HEADER_MD).is_ok());
    assert!(store.validate_exists(DO_HEADER_MD).is_ok());
}

fn build_do_coder_run_succeeds_without_checks_in_non_git_workspace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts_no_checks(&tmp);
    let store = prepare_do_prompt_store().expect("store");
    let run = build_do_coder_run_with_store(
        &store,
        &artifacts,
        "USER_TOKEN",
        crate::workflow_context::PromptModelOpts::new(DEFAULT_CLI_MODEL, false),
    );
    assert_eq!(run.combined, "USER_TOKEN");
    assert!(
        !run.combined.contains("Know thyself"),
        "headers are sent at spawn, not in the do work prompt"
    );
    assert!(
        !run.combined.contains("malvin --do"),
        "do_header.md is sent at spawn, not in the do work prompt"
    );
}

fn build_do_coder_run_work_prompt_is_user_only() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = mock_do_prompt_store(&tmp);
    let run = build_do_coder_run_with_store(
        &store,
        &artifacts,
        "USER_TOKEN\n\n",
        crate::workflow_context::PromptModelOpts::new(DEFAULT_CLI_MODEL, false),
    );
    assert_eq!(run.combined, "USER_TOKEN");
    let (trace_header, trace_user) = &run.header_user_for_trace;
    assert!(trace_header.is_empty());
    assert_eq!(trace_user, "USER_TOKEN");
    assert!(
        !run.combined.contains("CODING_HDR") && !run.combined.contains("DO_HDR"),
        "spawn binds header.md + do_header.md; work turn is user only"
    );
}

fn build_do_coder_run_default_store_work_prompt_is_user() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_do_prompt_store().expect("store");
    let run = build_do_coder_run_with_store(
        &store,
        &artifacts,
        "USER_TOKEN",
        crate::workflow_context::PromptModelOpts::new(DEFAULT_CLI_MODEL, false),
    );
    assert_eq!(run.combined, "USER_TOKEN");
}

fn combine_do_acp_prompt_joins_rendered_header_and_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let store = mock_do_prompt_store(&tmp);
    let artifacts = flow_test_artifacts(&tmp);
    let (combined, header, user) = combine_do_acp_prompt_header_and_user(
        &store,
        &artifacts,
        "USER_TOKEN",
        crate::workflow_context::PromptModelOpts::new(DEFAULT_CLI_MODEL, false),
    )
    .expect("combine");
    assert_eq!(header, "CODING_HDR");
    assert_eq!(user, "USER_TOKEN");
    assert_header_user_join(&combined, "CODING_HDR", "USER_TOKEN");
}

fn combine_do_raw_header_and_user_joins_rendered_do_header_and_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir");
    std::fs::write(prompt_root.join(DO_HEADER_MD), "DO_TOKEN\n").expect("do_header");
    let artifacts = flow_test_artifacts(&tmp);
    let store = PromptStore::with_root(prompt_root);
    let (combined, header, user) = combine_do_raw_header_and_user(
        &store,
        &artifacts,
        "USER_RAW_TOKEN\n\n",
        crate::workflow_context::PromptModelOpts::new(DEFAULT_CLI_MODEL, false),
    )
    .expect("combine");
    assert_eq!(header, "DO_TOKEN");
    assert_eq!(user, "USER_RAW_TOKEN");
    assert_header_user_join(&combined, "DO_TOKEN", "USER_RAW_TOKEN");
}

fn cli_accepts_do_and_passes_request() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "--do", "fix the bug"]).expect("parse");
    assert!(cli.do_workflow);
    assert_eq!(cli.request.as_deref(), Some("fix the bug"));
    assert!(cli.command.is_none());
}

fn cli_rejects_do_thoughts_flag() {
    use crate::cli::Cli;

    let err = Cli::try_parse_from(["malvin", "--do", "--thoughts", "z"]).expect_err("parse");
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected argument") || msg.contains("--thoughts"),
        "expected --thoughts rejected; got {msg}"
    );
}

fn cli_accepts_all_shared_flags_before_subcommand() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from([
        "malvin",
        "--model",
        "cursor:composer-2",
        "--no-force",
        "--do",
        "z",
    ])
    .expect("parse");
    assert_eq!(cli.shared.model.canonical(), "cursor:composer-2");
    assert!(cli.shared.no_force);
    assert!(cli.do_workflow);
    assert_eq!(cli.request.as_deref(), Some("z"));
}

fn cli_accepts_max_acp_retries_global_flag() {
    use crate::cli::Cli;
    use crate::config::DEFAULT_MAX_ACP_RETRIES;

    let cli = Cli::try_parse_from(["malvin", "--do", "task"]).expect("parse");
    assert_eq!(cli.shared.max_acp_retries, DEFAULT_MAX_ACP_RETRIES);

    let cli =
        Cli::try_parse_from(["malvin", "--max-acp-retries", "5", "--do", "task"]).expect("parse");
    assert_eq!(cli.shared.max_acp_retries, 5);
}

fn cli_accepts_verbose_short_and_long_global_flags() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "-v", "--do", "x"]).expect("parse");
    assert!(cli.shared.verbose);
    assert!(cli.do_workflow);
    assert_eq!(cli.request.as_deref(), Some("x"));

    let cli = Cli::try_parse_from(["malvin", "--do", "--verbose", "y"]).expect("parse");
    assert!(cli.shared.verbose);
    assert!(cli.do_workflow);
    assert_eq!(cli.request.as_deref(), Some("y"));
}

#[test]
fn kiss_bundled_cli_do_flow_tests() {
    combine_do_prompt_file_and_user_joins_rendered_template_and_request();
    prepare_do_prompt_store_loads_default_templates();
    build_do_coder_run_succeeds_without_checks_in_non_git_workspace();
    build_do_coder_run_work_prompt_is_user_only();
    build_do_coder_run_default_store_work_prompt_is_user();
    combine_do_acp_prompt_joins_rendered_header_and_request();
    combine_do_raw_header_and_user_joins_rendered_do_header_and_request();
    cli_accepts_do_and_passes_request();
    cli_rejects_do_thoughts_flag();
    cli_accepts_all_shared_flags_before_subcommand();
    cli_accepts_max_acp_retries_global_flag();
    cli_accepts_verbose_short_and_long_global_flags();
}
