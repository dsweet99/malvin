use crate::agent_backend::{
    agent_backend_set_run_timing, AgentBackend,
};
use crate::artifacts::{
    GitignoreBackup, MalvinChecksBackup, MalvinConfigBackup, MalvinConfigWorkspaceBackup,
    RunArtifacts, SessionDotfileBackups, VisionBackup,
};
use crate::cli::checks_discovery_flow::{ensure_malvin_checks_discovered, ChecksDiscoveryOpts};
use crate::cli::SharedOpts;
use crate::prompts::{PromptStore, ROUTER_B_COMPLEX_MD, ROUTER_B_SIMPLE_MD};
use crate::router_flow::router_flow_prompt;
use crate::run_timing::acp_post_run::RunTimingSessionEnd;
use std::sync::{Arc, Mutex};

pub(crate) struct RouterAcpSessionCtx<'a> {
    pub client: &'a mut AgentBackend,
    pub artifacts: &'a RunArtifacts,
    pub coder: &'a router_flow_prompt::RouterCoderRun,
    pub prompt_store: &'a PromptStore,
    pub shared: &'a SharedOpts,
    pub log_path: &'a std::path::Path,
    pub timing: &'a Arc<Mutex<crate::run_timing::RunTiming>>,
    pub session_end: RunTimingSessionEnd,
}

pub(crate) fn router_iteration_log_path(artifacts: &RunArtifacts, agent_loop: usize) -> std::path::PathBuf {
    artifacts.log_path(&format!("router_{agent_loop}"))
}

pub(crate) fn empty_iteration_backups() -> SessionDotfileBackups {
    SessionDotfileBackups::from_parts(crate::session_dotfile_backup::SessionDotfileParts {
        malvin_checks: MalvinChecksBackup::Missing,
        malvin_config: MalvinConfigBackup::Missing,
        gitignore: GitignoreBackup::Missing,
        vision: VisionBackup::Missing,
        malvin_config_workspace: MalvinConfigWorkspaceBackup::Missing,
    })
}

pub(crate) fn snapshot_iteration_backups(work_dir: &std::path::Path) -> SessionDotfileBackups {
    SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)
        .unwrap_or_else(|_| empty_iteration_backups())
}

pub(crate) fn workspace_has_valid_checks(work_dir: &std::path::Path) -> Result<bool, String> {
    let path = crate::malvin_checks_path(work_dir);
    if !path.is_file() {
        return Ok(false);
    }
    let lines = crate::repo_gates::load_malvin_checks(&path)?;
    Ok(!lines.is_empty())
}

pub(crate) enum RouterChecksSnapshotMode {
    KeepPreInit,
    RefreshAfterPossibleInit,
}

pub(crate) struct RouterAInitSnapshotInput {
    pub coding_task: bool,
    pub had_checks: bool,
}

impl RouterAInitSnapshotInput {
    pub(crate) const fn snapshot_mode(self) -> RouterChecksSnapshotMode {
        if self.coding_task && !self.had_checks {
            RouterChecksSnapshotMode::RefreshAfterPossibleInit
        } else {
            RouterChecksSnapshotMode::KeepPreInit
        }
    }
}

pub(crate) fn iteration_backups_after_router_a(
    work_dir: &std::path::Path,
    mode: RouterChecksSnapshotMode,
    pre_init_backups: SessionDotfileBackups,
) -> Result<SessionDotfileBackups, String> {
    match mode {
        RouterChecksSnapshotMode::RefreshAfterPossibleInit => {
            SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)
        }
        RouterChecksSnapshotMode::KeepPreInit => Ok(pre_init_backups),
    }
}

pub(crate) async fn run_router_a_coder_prompt(
    client: &mut AgentBackend,
    coder: &router_flow_prompt::RouterCoderRun,
    log_path: &std::path::Path,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            &coder.combined,
            log_path,
            "router_a",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: None,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn run_router_b_coder_prompt(
    client: &mut AgentBackend,
    router_b_prompt: &str,
    log_path: &std::path::Path,
    label: &str,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            router_b_prompt,
            log_path,
            label,
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: None,
                append_trace: true,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn run_router_c_coder_prompt(
    client: &mut AgentBackend,
    router_c_prompt: &str,
    log_path: &std::path::Path,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            router_c_prompt,
            log_path,
            "router_c",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: None,
                append_trace: true,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn maybe_run_router_init(
    work_dir: &std::path::Path,
    shared: &SharedOpts,
    coding_task: bool,
) -> Result<(), String> {
    if !coding_task {
        return Ok(());
    }
    let prior_error_run_dir = crate::cli::error_run_log::command_error_run_dir();
    let result =
        ensure_malvin_checks_discovered(work_dir, shared, ChecksDiscoveryOpts::INIT).await;
    crate::cli::error_run_log::set_command_error_run_dir(prior_error_run_dir);
    result
}

pub(crate) const fn router_b_template_and_label(complexity_score: u8) -> (&'static str, &'static str) {
    if complexity_score > 3 {
        (ROUTER_B_COMPLEX_MD, "router_b_complex")
    } else {
        (ROUTER_B_SIMPLE_MD, "router_b_simple")
    }
}

pub(crate) async fn run_router_turns(
    ctx: &mut RouterAcpSessionCtx<'_>,
) -> Result<SessionDotfileBackups, String> {
    run_router_a_coder_prompt(ctx.client, ctx.coder, ctx.log_path).await?;
    let agent_text = ctx
        .client
        .last_coder_prompt_agent_response()
        .ok_or_else(|| "router_a: missing agent response".to_string())?;
    let complexity_score = crate::router_flow::router_flow_parse::parse_complexity_score(&agent_text)?;
    let coding_task = crate::router_flow::router_flow_parse::parse_coding_task(&agent_text)?;
    let work_dir = ctx.artifacts.work_dir.as_path();
    let had_checks = workspace_has_valid_checks(work_dir)?;
    let pre_init_backups = SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
    maybe_run_router_init(work_dir, ctx.shared, coding_task).await?;
    let iteration_backups = iteration_backups_after_router_a(
        work_dir,
        RouterAInitSnapshotInput {
            coding_task,
            had_checks,
        }
        .snapshot_mode(),
        pre_init_backups,
    )?;
    let (b_md, router_b_label) = router_b_template_and_label(complexity_score);
    let b_body = router_flow_prompt::build_router_b_prompt(ctx.prompt_store, ctx.artifacts, b_md)?;
    run_router_b_coder_prompt(ctx.client, &b_body, ctx.log_path, router_b_label).await?;
    let router_c_prompt =
        router_flow_prompt::build_router_c_prompt(ctx.prompt_store, ctx.artifacts)?;
    run_router_c_coder_prompt(ctx.client, &router_c_prompt, ctx.log_path).await?;
    Ok(iteration_backups)
}

pub(crate) fn emit_router_acp_timing(
    ctx: &mut RouterAcpSessionCtx<'_>,
    agent_result: Result<(), String>,
) -> Result<(), String> {
    crate::acp_post_run::emit_run_timing_after_backend(crate::acp_post_run::RunTimingAfterBackend {
        backend: ctx.client,
        run_dir: &ctx.artifacts.run_dir,
        timing: ctx.timing,
        agent_result,
        session_end: ctx.session_end,
    })
}

pub(crate) async fn end_router_acp_session(
    ctx: &mut RouterAcpSessionCtx<'_>,
    run_res: Result<(), String>,
) -> Result<(), String> {
    let end_res = ctx.client.end_coder_session().await.map_err(|e| e.to_string());
    let merged = crate::acp_post_run::prefer_primary_over_secondary(run_res, end_res, "end coder session");
    emit_router_acp_timing(ctx, merged)
}

pub(crate) async fn abort_router_acp_session(
    ctx: &mut RouterAcpSessionCtx<'_>,
    err: String,
) -> Result<(), String> {
    agent_backend_set_run_timing(ctx.client, None);
    end_router_acp_session(ctx, Err(err)).await
}

#[cfg(test)]
#[path = "router_flow_acp_support_tests.rs"]
mod router_flow_acp_support_tests;

#[cfg(test)]
#[path = "router_flow_acp_support_kiss_cov_tests.rs"]
mod router_flow_acp_support_kiss_cov_tests;
