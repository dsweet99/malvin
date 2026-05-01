use std::path::Path;

use malvin::acp::{AgentClient, CoderPromptOptions};
use malvin::artifacts::{
    RunArtifacts, backup_workspace_grounding_if_present, create_run_artifacts_from_text,
    restore_workspace_kissconfig,
};
use malvin::orchestrator::workflow_context;
use malvin::output::{MALVIN_WHO, print_stdout_line};
use malvin::prompts::{PromptError, PromptStore};
use malvin::run_timing::TimingPhase;

use super::repo_checks;
use super::{SharedOpts, WorkflowCliOptions, run_emit, timing_merge};

const GROUND_REQUEST: &str = "ground";
const GROUND_MAX_LOOPS: usize = 5;
struct GroundSession {
    artifacts: RunArtifacts,
    grounding_backup: malvin::artifacts::GroundingBackup,
    write_prompt: String,
    improve_prompt: String,
    check_sync_prompt: String,
    grounding_exists: bool,
}

fn prepare_ground_prompt_store() -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("write_grounding.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("improve_grounding.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("check_sync.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

fn build_write_grounding_prompt(
    store: &PromptStore,
    artifacts: &RunArtifacts,
) -> Result<String, String> {
    let context =
        workflow_context(artifacts, store, GROUND_REQUEST).map_err(|e: PromptError| e.0)?;
    store
        .render("write_grounding.md", &context)
        .map_err(|e: PromptError| e.0)
}

fn build_improve_grounding_prompt(
    store: &PromptStore,
    artifacts: &RunArtifacts,
) -> Result<String, String> {
    let context =
        workflow_context(artifacts, store, GROUND_REQUEST).map_err(|e: PromptError| e.0)?;
    store
        .render("improve_grounding.md", &context)
        .map_err(|e: PromptError| e.0)
}

fn build_check_sync_prompt(
    store: &PromptStore,
    artifacts: &RunArtifacts,
) -> Result<String, String> {
    let context =
        workflow_context(artifacts, store, GROUND_REQUEST).map_err(|e: PromptError| e.0)?;
    store
        .render("check_sync.md", &context)
        .map_err(|e: PromptError| e.0)
}

fn clear_review_file(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn sync_review_file_for_attempt(
    artifact_review_path: &Path,
    workspace_review_path: &Path,
) -> Result<Option<String>, String> {
    if workspace_review_path.exists() {
        let workspace_text = std::fs::read_to_string(workspace_review_path)
            .map_err(|e| format!("failed to read workspace review file: {e}"))?;
        if !workspace_text.trim().is_empty() {
            std::fs::write(artifact_review_path, &workspace_text)
                .map_err(|e| format!("failed to sync workspace review into artifact: {e}"))?;
            return Ok(Some(workspace_text));
        }
    }

    if artifact_review_path.exists() {
        let artifact_text = std::fs::read_to_string(artifact_review_path)
            .map_err(|e| format!("failed to read artifact review file: {e}"))?;
        if !artifact_text.trim().is_empty() {
            return Ok(Some(artifact_text));
        }
    }

    Ok(None)
}

fn is_lgtm_str(text: &str) -> bool {
    let trimmed = text.trim().strip_prefix('\u{FEFF}').unwrap_or(text).trim();
    trimmed == "LGTM"
}

fn abort_message_from_result_md(result_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(result_path).ok()?;
    let text = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("ABORT:") {
            return Some(rest.trim_start().to_string());
        }
    }
    None
}

fn fail_on_abort_result(result_path: &Path) -> Result<(), String> {
    if let Some(abort_msg) = abort_message_from_result_md(result_path) {
        return Err(format!("ABORT: {abort_msg}"));
    }
    Ok(())
}

fn prepare_ground_session() -> Result<GroundSession, String> {
    let store = prepare_ground_prompt_store()?;
    let artifacts = create_run_artifacts_from_text(GROUND_REQUEST, Some(Path::new(".")))
        .map_err(|e| e.to_string())?;
    let grounding_exists = artifacts.work_dir.join("grounding.md").is_file();
    let grounding_backup = backup_workspace_grounding_if_present(&artifacts.work_dir)?;
    let write_prompt = build_write_grounding_prompt(&store, &artifacts)?;
    let improve_prompt = build_improve_grounding_prompt(&store, &artifacts)?;
    let check_sync_prompt = build_check_sync_prompt(&store, &artifacts)?;
    Ok(GroundSession {
        artifacts,
        grounding_backup,
        write_prompt,
        improve_prompt,
        check_sync_prompt,
        grounding_exists,
    })
}

async fn run_ground_acp(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    write_prompt: &str,
    improve_prompt: &str,
    check_sync_prompt: &str,
    grounding_exists: bool,
) -> Result<(), String> {
    let timing = client.attach_run_timing_for_session();
    let begin_res = client.begin_coder_session(&artifacts.work_dir).await;
    if let Err(e) = begin_res {
        client.set_run_timing(None);
        return Err(e.to_string());
    }
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_implement_display_name("ground");
    let acp_result = run_grounding_discrepancy_loop(
        client,
        artifacts,
        write_prompt,
        improve_prompt,
        check_sync_prompt,
        grounding_exists,
    )
    .await;
    let end_result = client.end_coder_session().await.map_err(|e| e.to_string());
    let result = merge_ground_acp_results(acp_result, end_result);
    timing_merge::emit_run_timing_after_acp(client, &artifacts.run_dir, &timing, result)
}

fn merge_with_abort_check(
    primary: Result<(), String>,
    restore_result: Result<(), String>,
    artifact_result_path: &Path,
) -> Result<(), String> {
    let merged_result = timing_merge::prefer_primary_over_secondary(
        primary,
        restore_result,
        "kissconfig restore failed",
    );
    if let Some(abort) = abort_message_from_result_md(artifact_result_path) {
        return match merged_result {
            Ok(()) => Err(format!("ABORT: {abort}")),
            Err(merge_error) => Err(format!("ABORT: {abort}; {merge_error}")),
        };
    }
    merged_result
}

fn merge_ground_acp_results(
    primary: Result<(), String>,
    end_result: Result<(), String>,
) -> Result<(), String> {
    match (primary, end_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(end_err)) => Err(format!("failed to end coder session: {end_err}")),
        (Err(primary_err), Ok(())) => Err(primary_err),
        (Err(primary_err), Err(end_err)) => Err(format!(
            "{primary_err}; failed to end coder session: {end_err}"
        )),
    }
}

async fn run_grounding_discrepancy_loop(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    write_prompt: &str,
    improve_prompt: &str,
    check_sync_prompt: &str,
    grounding_exists: bool,
) -> Result<(), String> {
    if !grounding_exists {
        run_grounding_write_attempt(client, artifacts, write_prompt).await?;
        fail_on_abort_result(&artifacts.artifact_result_md())?;
    }

    for attempt in 1..=GROUND_MAX_LOOPS {
        run_grounding_check_attempt(client, artifacts, check_sync_prompt, attempt).await?;
        fail_on_abort_result(&artifacts.artifact_result_md())?;
        let review_text = sync_review_file_for_attempt(
            &artifacts.artifact_review_md(),
            &artifacts.workspace_review_md(),
        )?;
        if review_text.as_deref().is_some_and(is_lgtm_str) {
            return Ok(());
        }
        if attempt == GROUND_MAX_LOOPS {
            break;
        }
        run_grounding_improve_attempt(client, artifacts, improve_prompt, attempt).await?;
        fail_on_abort_result(&artifacts.artifact_result_md())?;
    }
    Err("Did not receive LGTM for check_sync.md within max loops.".to_string())
}

async fn run_grounding_write_attempt(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    write_prompt: &str,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            write_prompt,
            &artifacts.log_path("write_grounding"),
            "write_grounding",
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                skip_repo_style: true,
                do_trace_split: None,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string())
}

async fn run_grounding_improve_attempt(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    improve_prompt: &str,
    attempt: usize,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            improve_prompt,
            &artifacts.log_path(&format!("grounding_attempt_{attempt}")),
            &format!("ground_attempt_{attempt}"),
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                skip_repo_style: true,
                do_trace_split: None,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string())
}

async fn run_grounding_check_attempt(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    check_sync_prompt: &str,
    attempt: usize,
) -> Result<(), String> {
    clear_review_file(&artifacts.artifact_review_md())?;
    clear_review_file(&artifacts.workspace_review_md())?;
    client
        .run_coder_prompt(
            check_sync_prompt,
            &artifacts.log_path(&format!("check_sync_attempt_{attempt}")),
            &format!("check_sync_attempt_{attempt}"),
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::SyncCheck),
                skip_repo_style: false,
                do_trace_split: None,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn run_ground(shared: &SharedOpts, workflow: WorkflowCliOptions) -> Result<(), String> {
    let session = prepare_ground_session()?;
    repo_checks::run_repo_workspace_gates(
        &session.artifacts.work_dir,
        repo_checks::RepoGateOutput::Tagged,
    )?;
    let mut client = super::build_agent(shared, workflow, shared.acp_stdout_markdown_enabled());
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    run_emit::emit_run_startup_sequence(
        &session.artifacts,
        shared.tee_startup_stdout(),
        GROUND_REQUEST,
    )?;
    let result = run_ground_acp(
        &mut client,
        &session.artifacts,
        &session.write_prompt,
        &session.improve_prompt,
        &session.check_sync_prompt,
        session.grounding_exists,
    )
    .await;
    let result = merge_with_abort_check(
        result,
        restore_workspace_kissconfig(&session.artifacts.work_dir, &session.grounding_backup),
        &session.artifacts.artifact_result_md(),
    );
    result?;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

#[test]
fn stringify_ground_flow_helpers() {
    let _ = stringify!(crate::cli::ground_cmd::prepare_ground_prompt_store);
    let _ = stringify!(crate::cli::ground_cmd::build_write_grounding_prompt);
    let _ = stringify!(crate::cli::ground_cmd::build_improve_grounding_prompt);
    let _ = stringify!(crate::cli::ground_cmd::build_check_sync_prompt);
    let _ = stringify!(crate::cli::ground_cmd::clear_review_file);
    let _ = stringify!(crate::cli::ground_cmd::sync_review_file_for_attempt);
    let _ = stringify!(crate::cli::ground_cmd::fail_on_abort_result);
    let _ = stringify!(crate::cli::ground_cmd::abort_message_from_result_md);
    let _ = stringify!(crate::cli::ground_cmd::prepare_ground_session);
    let _ = stringify!(crate::cli::ground_cmd::GroundSession);
    let _ = stringify!(crate::cli::ground_cmd::run_ground_acp);
    let _ = stringify!(crate::cli::ground_cmd::run_grounding_write_attempt);
    let _ = stringify!(crate::cli::ground_cmd::run_grounding_improve_attempt);
    let _ = stringify!(crate::cli::ground_cmd::run_ground);
}

#[test]
fn build_check_sync_prompt_includes_kpop_from_workflow_context() {
    let tmp = tempfile::tempdir().unwrap();
    let prompts = tmp.path().join(".malvin").join("prompts");
    std::fs::create_dir_all(&prompts).unwrap();
    std::fs::write(prompts.join("check_sync.md"), "{{ kpop }}").unwrap();
    std::fs::write(prompts.join("kpop_common.md"), "KPOP_PLACEHOLDER_CONTENT").unwrap();
    std::fs::write(prompts.join("write_grounding.md"), "write grounding prompt").unwrap();
    std::fs::write(
        prompts.join("improve_grounding.md"),
        "improve grounding prompt",
    )
    .unwrap();

    let artifacts =
        create_run_artifacts_from_text(GROUND_REQUEST, Some(tmp.path())).expect("run artifacts");
    let store = PromptStore::with_root(prompts);

    let prompt = build_check_sync_prompt(&store, &artifacts).expect("build check_sync");
    assert_eq!(prompt.trim(), "KPOP_PLACEHOLDER_CONTENT");
}

#[test]
fn merge_with_abort_check_preserves_abort_over_restore_error() {
    let tmp = tempfile::tempdir().unwrap();
    let result_file = tmp.path().join("artifact_result.md");
    std::fs::write(&result_file, "ABORT: user requested stop\n").unwrap();

    let merged = merge_with_abort_check(
        Err("acp failure".into()),
        Err("restore failure".into()),
        &result_file,
    );

    let error = merged.expect_err("should fail with abort");
    assert_eq!(
        error,
        "ABORT: user requested stop; acp failure; kissconfig restore failed: restore failure"
    );
}

#[test]
fn merge_ground_acp_results_surfaces_end_session_errors() {
    assert_eq!(
        merge_ground_acp_results(Ok(()), Err("end session failed".to_string()),),
        Err("failed to end coder session: end session failed".to_string())
    );
    assert_eq!(
        merge_ground_acp_results(
            Err("acp failed".to_string()),
            Err("end session failed".to_string()),
        ),
        Err("acp failed; failed to end coder session: end session failed".to_string())
    );
}
