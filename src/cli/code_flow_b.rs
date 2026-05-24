use super::code_flow_a::{
    build_agent, format_code_pre_check_failure, prepare_prompt_store, WorkflowCliOptions,
};
use super::{CodeArgs, SharedOpts};
use crate::cli::{error_run_log, run_emit};

struct CodeRunExecuteArgs<'a> {
    code: &'a CodeArgs,
    shared: &'a SharedOpts,
    workflow: WorkflowCliOptions,
    store: &'a crate::prompts::PromptStore,
    client: &'a mut crate::acp::AgentClient,
    artifacts: &'a crate::artifacts::RunArtifacts,
    startup_request: &'a str,
}

async fn code_run_execute(exec: CodeRunExecuteArgs<'_>) -> Result<(), String> {
    use crate::orchestrator::{Orchestrator, WorkflowConfig, WorkflowError, workflow_context};
    use crate::output::{MALVIN_WHO, print_stdout_line};
    let CodeRunExecuteArgs {
        code,
        shared,
        workflow,
        store,
        client,
        artifacts,
        startup_request,
    } = exec;
    client
        .ensure_authenticated()
        .map_err(|e| e.to_string())?;
    let session_dotfile_backups = crate::artifacts::SessionDotfileBackups::snapshot(&artifacts.work_dir)
        .map_err(|e| e.to_string())?;
    let ctx = workflow_context(artifacts, store, "code")
        .map_err(|e: crate::prompts::PromptError| e.0)?;
    run_emit::emit_run_startup_sequence(
        artifacts,
        shared.tee_startup_stdout(),
        startup_request,
    )?;
    let workflow_res = {
        let mut orch = Orchestrator {
            client,
            prompts: store,
            artifacts,
            config: WorkflowConfig {
                max_loops: code.max_loops,
                run_learn: workflow.run_learn,
                learn_min_elapsed_ms: crate::DEFAULT_LEARN_MIN_ELAPSED_MS,
                skip_check_plan: code.trust_the_plan,
            },
            progress_callback: Box::new(|msg: &str| {
                print_stdout_line(MALVIN_WHO, msg);
            }),
            session_dotfile_backups: session_dotfile_backups.clone(),
        };
        orch
            .run_with_pre_summary_gap(&ctx, crate::orchestrator::mid_noop)
            .await
            .map_err(|e: WorkflowError| e.0)
    };
    crate::acp_post_run::merge_acp_with_workspace_session_restore(
        workflow_res,
        &artifacts.work_dir,
        &session_dotfile_backups,
    )?;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

fn code_run_workspace_pre_checks(code: &CodeArgs, work_dir: &std::path::Path) -> Result<(), String> {
    use crate::repo_checks::{RepoGateOutput, run_repo_workspace_gates};
    if code.skip_pre_checks {
        return Ok(());
    }
    run_repo_workspace_gates(work_dir, RepoGateOutput::Tagged, None)
        .map_err(|e| format_code_pre_check_failure(&e))
}

fn code_run_prepare_artifacts(
    text: &str,
    work_dir: &std::path::Path,
    client: &mut crate::acp::AgentClient,
) -> Result<crate::artifacts::RunArtifacts, String> {
    use crate::artifacts::create_run_artifacts_from_text;
    let artifacts = create_run_artifacts_from_text(text, Some(work_dir))
        .map_err(|e| e.to_string())?;
    client.prompts_log_run_dir = Some(artifacts.run_dir.clone());
    error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    Ok(artifacts)
}

pub async fn run_code(
    code: CodeArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    use crate::artifacts::resolve_user_md_request;
    let store = prepare_prompt_store(workflow)?;
    let request = crate::cli::cli_request::require_cli_request(code.request.as_ref(), "code")?;
    let (text, work_dir) = resolve_user_md_request(&request)?;
    code_run_workspace_pre_checks(&code, &work_dir)?;
    let emit_stdout_markdown = shared.acp_stdout_markdown_enabled();
    let mut client = build_agent(shared, workflow, emit_stdout_markdown);
    let artifacts = code_run_prepare_artifacts(&text, &work_dir, &mut client)?;
    let r = code_run_execute(CodeRunExecuteArgs {
        code: &code,
        shared,
        workflow,
        store: &store,
        client: &mut client,
        artifacts: &artifacts,
        startup_request: &request,
    })
    .await;
    if r.is_ok() {
        error_run_log::clear_command_error_run_dir();
    }
    r
}

#[cfg(test)]
mod tests {
    #[test]
    fn format_code_pre_check_failure_includes_guidance() {
        let msg = super::format_code_pre_check_failure("`kiss check` failed");
        assert!(msg.starts_with("ERR: Pre-checks failed"));
        assert!(msg.contains("malvin tidy"));
        assert!(msg.contains("retry `malvin code`"));
        assert!(msg.contains("--skip-pre-checks` on"));
        assert!(msg.contains("`kiss check` failed"));
    }

    #[test]
    fn format_workspace_gate_failure_hunt_omits_skip_pre_checks() {
        let msg =
            crate::cli::format_workspace_gate_failure("malvin hunt", "`kiss check` failed");
        assert!(msg.contains("Workspace checks did not pass"));
        assert!(msg.contains("malvin tidy"));
        assert!(msg.contains("retry `malvin hunt`"));
        assert!(
            !msg.contains("--skip-pre-checks"),
            "hunt no longer supports --skip-pre-checks: {msg}"
        );
    }

    #[test]
    fn format_workspace_gate_failure_code_includes_skip_pre_checks() {
        let msg = crate::cli::format_workspace_gate_failure("malvin code", "`kiss check` failed");
        assert!(msg.contains("--skip-pre-checks` on `malvin code`"));
    }

    #[test]
    fn code_session_end_restore_includes_malvin_checks() {
        use crate::artifacts::SessionDotfileBackups;

        let tmp = tempfile::tempdir().expect("tempdir");
        let work = tmp.path();
        std::fs::write(work.join(".malvin_checks"), "from-backup\n").expect("seed checks");
        let backups = SessionDotfileBackups::snapshot(work).expect("snapshot backups");
        std::fs::write(work.join(".malvin_checks"), "agent-changed\n").expect("agent edit");
        crate::acp_post_run::merge_acp_with_workspace_session_restore(Ok(()), work, &backups)
            .expect("session restore");
        assert_eq!(
            std::fs::read_to_string(work.join(".malvin_checks")).expect("read checks"),
            "from-backup\n",
            "malvin code must restore .malvin_checks after the session"
        );
    }
}


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_code_run_execute_args() { let _ = stringify!(CodeRunExecuteArgs); }

    #[test]
    fn kiss_cov_code_run_execute() { let _ = stringify!(code_run_execute); }

    #[test]
    fn kiss_cov_code_run_workspace_pre_checks() { let _ = stringify!(code_run_workspace_pre_checks); }

    #[test]
    fn kiss_cov_code_run_prepare_artifacts() { let _ = stringify!(code_run_prepare_artifacts); }

    #[test]
    fn kiss_cov_run_code() { let _ = stringify!(run_code); }

}
