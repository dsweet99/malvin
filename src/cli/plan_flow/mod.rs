mod plan_prompt;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use malvin::acp::{AgentClient, CoderPromptOptions};
use malvin::artifacts::{
    RunArtifacts, SessionDotfileBackups, backup_workspace_kissconfig_if_present,
    backup_workspace_kissignore_if_present, backup_workspace_malvin_checks_if_present,
    create_run_artifacts, restore_workspace_session_dotfiles,
};
use malvin::output::{MALVIN_WHO, print_stdout_line};
use malvin::run_timing::{RunTiming, TimingPhase};

use super::PlanArgs;
use super::SharedOpts;
use super::code_flow::{WorkflowCliOptions, build_agent};
use super::run_emit;
use super::timing_merge;

fn resolve_user_plan_path(plan_path: Option<PathBuf>) -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let p = plan_path.unwrap_or_else(|| cwd.join("plan.md"));
    Ok(if p.is_absolute() { p } else { cwd.join(p) })
}

fn normalized_plan_file_bytes(text: &str) -> Result<Vec<u8>, String> {
    if text.trim().is_empty() {
        return Err("ERR: plan text is empty (after trimming).".to_string());
    }
    let core = text.trim_end_matches(['\r', '\n']);
    let mut s = String::with_capacity(core.len() + 1);
    s.push_str(core);
    s.push('\n');
    Ok(s.into_bytes())
}

fn write_plan_source(plan: &PlanArgs, user_plan_path: &Path) -> Result<(), String> {
    if let Some(ref t) = plan.text {
        let bytes = normalized_plan_file_bytes(t)?;
        if let Some(parent) = user_plan_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(user_plan_path, bytes).map_err(|e| e.to_string())?;
        return Ok(());
    }
    if !user_plan_path.exists() {
        return Err(format!(
            "ERR: plan file does not exist: {}",
            user_plan_path.display()
        ));
    }
    Ok(())
}

fn artifacts_work_dir_for_run(user_plan_path: &Path) -> PathBuf {
    user_plan_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn plan_run_artifacts(user_plan_path: &Path) -> Result<RunArtifacts, String> {
    let work_dir_for_run = artifacts_work_dir_for_run(user_plan_path);
    create_run_artifacts(user_plan_path, Some(work_dir_for_run.as_path()))
        .map_err(|e| e.to_string())
}

fn start_plan_workspace_session(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    shared: &SharedOpts,
    user_plan_path: &Path,
) -> Result<SessionDotfileBackups, String> {
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    malvin::repo_gates::ensure_default_malvin_checks_file(&artifacts.work_dir)?;
    let malvin_checks_backup = backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;
    let kissconfig_backup = backup_workspace_kissconfig_if_present(&artifacts.work_dir)?;
    let kissignore_backup = backup_workspace_kissignore_if_present(&artifacts.work_dir)?;
    let startup_tag = user_plan_path.display().to_string();
    run_emit::emit_run_startup_sequence(artifacts, shared.tee_startup_stdout(), &startup_tag)?;
    client.prompts_log_run_dir = Some(artifacts.run_dir.clone());
    Ok(SessionDotfileBackups::from_parts(
        kissconfig_backup,
        malvin_checks_backup,
        kissignore_backup,
    ))
}

fn build_rendered_plan_prompt(
    artifacts: &RunArtifacts,
    user_plan_path: &Path,
) -> Result<String, String> {
    let store = plan_prompt::prepare_plan_prompt_store()?;
    let context = plan_prompt::plan_prompt_context(artifacts, user_plan_path, &store)?;
    plan_prompt::compose_plan_prompt(&store, &context)
}

fn set_plan_timing_label(timing: &Arc<Mutex<RunTiming>>) {
    let mut g = timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    g.set_implement_display_name("plan");
}

fn restore_after_plan_prompt(
    work_dir: &Path,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), String> {
    restore_workspace_session_dotfiles(work_dir, session_dotfile_backups)
    .map_err(|e| format!("workspace session restore failed after plan prompt: {e}"))
}

fn pair_run_and_restore(
    run_res: Result<(), String>,
    restore_res: Result<(), String>,
) -> Result<(), String> {
    match (run_res, restore_res) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), Ok(())) | (Ok(()), Err(e)) => Err(e),
        (Err(e), Err(r)) => Err(format!(
            "{e}; workspace session restore failed after plan prompt: {r}"
        )),
    }
}

async fn plan_coder_prompt(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    prompt: &str,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            prompt,
            &artifacts.log_path("review_plan"),
            "review_plan",
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

struct PlanReviewOnce<'a> {
    artifacts: &'a RunArtifacts,
    session_dotfile_backups: &'a SessionDotfileBackups,
    prompt: &'a str,
}

async fn run_plan_review_once(
    client: &mut AgentClient,
    request: PlanReviewOnce<'_>,
) -> Result<(), String> {
    let timing = client.attach_run_timing_for_session();
    set_plan_timing_label(&timing);
    client.prompts_log_run_dir = Some(request.artifacts.run_dir.clone());
    let begin_res = client
        .begin_coder_session(&request.artifacts.work_dir)
        .await
        .map_err(|e| e.to_string());
    if let Err(e) = begin_res {
        client.set_run_timing(None);
        return Err(e);
    }

    let run_res = plan_coder_prompt(client, request.artifacts, request.prompt).await;
    let restore_res = restore_after_plan_prompt(
        &request.artifacts.work_dir,
        request.session_dotfile_backups,
    );
    let acp_result = pair_run_and_restore(run_res, restore_res);

    let end_result = client.end_coder_session().await.map_err(|e| e.to_string());
    let acp_result =
        timing_merge::prefer_primary_over_secondary(acp_result, end_result, "end_coder_session");
    timing_merge::emit_run_timing_after_acp(client, &request.artifacts.run_dir, &timing, acp_result)
}

pub async fn run_plan(
    plan: PlanArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let user_plan_path = resolve_user_plan_path(plan.plan_path.clone())?;
    write_plan_source(&plan, &user_plan_path)?;
    let artifacts = plan_run_artifacts(&user_plan_path)?;
    super::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    let r = async {
        let mut client = build_agent(shared, workflow, shared.acp_stdout_markdown_enabled());
        let session_dotfile_backups =
            start_plan_workspace_session(&mut client, &artifacts, shared, &user_plan_path)?;
        let prompt = build_rendered_plan_prompt(&artifacts, &user_plan_path)?;
        let wf_res = run_plan_review_once(
            &mut client,
            PlanReviewOnce {
                artifacts: &artifacts,
                session_dotfile_backups: &session_dotfile_backups,
                prompt: &prompt,
            },
        )
        .await;
        timing_merge::merge_acp_with_workspace_session_restore_and_check_abort(
            wf_res,
            &artifacts.work_dir,
            &session_dotfile_backups,
            &artifacts.artifact_result_md(),
        )?;
        print_stdout_line(MALVIN_WHO, "DONE");
        Ok(())
    }
    .await;
    if r.is_ok() {
        super::error_run_log::clear_command_error_run_dir();
    }
    r
}

#[cfg(test)]
mod tests;
