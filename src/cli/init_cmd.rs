//! `malvin init` — install templates and bootstrap local tooling.

pub(crate) const TPL_GITIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/gitignore"
));
pub(crate) const TPL_KISSIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/kissignore"
));
pub(crate) const ADMIN_CHECK_UNTRACKED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/admin/check_untracked.sh"
));
pub(crate) const PRE_COMMIT_HEADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/hooks/header.yaml"
));
pub(crate) const HOOK_RUFF: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/hooks/ruff.yaml"
));
pub(crate) const HOOK_CLIPPY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/hooks/clippy.yaml"
));
pub(crate) const HOOK_KISS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/hooks/kiss.yaml"
));
pub(crate) const HOOK_UNTRACKED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/hooks/untracked.yaml"
));
pub(crate) const TPL_STYLE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/llm_style/style.md"
));

use std::path::{Path, PathBuf};

use clap::Args;
/// Supported languages for `malvin init`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Python,
    Rust,
}

impl Language {
    fn from_str_case_insensitive(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "python" => Some(Self::Python),
            "rust" => Some(Self::Rust),
            _ => None,
        }
    }
}

/// `--force` overwrites files installed from `default_repo/` and refreshes `admin/check_untracked.sh`.
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Overwrite `default_repo/` installs; refresh `admin/check_untracked.sh`.
    #[arg(long, default_value_t = false)]
    pub force: bool,
    /// Languages to support (python, rust). At least one required unless `--doc`.
    pub languages: Vec<String>,
    /// Target directory [default: cwd].
    #[arg(long)]
    pub path: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug)]
pub struct RunInitOptions {
    pub overwrite_templates: bool,
    pub tee_startup_stdout: bool,
}

pub struct RunInitRequest<'a> {
    pub path: Option<PathBuf>,
    pub languages: &'a [String],
    pub shared: &'a crate::cli::SharedOpts,
    pub opts: RunInitOptions,
}

pub async fn run_init(req: RunInitRequest<'_>) -> Result<(), String> {
    let languages = parse_languages(req.languages)?;
    let root = resolve_init_root(req.path)?;
    let artifacts = emit_init_startup(&root, req.opts.tee_startup_stdout)?;
    crate::cli::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    let r = async {
        write_init_templates(
            &root,
            req.opts.overwrite_templates,
            &languages,
        )?;
        bootstrap_repo_tooling(&root)?;
        run_init_summary_phase(req.shared, &artifacts).await
    }
    .await;
    if r.is_ok() {
        crate::cli::error_run_log::clear_command_error_run_dir();
    }
    r
}

fn emit_init_startup(
    root: &Path,
    tee_startup_stdout: bool,
) -> Result<crate::artifacts::RunArtifacts, String> {
    use crate::artifacts::create_run_artifacts_from_text;
    let artifacts =
        create_run_artifacts_from_text("init", Some(root)).map_err(|e| format!("init: {e}"))?;
    crate::cli::run_emit::emit_run_startup_sequence(&artifacts, tee_startup_stdout, "init")?;
    Ok(artifacts)
}

fn init_summary_combined_body(
    store: &crate::prompts::PromptStore,
    ctx: &std::collections::HashMap<String, String>,
) -> Result<String, String> {
    use crate::prompts::{HEADER_MD, PromptError};
    let header_body = store
        .render_prompt_only(HEADER_MD, ctx)
        .map_err(|e: PromptError| e.0)?;
    let summary_only = store
        .render("summary.md", ctx)
        .map_err(|e: PromptError| e.0)?;
    Ok(format!(
        "{}\n\n{}",
        header_body.trim_end(),
        summary_only.trim_end()
    ))
}

async fn init_summary_coder_turn_with_timing_emit(
    client: &mut crate::acp::AgentClient,
    artifacts: &crate::artifacts::RunArtifacts,
    body: &str,
) -> Result<(), String> {
    use crate::run_timing::TimingPhase;
    let timing = client.attach_run_timing_for_session();
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_implement_display_name("init");
    let begin_res = client.begin_coder_session(&artifacts.work_dir).await;
    if let Err(e) = begin_res {
        client.set_run_timing(None);
        return Err(e.to_string());
    }
    let prompt_res = client
        .run_coder_prompt(
            body,
            &artifacts.log_path("summary"),
            "summary",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(TimingPhase::Summary),
                skip_repo_style: true,
                do_trace_split: None,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string());
    let end_res = client.end_coder_session().await.map_err(|e| e.to_string());
    let merged = crate::acp_post_run::prefer_primary_over_secondary(
        prompt_res,
        end_res,
        "failed to end coder session",
    );
    crate::acp_post_run::emit_run_timing_after_acp(client, &artifacts.run_dir, &timing, merged)
}

async fn run_init_summary_phase(
    shared: &crate::cli::SharedOpts,
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<(), String> {
    use crate::orchestrator::workflow_context;
    use crate::prompts::{PromptError, PromptStore};
    let workflow = crate::cli::WorkflowCliOptions {
        force: !shared.no_force,
        run_learn: false,
    };
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("summary.md")
        .map_err(|e: PromptError| e.0)?;
    let ctx = workflow_context(artifacts, &store, "init").map_err(|e: PromptError| e.0)?;
    let session_dotfile_backups =
        crate::artifacts::SessionDotfileBackups::snapshot(&artifacts.work_dir)?;
    let mut client =
        crate::cli::build_agent(shared, workflow, shared.acp_stdout_markdown_enabled());
    client
        .ensure_authenticated()
        .map_err(|e: crate::acp::AuthError| e.to_string())?;
    client.prompts_log_run_dir = Some(artifacts.run_dir.clone());
    let body = init_summary_combined_body(&store, &ctx)?;
    let coder_turn_out =
        init_summary_coder_turn_with_timing_emit(&mut client, artifacts, &body).await;
    crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        coder_turn_out,
        &artifacts.work_dir,
        &session_dotfile_backups,
        &artifacts.artifact_result_md(),
    )
}

pub fn parse_languages(args: &[String]) -> Result<Vec<Language>, String> {
    if args.is_empty() {
        return Err("At least one language is required. Supported: python, rust".to_string());
    }
    let mut languages = Vec::new();
    for arg in args {
        match Language::from_str_case_insensitive(arg) {
            Some(lang) => {
                if !languages.contains(&lang) {
                    languages.push(lang);
                }
            }
            None => return Err(format!("Unknown language '{arg}'. Supported: python, rust")),
        }
    }
    Ok(languages)
}

use std::process::Command;

use crate::{lookup_bin_on_path, require_kiss_for_malvin};


include!("init_cmd_mid_core.inc");
include!("init_cmd_mid_tests.inc");
