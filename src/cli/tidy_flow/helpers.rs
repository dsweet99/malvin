use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

use malvin::acp::{AgentClient, CoderPromptOptions};
use malvin::artifacts::{
    RunArtifacts, SessionDotfileBackups, backup_workspace_kissconfig_if_present,
    backup_workspace_kissignore_if_present, backup_workspace_malvin_checks_if_present,
    create_run_artifacts_from_text, restore_workspace_session_dotfiles,
};
use malvin::orchestrator::{
    ReviewAttemptKernelInput, format_prompt_path, load_review_descriptions_for_kernel,
    review_attempt_is_lgtm, run_review_fanout_prefix, should_run_learn_check, workflow_context,
};
use malvin::prompts::{HEADER_MD, PromptError, PromptStore, merged_coding_rules};
use malvin::run_timing::TimingPhase;

use super::repo_checks::{RepoGateOutput, run_repo_workspace_gates};
use super::timing_merge;
use super::{
    LEARN_MIN_ELAPSED_MS, SharedOpts, WorkflowCliOptions, build_agent, emit_run_startup_sequence,
};

pub enum TidyStartup {
    /// Workspace gates (`kiss check`, `.malvin_checks`, …) already passed; skip coder session.
    SkipAgent {
        artifacts: RunArtifacts,
        session_dotfile_backups: SessionDotfileBackups,
    },
    RunAgent {
        client: AgentClient,
        artifacts: RunArtifacts,
        session_dotfile_backups: SessionDotfileBackups,
        store: PromptStore,
        context: HashMap<String, String>,
        run_learn: bool,
    },
}

pub struct TidyAcpInput<'a> {
    pub(crate) client: &'a mut AgentClient,
    pub(crate) artifacts: &'a RunArtifacts,
    pub(crate) store: &'a PromptStore,
    pub(crate) context: &'a HashMap<String, String>,
    pub(crate) run_learn: bool,
}

pub struct TidyPromptRestore<'a> {
    pub(crate) prompt: &'a str,
    pub(crate) label: &'a str,
    pub(crate) phase: TimingPhase,
    pub(crate) session_dotfile_backups: &'a SessionDotfileBackups,
    pub(crate) restore_context: &'a str,
}

pub fn prepare_tidy_prompt_store() -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(HEADER_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("tidy.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("coding_rules.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("summary.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("review_descriptions.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("reviewer_template.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("review_write.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("tidy_concerns.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn compose_tidy_prompt(
    store: &PromptStore,
    context: &HashMap<String, String>,
) -> Result<String, String> {
    let header = store
        .render_prompt_only(HEADER_MD, context)
        .map_err(|e: PromptError| e.0)?;
    let rules = merged_coding_rules(store, context).map_err(|e: PromptError| e.0)?;
    let tidy = store
        .render("tidy.md", context)
        .map_err(|e: PromptError| e.0)?;
    Ok(format!(
        "{}\n\n{}\n\n{}",
        header.trim_end(),
        rules.trim_end(),
        tidy.trim_end()
    ))
}

pub async fn run_tidy_prompt(
    input: &mut TidyAcpInput<'_>,
    prompt: &str,
    kind: &str,
    phase: TimingPhase,
) -> Result<(), String> {
    input
        .client
        .run_coder_prompt(
            prompt,
            &input.artifacts.log_path(kind),
            kind,
            CoderPromptOptions {
                llm_phase: Some(phase),
                skip_repo_style: true,
                do_trace_split: None,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub fn compose_tidy_concerns_prompt(
    store: &PromptStore,
    context: &HashMap<String, String>,
) -> Result<String, String> {
    store
        .render("tidy_concerns.md", context)
        .map_err(|e: PromptError| e.0)
}

pub fn write_checks_do_not_pass_to_review_path(review_path: &Path) -> Result<(), String> {
    if let Some(parent) = review_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "failed to create parent dirs for {}: {e}",
                review_path.display()
            )
        })?;
    }
    std::fs::write(review_path, b"Checks do not pass\n").map_err(|e| {
        format!(
            "failed to write checks-do-not-pass marker {}: {e}",
            review_path.display()
        )
    })
}

pub fn write_checks_do_not_pass_for_artifacts(artifacts: &RunArtifacts) -> Result<(), String> {
    write_checks_do_not_pass_to_review_path(&artifacts.artifact_review_md())?;
    write_checks_do_not_pass_to_review_path(&artifacts.workspace_review_md())
}

pub struct TidyReviewWriteSession<'a> {
    pub client: &'a mut AgentClient,
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub context: &'a HashMap<String, String>,
    pub reviewers_subdir: &'a Path,
    pub attempt: usize,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
}

pub async fn run_tidy_review_write(session: TidyReviewWriteSession<'_>) -> Result<(), String> {
    let TidyReviewWriteSession {
        client,
        store,
        artifacts,
        context,
        reviewers_subdir,
        attempt,
        session_dotfile_backups,
    } = session;
    let mut write_ctx = context.clone();
    write_ctx.insert(
        "reviewers_subdir".to_string(),
        format_prompt_path(reviewers_subdir, &artifacts.work_dir),
    );
    let prompt = store
        .render("review_write.md", &write_ctx)
        .map_err(|e: PromptError| e.0)?;
    let label = format!("review_write_attempt_{attempt}");
    let run_result = client
        .run_coder_prompt(
            prompt.as_str(),
            &artifacts.log_path(label.as_str()),
            label.as_str(),
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::ReviewWrite),
                skip_repo_style: true,
                do_trace_split: None,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string());
    let restore_result =
        restore_workspace_session_dotfiles(&artifacts.work_dir, session_dotfile_backups);
    match (run_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), Ok(())) | (Ok(()), Err(e)) => Err(e),
        (Err(run_err), Err(restore_err)) => Err(format!("{run_err}, {restore_err}")),
    }
}

pub async fn run_tidy_review_attempt(
    input: &mut TidyAcpInput<'_>,
    descriptions: &[String],
    attempt: usize,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<bool, String> {
    let kernel = ReviewAttemptKernelInput {
        store: input.store,
        artifacts: input.artifacts,
        context: input.context,
        descriptions,
        attempt,
    };
    let reviewers_subdir = run_review_fanout_prefix(&*input.client, &kernel)
        .await
        .map_err(|e| e.0)?;
    run_tidy_review_write(TidyReviewWriteSession {
        client: input.client,
        store: input.store,
        artifacts: input.artifacts,
        context: input.context,
        reviewers_subdir: &reviewers_subdir,
        attempt,
        session_dotfile_backups,
    })
    .await?;
    review_attempt_is_lgtm(input.artifacts).map_err(|e| e.0)
}

async fn run_tidy_concerns_coder_turn(
    input: &mut TidyAcpInput<'_>,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), String> {
    let concerns = compose_tidy_concerns_prompt(input.store, input.context)?;
    run_tidy_prompt_with_restore(
        input,
        TidyPromptRestore {
            prompt: concerns.as_str(),
            label: "tidy",
            phase: TimingPhase::Implement,
            session_dotfile_backups,
            restore_context: "tidy",
        },
    )
    .await
}

async fn run_tidy_bonus_gate_recovery(
    input: &mut TidyAcpInput<'_>,
    descriptions: &[String],
    attempt: usize,
    session_dotfile_backups: &SessionDotfileBackups,
    work_dir: &Path,
    run_dir: &Path,
) -> Result<bool, String> {
    run_tidy_concerns_coder_turn(input, session_dotfile_backups).await?;
    if run_repo_workspace_gates(work_dir, RepoGateOutput::Tagged, Some(run_dir)).is_err() {
        return Ok(false);
    }
    let lgtm = run_tidy_review_attempt(
        input,
        descriptions,
        attempt,
        session_dotfile_backups,
    )
    .await?;
    if !lgtm {
        return Ok(false);
    }
    if run_repo_workspace_gates(work_dir, RepoGateOutput::Tagged, Some(run_dir)).is_ok() {
        return Ok(true);
    }
    write_checks_do_not_pass_for_artifacts(input.artifacts)?;
    Ok(false)
}

pub async fn run_tidy_interleaved_loop(
    input: &mut TidyAcpInput<'_>,
    initial_tidy_prompt: &str,
    session_dotfile_backups: &SessionDotfileBackups,
    max_loops: usize,
) -> Result<(), String> {
    let max_attempts = max_loops.max(1);
    let work_dir = input.artifacts.work_dir.clone();
    let run_dir = input.artifacts.run_dir.clone();
    let descriptions = load_review_descriptions_for_kernel(input.store).map_err(|e| e.0)?;
    for attempt in 1..=max_attempts {
        print_stdout_line(
            MALVIN_WHO,
            &format!("tidy iteration {attempt}/{max_attempts}"),
        );
        let coder_prompt: Cow<'_, str> = if attempt == 1 {
            Cow::Borrowed(initial_tidy_prompt)
        } else {
            Cow::Owned(compose_tidy_concerns_prompt(input.store, input.context)?)
        };
        run_tidy_prompt_with_restore(
            input,
            TidyPromptRestore {
                prompt: coder_prompt.as_ref(),
                label: "tidy",
                phase: TimingPhase::Implement,
                session_dotfile_backups,
                restore_context: "tidy",
            },
        )
        .await?;

        let lgtm = run_tidy_review_attempt(
            input,
            &descriptions,
            attempt,
            session_dotfile_backups,
        )
        .await?;
        if lgtm
            && run_repo_workspace_gates(&work_dir, RepoGateOutput::Tagged, Some(&run_dir)).is_ok()
        {
            return Ok(());
        }
        if lgtm {
            write_checks_do_not_pass_for_artifacts(input.artifacts)?;
            if attempt < max_attempts {
                continue;
            }
            let bonus = max_attempts + 1;
            print_stdout_line(
                MALVIN_WHO,
                &format!("tidy iteration {bonus}/{max_attempts}"),
            );
            if run_tidy_bonus_gate_recovery(
                input,
                &descriptions,
                attempt,
                session_dotfile_backups,
                &work_dir,
                &run_dir,
            )
            .await?
            {
                return Ok(());
            }
        }
    }
    Err(format!(
        "tidy did not converge within {max_attempts} iterations"
    ))
}

async fn run_tidy_learn_mid_gates_and_summary(
    input: &mut TidyAcpInput<'_>,
    timing: &std::sync::Arc<std::sync::Mutex<malvin::run_timing::RunTiming>>,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), String> {
    if input.run_learn {
        let elapsed_ms = timing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .elapsed_so_far()
            .as_millis();
        if should_run_learn_check(
            LEARN_MIN_ELAPSED_MS,
            u64::try_from(elapsed_ms).unwrap_or(u64::MAX),
        ) {
            let learn_prompt = input
                .store
                .render("learn.md", input.context)
                .map_err(|e: PromptError| e.0)?;
            run_tidy_prompt_with_restore(
                input,
                TidyPromptRestore {
                    prompt: &learn_prompt,
                    label: "learn",
                    phase: TimingPhase::Learn,
                    session_dotfile_backups,
                    restore_context: "learn",
                },
            )
            .await?;
        }
    }
    super::repo_checks::run_repo_workspace_gates(
        &input.artifacts.work_dir,
        RepoGateOutput::Tagged,
        Some(&input.artifacts.run_dir),
    )?;
    let header_only = input
        .store
        .render_prompt_only(HEADER_MD, input.context)
        .map_err(|e: PromptError| e.0)?;
    let summary_only = input
        .store
        .render("summary.md", input.context)
        .map_err(|e: PromptError| e.0)?;
    let summary_prompt = format!(
        "{}\n\n{}",
        header_only.trim_end(),
        summary_only.trim_end()
    );
    run_tidy_prompt_with_restore(
        input,
        TidyPromptRestore {
            prompt: &summary_prompt,
            label: "summary",
            phase: TimingPhase::Summary,
            session_dotfile_backups,
            restore_context: "summary",
        },
    )
    .await?;
    Ok(())
}

pub async fn run_tidy_acp(
    input: &mut TidyAcpInput<'_>,
    prompt: &str,
    session_dotfile_backups: &SessionDotfileBackups,
    max_loops: usize,
) -> Result<(), String> {
    let timing = input.client.attach_run_timing_for_session();
    input.client.prompts_log_run_dir = Some(input.artifacts.run_dir.clone());
    let begin_res = input
        .client
        .begin_coder_session(&input.artifacts.work_dir)
        .await;
    if let Err(e) = begin_res {
        input.client.set_run_timing(None);
        return Err(e.to_string());
    }

    let mut acp_result = run_tidy_interleaved_loop(
        input,
        prompt,
        session_dotfile_backups,
        max_loops,
    )
    .await;
    if acp_result.is_ok() {
        acp_result = run_tidy_learn_mid_gates_and_summary(
            input,
            &timing,
            session_dotfile_backups,
        )
        .await;
    }
    let end_result = input
        .client
        .end_coder_session()
        .await
        .map_err(|e| e.to_string());
    if end_result.is_err() {
        if acp_result.is_ok() {
            acp_result = end_result;
        } else {
            acp_result = Err(format!("{acp_result:?} end_coder_session: {end_result:?}"));
        }
    }

    timing_merge::emit_run_timing_after_acp(
        input.client,
        &input.artifacts.run_dir,
        &timing,
        acp_result,
    )
}

pub async fn run_tidy_prompt_with_restore(
    input: &mut TidyAcpInput<'_>,
    request: TidyPromptRestore<'_>,
) -> Result<(), String> {
    let acp_result = run_tidy_prompt(input, request.prompt, request.label, request.phase).await;
    let restore_result = restore_workspace_session_dotfiles(
        &input.artifacts.work_dir,
        request.session_dotfile_backups,
    )
    .map_err(|e| format!("tidy restore failed after {}: {e}", request.restore_context));
    match (acp_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), Ok(())) | (Ok(()), Err(e)) => Err(e),
        (Err(e), Err(restore_error)) => Err(format!(
            "{e}; tidy restore failed after {}: {restore_error}",
            request.restore_context
        )),
    }
}

pub fn tidy_prompt_context(
    artifacts: &RunArtifacts,
) -> Result<(PromptStore, HashMap<String, String>), String> {
    let store = prepare_tidy_prompt_store()?;
    let context = workflow_context(artifacts, &store, "tidy").map_err(|e: PromptError| e.0)?;
    Ok((store, context))
}

pub fn prepare_tidy_run(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    run_learn: bool,
) -> Result<TidyStartup, String> {
    let artifacts =
        create_run_artifacts_from_text("tidy", Some(Path::new("."))).map_err(|e| e.to_string())?;
    malvin::repo_gates::ensure_default_malvin_checks_file(&artifacts.work_dir)?;

    let gates_ok = run_repo_workspace_gates(
        &artifacts.work_dir,
        RepoGateOutput::Tagged,
        Some(&artifacts.run_dir),
    )
    .is_ok();

    let malvin_checks_backup =
        backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;

    if gates_ok {
        emit_run_startup_sequence(&artifacts, shared.tee_startup_stdout(), "tidy")?;
        let kissconfig_backup = backup_workspace_kissconfig_if_present(&artifacts.work_dir)?;
        let kissignore_backup = backup_workspace_kissignore_if_present(&artifacts.work_dir)?;
        let session_dotfile_backups = SessionDotfileBackups::from_parts(
            kissconfig_backup,
            malvin_checks_backup,
            kissignore_backup,
        );
        return Ok(TidyStartup::SkipAgent {
            artifacts,
            session_dotfile_backups,
        });
    }

    let mut client = build_agent(shared, workflow, shared.acp_stdout_markdown_enabled());
    client.prompts_log_run_dir = Some(artifacts.run_dir.clone());
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    emit_run_startup_sequence(&artifacts, shared.tee_startup_stdout(), "tidy")?;
    let kissconfig_backup = backup_workspace_kissconfig_if_present(&artifacts.work_dir)?;
    let kissignore_backup = backup_workspace_kissignore_if_present(&artifacts.work_dir)?;
    let session_dotfile_backups = SessionDotfileBackups::from_parts(
        kissconfig_backup,
        malvin_checks_backup,
        kissignore_backup,
    );
    let (store, context) = tidy_prompt_context(&artifacts)?;
    Ok(TidyStartup::RunAgent {
        client,
        artifacts,
        session_dotfile_backups,
        store,
        context,
        run_learn,
    })
}

pub fn merge_tidy_timing(
    result: Result<(), String>,
    artifacts: &RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), String> {
    timing_merge::merge_acp_with_workspace_session_restore_and_check_abort(
        result,
        &artifacts.work_dir,
        session_dotfile_backups,
        &artifacts.artifact_result_md(),
    )?;
    Ok(())
}
