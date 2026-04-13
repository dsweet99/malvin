//! `do` subcommand: single coder ACP prompt with `header.md` + user text.

use clap::Args;

use super::WorkflowCliOptions;
use super::build_agent;
use super::emit_run_startup_sequence;
use super::shared_opts::SharedOpts;
use super::timing_merge::merge_acp_and_timing_results;
use malvin::acp::AgentClient;
use malvin::artifacts::{RunArtifacts, create_run_artifacts_from_text, resolve_user_request};
use malvin::orchestrator::workflow_context;
use malvin::output::{MALVIN_WHO, print_stdout_line};
use malvin::prompts::{PromptError, PromptStore};
use malvin::run_timing::{self, TimingPhase};

/// Arguments for [`run_do`].
#[derive(Args, Debug)]
pub struct DoArgs {
    #[command(flatten)]
    pub shared: SharedOpts,
    /// `@path` reads a file; otherwise literal user text. Stored as `_malvin/.../plan.md`.
    pub request: String,
}

/// Ensure `~/.malvin/prompts` defaults (including `header.md`) exist.
pub fn prepare_do_prompt_store() -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store
        .ensure_defaults()
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("header.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub async fn run_do(do_args: DoArgs, workflow: WorkflowCliOptions) -> Result<(), String> {
    let store = prepare_do_prompt_store()?;
    let mut client = build_agent(&do_args.shared, workflow);
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let (text, work_dir) = resolve_user_request(&do_args.request)?;
    let artifacts =
        create_run_artifacts_from_text(&text, Some(work_dir.as_path())).map_err(|e| e.to_string())?;

    emit_run_startup_sequence(&artifacts, do_args.shared.tee_startup_stdout())?;

    let combined = combine_do_acp_prompt(&store, &artifacts, &text)?;

    run_do_with_timing(&mut client, &artifacts, &combined).await?;

    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

async fn run_do_with_timing(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    combined: &str,
) -> Result<(), String> {
    let timing = client.attach_run_timing_for_session();
    let acp_result = run_do_acp(client, artifacts, combined).await;
    let timing_result = run_timing::finalize_and_emit_run_timing(&artifacts.run_dir, &timing);
    client.set_run_timing(None);
    merge_acp_and_timing_results(acp_result, timing_result)
}

/// Build the coder ACP prompt: expanded `header.md`, then the resolved request text.
pub fn combine_do_acp_prompt(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
) -> Result<String, String> {
    let context = workflow_context(artifacts);
    let header_body = store
        .render_prompt_only("header.md", &context)
        .map_err(|e: PromptError| e.0)?;
    Ok(format!(
        "{}\n\n{}",
        header_body.trim_end(),
        text.trim_end()
    ))
}

async fn run_do_acp(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    combined: &str,
) -> Result<(), String> {
    client
        .begin_coder_session(&artifacts.work_dir)
        .await
        .map_err(|e| e.to_string())?;
    let log = artifacts.log_path("do");
    let run_res = client
        .run_coder_prompt(
            combined,
            &log,
            "do",
            Some(TimingPhase::Implement),
        )
        .await
        .map_err(|e| e.to_string());
    let end_res = client.end_coder_session().await.map_err(|e| e.to_string());
    match (run_res, end_res) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), _) | (Ok(()), Err(e)) => Err(e),
    }
}

#[test]
fn stringify_do_flow_helpers() {
    let _ = stringify!(crate::cli::do_flow::prepare_do_prompt_store);
    let _ = stringify!(crate::cli::do_flow::run_do);
    let _ = stringify!(crate::cli::do_flow::DoArgs);
    let _ = stringify!(crate::cli::do_flow::combine_do_acp_prompt);
}

#[cfg(test)]
mod do_tests {
    use clap::Parser;

    use malvin::artifacts::RunArtifacts;
    use malvin::prompts::PromptStore;

    use super::combine_do_acp_prompt;

    #[test]
    fn combine_do_acp_prompt_joins_rendered_header_and_request() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let prompt_root = tmp.path().join("prompts");
        std::fs::create_dir_all(&prompt_root).expect("mkdir");
        std::fs::write(prompt_root.join("header.md"), "OPEN\n").expect("header");
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
        let out = combine_do_acp_prompt(&store, &artifacts, "USER_TEXT").expect("combine");
        assert!(
            out.starts_with("OPEN"),
            "expected header first; got {out:?}"
        );
        assert!(
            out.contains("USER_TEXT"),
            "expected request body; got {out:?}"
        );
    }

    #[test]
    fn cli_accepts_do_and_passes_request() {
        use crate::cli::Cli;
        use crate::cli::Commands;

        let cli = Cli::try_parse_from(["malvin", "do", "fix the bug"]).expect("parse");
        match cli.command {
            Commands::Do(d) => assert_eq!(d.request, "fix the bug"),
            _ => panic!("expected Do subcommand"),
        }
    }
}
