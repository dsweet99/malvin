use crate::agent_backend::{build_agent_backend, AgentBackend};
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::cli::default_output_path::path_relative_to_cwd;
use crate::cli::one_shot_session::{
    finish_one_shot_after_prompt, finish_one_shot_auth_and_backups, OneShotCoderGuard,
};
use crate::cli::run_emit::{emit_run_logs_line, emit_run_startup_banner, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::run_timing::TimingPhase;
use crate::workflow_context::format_prompt_path;

use super::prep::{compose_write_a_prompt, compose_write_b_prompt, write_preflight};
use super::WriteArgs;

struct WriteRunPrep {
    client: AgentBackend,
    artifacts: RunArtifacts,
    prompt_a: String,
    prompt_b: String,
    session_dotfile_backups: SessionDotfileBackups,
}

fn new_write_client(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<AgentBackend, String> {
    build_agent_backend(
        shared,
        workflow,
        shared.acp_stdout_markdown_enabled(),
        "write",
    )
}

fn write_workspace_dir_display(artifacts: &RunArtifacts) -> String {
    format_prompt_path(&artifacts.run_dir, &artifacts.work_dir)
}

fn prepare_write_prompts(
    request_text: &str,
    out_paths: (&str, &str),
    artifacts: &RunArtifacts,
) -> Result<(String, String), String> {
    let workspace_dir = write_workspace_dir_display(artifacts);
    let (tex_display, pdf_display) = out_paths;
    Ok((
        compose_write_a_prompt(request_text, &workspace_dir)?,
        compose_write_b_prompt(tex_display, pdf_display, &workspace_dir)?,
    ))
}

fn create_write_artifacts(
    request_text: &str,
    request_work_dir: &std::path::Path,
) -> Result<RunArtifacts, String> {
    let artifacts = crate::artifacts::create_run_artifacts_from_text_opts(
        request_text,
        Some(request_work_dir),
        crate::run_id::RunDirOptions { gc: false },
    )
    .map_err(|e| e.to_string())?;
    crate::cli::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    Ok(artifacts)
}

async fn prepare_write_run(
    write_args: &mut WriteArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<WriteRunPrep, String> {
    let (request_text, request_work_dir, outputs) = write_preflight(
        write_args.request.as_ref(),
        &write_args.out_path,
        write_args.out_path_explicit,
    )?;
    let tex_display = path_relative_to_cwd(&outputs.tex_path)?;
    let pdf_display = path_relative_to_cwd(&outputs.pdf_path)?;
    write_args.out_path = tex_display.clone();

    let mut client = new_write_client(shared, workflow)?;
    let artifacts = create_write_artifacts(&request_text, &request_work_dir)?;
    let banner_request = write_args
        .request
        .clone()
        .unwrap_or_else(|| request_text.clone());
    emit_run_startup_banner(
        &artifacts,
        RunStartupEmitOpts::from_shared(shared, true),
        &banner_request,
    )?;
    crate::run_id::maybe_gc_after_run_created(&artifacts.work_dir, &artifacts.run_dir);
    let session_dotfile_backups = finish_one_shot_auth_and_backups(&mut client, &artifacts)?;
    let (prompt_a, prompt_b) =
        prepare_write_prompts(&request_text, (&tex_display, &pdf_display), &artifacts)?;
    Ok(WriteRunPrep {
        client,
        artifacts,
        prompt_a,
        prompt_b,
        session_dotfile_backups,
    })
}

async fn run_write_coder_prompt(
    client: &mut AgentBackend,
    artifacts: &RunArtifacts,
    prompt: &str,
    who: &str,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            prompt,
            &artifacts.log_path(who),
            who,
            crate::acp::CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: None,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

async fn run_write_coder_session(
    client: &mut AgentBackend,
    artifacts: &RunArtifacts,
    prompt_a: &str,
    prompt_b: &str,
) -> Result<(), String> {
    let guard = OneShotCoderGuard::begin(client, artifacts, "write").await?;
    let run_res = async {
        run_write_coder_prompt(client, artifacts, prompt_a, "write_a").await?;
        run_write_coder_prompt(client, artifacts, prompt_b, "write_b").await
    }
    .await;
    guard.finish(client, run_res).await
}

pub async fn run_write(
    write_args: &mut WriteArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let mut prep = prepare_write_run(write_args, shared, workflow).await?;
    prep.client
        .begin_coder_session(&prep.artifacts.work_dir)
        .await
        .map_err(|e| e.to_string())?;
    emit_run_logs_line(&prep.artifacts)?;
    let acp_res = run_write_coder_session(
        &mut prep.client,
        &prep.artifacts,
        &prep.prompt_a,
        &prep.prompt_b,
    )
    .await;
    finish_one_shot_after_prompt(
        acp_res,
        &prep.artifacts.work_dir,
        &prep.session_dotfile_backups,
        &prep.artifacts.artifact_result_md(),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_run_write_symbol() {
        let _: Option<WriteRunPrep> = None;
        let _ = run_write;
        let _ = prepare_write_prompts;
        let _ = write_workspace_dir_display;
        let _ = new_write_client;
        let _ = create_write_artifacts;
        let _ = run_write_coder_prompt;
        let _ = run_write_coder_session;
        let _ = prepare_write_run;
    }

    #[test]
    fn prepare_write_prompts_renders_a_then_b() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            crate::artifacts::create_run_artifacts_from_text("topic", Some(tmp.path()))
                .expect("art");
        assert!(!write_workspace_dir_display(&artifacts).is_empty());
        let (a, b) =
            prepare_write_prompts("how gates exit", ("write.tex", "write.pdf"), &artifacts)
                .expect("prompts");
        assert!(a.contains("how gates exit") && a.contains("notes.tex"));
        assert!(b.contains("`write.tex`") && b.contains("`write.pdf`"));
        assert!(!a.contains("{{") && !b.contains("{{"));
    }

    #[test]
    fn create_write_artifacts_sets_work_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = create_write_artifacts("write topic", tmp.path()).expect("art");
        assert!(artifacts.run_dir.is_dir());
        assert_eq!(artifacts.work_dir, tmp.path());
    }

    #[test]
    fn write_client_uses_styled_agent_io() {
        let shared = SharedOpts {
            model: crate::model_id::parse_model_id(crate::config::DEFAULT_CLI_MODEL).expect("model"),
            no_force: true,
            no_tenacious: false,
            gates: false,
            quiet: false,
            verbose: false,
            max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
            doc: false,
            name: None,
            git: false,
            creative: false,
        };
        let io = new_write_client(&shared, WorkflowCliOptions { force: false })
            .expect("backend")
            .io;
        assert!(!io.raw_output);
        assert!(io.show_thoughts_on_stdout && io.emit_stdout_markdown);
    }
}
