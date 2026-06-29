use clap::Parser;
use std::collections::HashMap;

use crate::artifacts::RunArtifacts;
use crate::do_flow::do_flow_prompt::{
    build_do_coder_run, build_do_coder_run_with_store, combine_do_acp_prompt_header_and_user,
    combine_do_prompt_file_and_user, combine_do_raw_header_and_user, prepare_do_prompt_store,
};
use crate::prompts::{DO_HEADER_MD, HEADER_MD, PromptStore};
use crate::prompt_stratification::WorkflowRenderContext;

fn do_flow_test_artifacts(tmp: &tempfile::TempDir) -> RunArtifacts {
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .status()
        .expect("git init");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "ignored").expect("plan");
    let run_dir = tmp.path().join(".malvin/logs").join("r");
    std::fs::create_dir_all(&run_dir).expect("run");
    RunArtifacts {
        run_dir,
        plan_path: plan,
        work_dir: tmp.path().to_path_buf(),
    }
}

fn assert_header_user_join(combined: &str, header: &str, user: &str) {
    assert_eq!(combined, format!("{header}\n\n{user}"));
    assert_eq!(combined.split("\n\n").count(), 2);
    assert_eq!(combined.matches(header).count(), 1);
    assert_eq!(combined.matches(user).count(), 1);
}

fn assert_do_triple_join(combined: &str, coding_header: &str, do_header: &str, user: &str) {
    assert_eq!(
        combined,
        format!("{coding_header}\n\n{do_header}\n\n{user}")
    );
    assert_eq!(combined.split("\n\n").count(), 3);
    assert!(combined.contains(coding_header));
    assert!(combined.contains(do_header));
    assert!(combined.ends_with(user));
    assert_eq!(combined.matches(user).count(), 1);
}

fn mock_do_prompt_store(tmp: &tempfile::TempDir) -> PromptStore {
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir");
    std::fs::write(prompt_root.join(HEADER_MD), "CODING_HDR\n").expect("header");
    std::fs::write(prompt_root.join(DO_HEADER_MD), "DO_HDR\n").expect("do_header");
    std::fs::write(prompt_root.join("kpop_common.md"), "").expect("kpop_common");
    PromptStore::with_root(prompt_root)
}

#[test]
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

#[test]
fn prepare_do_prompt_store_loads_default_templates() {
    let store = prepare_do_prompt_store().expect("store");
    assert!(store.validate_exists(HEADER_MD).is_ok());
    assert!(store.validate_exists(DO_HEADER_MD).is_ok());
}

#[test]
fn build_do_coder_run_combines_both_headers_and_user() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = do_flow_test_artifacts(&tmp);
    let store = mock_do_prompt_store(&tmp);
    let run = build_do_coder_run_with_store(&store, &artifacts, "USER_TOKEN\n\n").expect("run");
    assert_do_triple_join(&run.combined, "CODING_HDR", "DO_HDR", "USER_TOKEN");
    let (trace_header, trace_user) = &run.header_user_for_trace;
    assert_header_user_join(trace_header, "CODING_HDR", "DO_HDR");
    assert_eq!(trace_user, "USER_TOKEN");
}

#[test]
fn build_do_coder_run_default_store_produces_dual_headers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = do_flow_test_artifacts(&tmp);
    let run = build_do_coder_run(&artifacts, "USER_TOKEN").expect("run");
    assert!(run.combined.contains("Know thyself"));
    assert!(run.combined.contains("malvin do"));
    assert!(
        run.combined.contains("Context Prep"),
        "do prompt must include standard header content"
    );
    assert!(
        run.combined.contains("User:"),
        "do prompt must render current_state from workflow context"
    );
    assert_eq!(run.combined.matches("USER_TOKEN").count(), 1);
}

#[test]
fn combine_do_acp_prompt_joins_rendered_header_and_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let store = mock_do_prompt_store(&tmp);
    let artifacts = do_flow_test_artifacts(&tmp);
    let (combined, header, user) =
        combine_do_acp_prompt_header_and_user(&store, &artifacts, "USER_TOKEN").expect("combine");
    assert_eq!(header, "CODING_HDR");
    assert_eq!(user, "USER_TOKEN");
    assert_header_user_join(&combined, "CODING_HDR", "USER_TOKEN");
}

#[test]
fn combine_do_raw_header_and_user_joins_rendered_do_header_and_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir");
    std::fs::write(prompt_root.join(DO_HEADER_MD), "DO_TOKEN\n").expect("do_header");
    let artifacts = do_flow_test_artifacts(&tmp);
    let store = PromptStore::with_root(prompt_root);
    let (combined, header, user) =
        combine_do_raw_header_and_user(&store, &artifacts, "USER_RAW_TOKEN\n\n").expect("combine");
    assert_eq!(header, "DO_TOKEN");
    assert_eq!(user, "USER_RAW_TOKEN");
    assert_header_user_join(&combined, "DO_TOKEN", "USER_RAW_TOKEN");
}

#[test]
fn cli_accepts_do_and_passes_request() {
    use crate::cli::Cli;
    use crate::cli::Commands;

    let cli = Cli::try_parse_from(["malvin", "do", "fix the bug"]).expect("parse");
    match cli.command {
        Some(Commands::Do(d)) => {
            assert_eq!(d.request.as_deref(), Some("fix the bug"));
            assert!(!d.repo_gates);
            assert!(!d.thoughts);
        }
        _ => panic!("expected Do subcommand"),
    }
}

#[test]
fn cli_accepts_do_repo_gates() {
    use crate::cli::Cli;
    use crate::cli::Commands;

    let cli = Cli::try_parse_from(["malvin", "do", "--repo-gates", "y"]).expect("parse");
    match cli.command {
        Some(Commands::Do(d)) => {
            assert!(d.repo_gates);
            assert_eq!(d.request.as_deref(), Some("y"));
            assert!(!d.thoughts);
        }
        _ => panic!("expected Do subcommand"),
    }
}

#[test]
fn cli_accepts_do_thoughts() {
    use crate::cli::Cli;
    use crate::cli::Commands;

    let cli = Cli::try_parse_from(["malvin", "do", "--thoughts", "z"]).expect("parse");
    match cli.command {
        Some(Commands::Do(d)) => {
            assert!(d.thoughts);
            assert_eq!(d.request.as_deref(), Some("z"));
        }
        _ => panic!("expected Do subcommand"),
    }
}

#[test]
fn cli_accepts_all_shared_flags_before_subcommand() {
    use crate::cli::Cli;
    use crate::cli::Commands;

    let cli = Cli::try_parse_from([
        "malvin",
        "--model",
        "composer-2",
        "--no-force",
        "--no-tee",
        "do",
        "z",
    ])
    .expect("parse");
    assert_eq!(cli.shared.model, "composer-2");
    assert!(cli.shared.no_tee);
    assert!(cli.shared.no_force);
    match cli.command {
        Some(Commands::Do(d)) => assert_eq!(d.request.as_deref(), Some("z")),
        _ => panic!("expected Do subcommand"),
    }
}

#[test]
fn cli_accepts_max_acp_retries_global_flag() {
    use crate::cli::Cli;
    use crate::config::DEFAULT_MAX_ACP_RETRIES;

    let cli = Cli::try_parse_from(["malvin", "do", "task"]).expect("parse");
    assert_eq!(cli.shared.max_acp_retries, DEFAULT_MAX_ACP_RETRIES);

    let cli = Cli::try_parse_from(["malvin", "--max-acp-retries", "5", "do", "task"]).expect("parse");
    assert_eq!(cli.shared.max_acp_retries, 5);
}

#[test]
fn cli_accepts_verbose_short_and_long_global_flags() {
    use crate::cli::Cli;
    use crate::cli::Commands;

    let cli = Cli::try_parse_from(["malvin", "-v", "do", "x"]).expect("parse");
    assert!(cli.shared.verbose);
    match cli.command.as_ref() {
        Some(Commands::Do(d)) => assert_eq!(d.request.as_deref(), Some("x")),
        _ => panic!("expected Do subcommand"),
    }

    let cli = Cli::try_parse_from(["malvin", "do", "--verbose", "y"]).expect("parse");
    assert!(cli.shared.verbose);
    match cli.command {
        Some(Commands::Do(d)) => assert_eq!(d.request.as_deref(), Some("y")),
        _ => panic!("expected Do subcommand"),
    }
}
