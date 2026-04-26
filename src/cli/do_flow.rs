//! `do` subcommand: one coder ACP prompt. Default raw mode prepends `do_header.md` to the user
//! request; `--cooked` prepends `header.md` instead (and allows repo style).

use std::collections::HashMap;

use clap::Args;

use super::WorkflowCliOptions;
use super::repo_checks;
use super::repo_checks::RepoGateOutput;
use super::shared_opts::SharedOpts;
use super::timing_merge;
use malvin::acp::{AgentClient, CoderPromptOptions};
use malvin::artifacts::{
    RunArtifacts, backup_workspace_grounding_if_present, create_run_artifacts_from_text,
    resolve_user_request,
};
use malvin::orchestrator::{workflow_context, workflow_context_paths_only};
use malvin::prompts::{DO_HEADER_MD, HEADER_MD, PromptError, PromptStore};
use malvin::run_timing::TimingPhase;

struct DoCoderRun {
    combined: String,
    header_user_for_trace: (String, String),
    skip_repo_style: bool,
}

/// Arguments for [`run_do`].
#[derive(Args, Debug)]
pub struct DoArgs {
    /// Prepend `header.md` and allow optional injected repo style
    #[arg(long, default_value_t = false)]
    pub cooked: bool,
    /// Run kiss clamp + configured pre-commit hooks before the prompt (coding-style runs).
    #[arg(long, default_value_t = false)]
    pub repo_gates: bool,
    #[arg(long, default_value_t = false)]
    pub thoughts: bool,
    /// Request or `@file` → `_malvin/.../plan.md`.
    pub request: String,
}

fn prepare_do_prompt_store_validating(required_template: &str) -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(required_template)
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn prepare_do_prompt_store() -> Result<PromptStore, String> {
    prepare_do_prompt_store_validating(HEADER_MD)
}

pub fn prepare_do_raw_prompt_store() -> Result<PromptStore, String> {
    prepare_do_prompt_store_validating(DO_HEADER_MD)
}

pub async fn run_do(
    do_args: DoArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let raw_output = true;
    let skip_repo_style = !do_args.cooked;
    let mut client = AgentClient::new(
        shared.model.clone(),
        super::agent_io_options(
            shared,
            workflow,
            super::AgentStdoutTeeFlags {
                emit_stdout_markdown: false,
                raw_output,
                show_thoughts_on_stdout: do_args.thoughts,
            },
        ),
    );
    let (text, work_dir) = resolve_user_request(&do_args.request)?;
    let artifacts = create_run_artifacts_from_text(&text, Some(work_dir.as_path()))
        .map_err(|e| e.to_string())?;

    if do_args.repo_gates {
        repo_checks::run_repo_workspace_gates(&artifacts.work_dir, RepoGateOutput::Stderr)?;
    }
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let (combined, header_user) = if do_args.cooked {
        let store = prepare_do_prompt_store()?;
        let (combined, header, user) =
            combine_do_acp_prompt_header_and_user(&store, &artifacts, &text)?;
        (combined, (header, user))
    } else {
        let store = prepare_do_raw_prompt_store()?;
        let (combined, header, user) = combine_do_raw_header_and_user(&store, &artifacts, &text)?;
        (combined, (header, user))
    };

    let coder = DoCoderRun {
        combined,
        header_user_for_trace: header_user,
        skip_repo_style,
    };
    let grounding_backup = backup_workspace_grounding_if_present(&artifacts.work_dir)?;
    super::run_emit::emit_command_line(&artifacts.run_dir, false)?;
    let acp_res = run_do_acp(&mut client, &artifacts, coder).await;
    timing_merge::merge_acp_with_grounding_restore(acp_res, &artifacts.work_dir, &grounding_backup)?;
    Ok(())
}

fn combine_do_prompt_file_and_user(
    store: &PromptStore,
    text: &str,
    template_file: &str,
    context: &HashMap<String, String>,
) -> Result<(String, String, String), String> {
    let header_body = store
        .render_prompt_only(template_file, context)
        .map_err(|e: PromptError| e.0)?;
    let header = header_body.trim_end().to_string();
    let user = text.trim_end().to_string();
    let combined = format!("{header}\n\n{user}");
    Ok((combined, header, user))
}

/// Renders `header.md` once; returns combined prompt plus header and user strings for ACP trace splitting.
pub fn combine_do_acp_prompt_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
) -> Result<(String, String, String), String> {
    let context = workflow_context(artifacts, store).map_err(|e: PromptError| e.0)?;
    combine_do_prompt_file_and_user(store, text, HEADER_MD, &context)
}

/// Renders `do_header.md` once; same return shape for default raw `malvin do`.
pub fn combine_do_raw_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
) -> Result<(String, String, String), String> {
    let context = workflow_context_paths_only(artifacts);
    combine_do_prompt_file_and_user(store, text, DO_HEADER_MD, &context)
}

async fn run_do_acp(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    coder: DoCoderRun,
) -> Result<(), String> {
    client
        .begin_coder_session(&artifacts.work_dir)
        .await
        .map_err(|e| e.to_string())?;
    let timing = client.attach_run_timing_for_session();
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_implement_display_name("do");
    let log = artifacts.log_path("do");
    let (ref header, ref user) = coder.header_user_for_trace;
    let do_split = Some((header.as_str(), user.as_str()));
    let run_res = client
        .run_coder_prompt(
            &coder.combined,
            &log,
            "do",
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                skip_repo_style: coder.skip_repo_style,
                do_trace_split: do_split,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string());
    let end_res = client.end_coder_session().await.map_err(|e| e.to_string());
    let merged = if let Err(e) = run_res {
        Err(e)
    } else if let Err(e) = end_res {
        Err(e)
    } else {
        Ok(())
    };
    timing_merge::emit_run_timing_json_only_after_acp(client, &artifacts.run_dir, &timing, merged)
}

#[test]
fn stringify_do_flow_helpers() {
    let _ = stringify!(crate::cli::do_flow::prepare_do_prompt_store);
    let _ = stringify!(crate::cli::do_flow::run_do);
    let _ = stringify!(crate::cli::do_flow::DoArgs);
    let _ = stringify!(crate::cli::do_flow::combine_do_acp_prompt_header_and_user);
    let _ = stringify!(crate::cli::do_flow::combine_do_raw_header_and_user);
    let _ = stringify!(crate::cli::do_flow::prepare_do_raw_prompt_store);
    let _ = stringify!(crate::cli::do_flow::DoCoderRun);
}

#[cfg(test)]
mod do_tests {
    use clap::Parser;

    use malvin::artifacts::RunArtifacts;
    use malvin::prompts::{DO_HEADER_MD, HEADER_MD, PromptStore};

    use super::{combine_do_acp_prompt_header_and_user, combine_do_raw_header_and_user};

    #[test]
    fn combine_do_acp_prompt_joins_rendered_header_and_request() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let prompt_root = tmp.path().join("prompts");
        std::fs::create_dir_all(&prompt_root).expect("mkdir");
        std::fs::write(prompt_root.join(HEADER_MD), "HEADER_TOKEN\n").expect("header");
        std::fs::write(prompt_root.join("kpop_common.md"), "").expect("kpop_common");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "ignored").expect("plan");
        let run_dir = tmp.path().join("_malvin").join("r");
        std::fs::create_dir_all(&run_dir).expect("run");
        let artifacts = RunArtifacts {
            run_dir,
            plan_path: plan,
            work_dir: tmp.path().to_path_buf(),
        };
        let store = PromptStore::with_root(prompt_root);
        let (combined, header, user) =
            combine_do_acp_prompt_header_and_user(&store, &artifacts, "USER_TOKEN")
                .expect("combine");
        assert_eq!(header, "HEADER_TOKEN");
        assert_eq!(user, "USER_TOKEN");
        assert_eq!(combined, "HEADER_TOKEN\n\nUSER_TOKEN");
        assert_eq!(combined.split("\n\n").count(), 2);
        assert_eq!(combined.matches("HEADER_TOKEN").count(), 1);
        assert_eq!(combined.matches("USER_TOKEN").count(), 1);
    }

    #[test]
    fn combine_do_raw_header_and_user_joins_rendered_do_header_and_request() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let prompt_root = tmp.path().join("prompts");
        std::fs::create_dir_all(&prompt_root).expect("mkdir");
        std::fs::write(prompt_root.join(DO_HEADER_MD), "DO_TOKEN\n").expect("do_header");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "ignored").expect("plan");
        let run_dir = tmp.path().join("_malvin").join("r");
        std::fs::create_dir_all(&run_dir).expect("run");
        let artifacts = RunArtifacts {
            run_dir,
            plan_path: plan,
            work_dir: tmp.path().to_path_buf(),
        };
        let store = PromptStore::with_root(prompt_root);
        let (combined, header, user) =
            combine_do_raw_header_and_user(&store, &artifacts, "USER_RAW_TOKEN\n\n")
                .expect("combine");
        assert_eq!(header, "DO_TOKEN");
        assert_eq!(user, "USER_RAW_TOKEN");
        assert_eq!(combined, "DO_TOKEN\n\nUSER_RAW_TOKEN");
        assert_eq!(combined.split("\n\n").count(), 2);
        assert_eq!(combined.matches("DO_TOKEN").count(), 1);
        assert_eq!(combined.matches("USER_RAW_TOKEN").count(), 1);
    }

    #[test]
    fn cli_accepts_do_and_passes_request() {
        use crate::cli::Cli;
        use crate::cli::Commands;

        let cli = Cli::try_parse_from(["malvin", "do", "fix the bug"]).expect("parse");
        match cli.command {
            Commands::Do(d) => {
                assert_eq!(d.request, "fix the bug");
                assert!(!d.cooked);
                assert!(!d.repo_gates);
                assert!(!d.thoughts);
            }
            _ => panic!("expected Do subcommand"),
        }
    }

    #[test]
    fn cli_accepts_do_cooked() {
        use crate::cli::Cli;
        use crate::cli::Commands;

        let cli = Cli::try_parse_from(["malvin", "do", "--cooked", "x"]).expect("parse");
        match cli.command {
            Commands::Do(d) => {
                assert!(d.cooked);
                assert_eq!(d.request, "x");
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
            Commands::Do(d) => {
                assert!(d.repo_gates);
                assert_eq!(d.request, "y");
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
            Commands::Do(d) => {
                assert!(d.thoughts);
                assert_eq!(d.request, "z");
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
            Commands::Do(d) => assert_eq!(d.request, "z"),
            _ => panic!("expected Do subcommand"),
        }
    }

}
