//! `malvin init` — install templates and bootstrap local tooling.

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Args;
use malvin::acp::CoderPromptOptions;
use malvin::artifacts::{
    backup_workspace_kissconfig_if_present, backup_workspace_kissignore_if_present,
    backup_workspace_malvin_checks_if_present, create_run_artifacts_from_text,
};
use malvin::env_path::{lookup_bin_on_path, require_kiss_for_malvin};
use malvin::orchestrator::workflow_context;
use malvin::prompts::{HEADER_MD, PromptError, PromptStore};
use malvin::run_timing::TimingPhase;

const TPL_GITIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/gitignore"
));
const TPL_KISSIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/kissignore"
));
const ADMIN_CHECK_UNTRACKED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/admin/check_untracked.sh"
));
const PRE_COMMIT_HEADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/hooks/header.yaml"
));
const HOOK_RUFF: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/hooks/ruff.yaml"
));
const HOOK_CLIPPY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/hooks/clippy.yaml"
));
const HOOK_KISS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/hooks/kiss.yaml"
));
const HOOK_UNTRACKED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/hooks/untracked.yaml"
));
const TPL_STYLE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/llm_style/style.md"
));

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
    /// Languages to support (python, rust). At least one required.
    #[arg(required = true)]
    pub languages: Vec<String>,
    /// Target directory [default: cwd].
    #[arg(long)]
    pub path: Option<PathBuf>,
}

pub async fn run_init(
    path: Option<PathBuf>,
    force: bool,
    language_args: &[String],
    shared: &super::SharedOpts,
    tee_startup_stdout: bool,
) -> Result<(), String> {
    let languages = parse_languages(language_args)?;
    let root = resolve_init_root(path)?;
    let artifacts = emit_init_startup(&root, tee_startup_stdout)?;
    super::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    let r = async {
        write_init_templates(&root, force, &languages)?;
        bootstrap_repo_tooling(&root)?;
        run_init_summary_phase(shared, &artifacts).await
    }
    .await;
    if r.is_ok() {
        super::error_run_log::clear_command_error_run_dir();
    }
    r
}

fn emit_init_startup(
    root: &Path,
    tee_startup_stdout: bool,
) -> Result<malvin::artifacts::RunArtifacts, String> {
    let artifacts =
        create_run_artifacts_from_text("init", Some(root)).map_err(|e| format!("init: {e}"))?;
    super::run_emit::emit_run_startup_sequence(&artifacts, tee_startup_stdout, "init")?;
    Ok(artifacts)
}

async fn run_init_summary_phase(
    shared: &super::SharedOpts,
    artifacts: &malvin::artifacts::RunArtifacts,
) -> Result<(), String> {
    let workflow = super::WorkflowCliOptions {
        force: !shared.no_force,
        run_learn: false,
    };
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("summary.md")
        .map_err(|e: PromptError| e.0)?;
    let ctx = workflow_context(artifacts, &store, "init").map_err(|e: PromptError| e.0)?;
    let malvin_checks_backup = backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;
    let kissconfig_backup = backup_workspace_kissconfig_if_present(&artifacts.work_dir)?;
    let kissignore_backup = backup_workspace_kissignore_if_present(&artifacts.work_dir)?;
    let session_dotfile_backups = malvin::artifacts::SessionDotfileBackups::from_parts(
        kissconfig_backup,
        malvin_checks_backup,
        kissignore_backup,
    );
    let mut client = super::build_agent(shared, workflow, shared.acp_stdout_markdown_enabled());
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    client.prompts_log_run_dir = Some(artifacts.run_dir.clone());
    let header_body = store
        .render_prompt_only(HEADER_MD, &ctx)
        .map_err(|e: PromptError| e.0)?;
    let summary_only = store
        .render("summary.md", &ctx)
        .map_err(|e: PromptError| e.0)?;
    let body = format!("{}\n\n{}", header_body.trim_end(), summary_only.trim_end());
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
            &body,
            &artifacts.log_path("summary"),
            "summary",
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::Summary),
                skip_repo_style: true,
                do_trace_split: None,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string());
    let end_res = client.end_coder_session().await.map_err(|e| e.to_string());
    let merged = super::timing_merge::prefer_primary_over_secondary(
        prompt_res,
        end_res,
        "failed to end coder session",
    );
    let timing_out = super::timing_merge::emit_run_timing_after_acp(
        &mut client,
        &artifacts.run_dir,
        &timing,
        merged,
    );
    super::timing_merge::merge_acp_with_workspace_session_restore_and_check_abort(
        timing_out,
        &artifacts.work_dir,
        &session_dotfile_backups,
        &artifacts.artifact_result_md(),
    )
}

fn parse_languages(args: &[String]) -> Result<Vec<Language>, String> {
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

fn build_pre_commit_config(languages: &[Language]) -> String {
    let mut config = PRE_COMMIT_HEADER.to_string();
    if languages.contains(&Language::Python) {
        config.push_str(HOOK_RUFF);
    }
    if languages.contains(&Language::Rust) {
        config.push_str(HOOK_CLIPPY);
    }
    config.push_str(HOOK_KISS);
    config.push_str(HOOK_UNTRACKED);
    config
}

fn resolve_init_root(path: Option<PathBuf>) -> Result<PathBuf, String> {
    let root = path.map_or_else(|| std::env::current_dir().map_err(|e| e.to_string()), Ok)?;
    if !root.exists() {
        std::fs::create_dir_all(&root)
            .map_err(|e| format!("init: create directory {}: {e}", root.display()))?;
    }
    Ok(root)
}

fn write_init_templates(root: &Path, force: bool, languages: &[Language]) -> Result<(), String> {
    write_text_file(&root.join(".gitignore"), TPL_GITIGNORE, force)?;
    write_text_file(&root.join(".kissignore"), TPL_KISSIGNORE, force)?;
    let pre_commit_config = build_pre_commit_config(languages);
    write_text_file(
        &root.join(".pre-commit-config.yaml"),
        &pre_commit_config,
        force,
    )?;
    let admin_dir = root.join("admin");
    std::fs::create_dir_all(&admin_dir).map_err(|e| format!("init: mkdir admin: {e}"))?;
    write_shell_script(
        &admin_dir.join("check_untracked.sh"),
        ADMIN_CHECK_UNTRACKED,
        force,
    )?;
    write_text_file(
        &root.join(".malvin_memory").join("style.md"),
        TPL_STYLE,
        force,
    )
}

fn bootstrap_repo_tooling(root: &Path) -> Result<(), String> {
    require_on_path(
        "pre-commit",
        "`pre-commit` is not installed; run `pip install pre-commit`.",
    )?;
    run_command_expect_success(
        Command::new("pre-commit").arg("install").current_dir(root),
        "`pre-commit install` failed.",
    )?;
    require_kiss_for_malvin("init")?;
    run_command_expect_success(
        Command::new("kiss").arg("init").current_dir(root),
        "`kiss init` failed.",
    )?;
    install_git_lfs(root)?;
    create_initial_commit(root)
}

fn create_initial_commit(root: &Path) -> Result<(), String> {
    if repo_already_has_commits(root) {
        return Ok(());
    }
    run_command_expect_success(
        Command::new("git").args(["add", "."]).current_dir(root),
        "`git add .` failed.",
    )?;
    let has_staged = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(root)
        .status()
        .is_ok_and(|s| !s.success());
    if has_staged {
        eprintln!(
            "init: creating initial commit (skipping pre-commit hooks to avoid bootstrap cycle)"
        );
        run_command_expect_success(
            Command::new("git")
                .args([
                    "-c",
                    "user.name=malvin",
                    "-c",
                    "user.email=malvin@localhost",
                ])
                .args([
                    "commit",
                    "--no-verify",
                    "-m",
                    "Initial commit from malvin init",
                ])
                .current_dir(root),
            "`git commit` failed.",
        )?;
        ensure_branch_is_main(root)?;
    }
    Ok(())
}

fn repo_already_has_commits(root: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .is_ok_and(|o| o.status.success())
}

fn ensure_branch_is_main(root: &Path) -> Result<(), String> {
    let current = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("`git branch --show-current` failed: {e}"))?;
    let branch = String::from_utf8_lossy(&current.stdout);
    if branch.trim() == "main" {
        return Ok(());
    }
    run_command_expect_success(
        Command::new("git")
            .args(["branch", "-M", "main"])
            .current_dir(root),
        "`git branch -M main` failed.",
    )
}

fn require_on_path(bin: &str, err: &str) -> Result<(), String> {
    if lookup_bin_on_path(bin).is_none() {
        return Err(err.to_string());
    }
    Ok(())
}

fn install_git_lfs(root: &Path) -> Result<(), String> {
    let err = "`git lfs` is not available. Install Git LFS so `git lfs version` succeeds.";
    let status = Command::new("git")
        .args(["lfs", "version"])
        .current_dir(root)
        .status()
        .map_err(|_| err.to_string())?;
    if !status.success() {
        return Err(err.to_string());
    }
    run_command_expect_success(
        Command::new("git")
            .args(["lfs", "install"])
            .current_dir(root),
        "`git lfs install` failed.",
    )
}

fn run_command_expect_success(cmd: &mut Command, err: &str) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("{err} ({e})"))?;
    if status.success() {
        Ok(())
    } else {
        Err(err.to_string())
    }
}

fn write_text_file(path: &Path, contents: &str, force: bool) -> Result<(), String> {
    if path.exists() && !force {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("init: mkdir {}: {e}", parent.display()))?;
    }
    std::fs::write(path, contents).map_err(|e| format!("init: write {}: {e}", path.display()))
}

fn write_shell_script(path: &Path, contents: &str, force: bool) -> Result<(), String> {
    if path.exists() && !force {
        return Ok(());
    }
    write_text_file(path, contents, force)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)
            .map_err(|e| format!("init: stat {}: {e}", path.display()))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)
            .map_err(|e| format!("init: chmod {}: {e}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_are_nonempty() {
        assert!(!TPL_GITIGNORE.trim().is_empty());
        assert!(
            ADMIN_CHECK_UNTRACKED.starts_with("#!/bin/bash\n"),
            "check_untracked.sh must have a bash shebang for pre-commit exec"
        );
        assert!(ADMIN_CHECK_UNTRACKED.contains("check_untracked"));
        assert!(ADMIN_CHECK_UNTRACKED.contains("exclude-standard"));
        assert!(!TPL_STYLE.trim().is_empty());
    }

    #[test]
    fn parse_languages_accepts_valid_languages() {
        assert_eq!(
            parse_languages(&["python".into()]).unwrap(),
            vec![Language::Python]
        );
        assert_eq!(
            parse_languages(&["Python".into(), "rust".into()]).unwrap(),
            vec![Language::Python, Language::Rust]
        );
    }

    #[test]
    fn parse_languages_rejects_invalid() {
        assert!(parse_languages(&["javascript".into()]).is_err());
        assert!(parse_languages(&[]).is_err());
    }

    #[test]
    fn build_pre_commit_config_includes_correct_hooks() {
        let py = build_pre_commit_config(&[Language::Python]);
        assert!(py.contains("ruff"));
        assert!(!py.contains("clippy"));
        assert!(py.contains("kiss"));
        let both = build_pre_commit_config(&[Language::Python, Language::Rust]);
        assert!(both.contains("ruff"));
        assert!(both.contains("clippy"));
    }

    #[test]
    fn resolve_init_root_creates_nested_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("a").join("b");
        assert!(resolve_init_root(Some(nested.clone())).is_ok());
        assert!(nested.exists());
    }

    #[test]
    fn require_on_path_finds_existing_binary() {
        require_on_path("ls", "e").unwrap();
    }

    #[test]
    fn require_on_path_fails_for_missing_binary() {
        assert!(require_on_path("nonexistent_xyz_binary_12345", "e").is_err());
    }

    #[test]
    fn run_command_expect_success_detects_failure() {
        run_command_expect_success(&mut Command::new("true"), "ok").unwrap();
        assert!(run_command_expect_success(&mut Command::new("false"), "f").is_err());
    }

    #[test]
    fn write_text_file_respects_force_flag() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("sub").join("f.txt");
        write_text_file(&path, "hello", false).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
        std::fs::write(&path, "orig").unwrap();
        write_text_file(&path, "new", false).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "orig");
        write_text_file(&path, "new", true).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    }

    #[test]
    #[cfg(unix)]
    fn write_shell_script_sets_executable_bit() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let script = tmp.path().join("s.sh");
        write_shell_script(&script, "#!/bin/sh", false).unwrap();
        assert!(std::fs::metadata(&script).unwrap().permissions().mode() & 0o111 != 0);
    }

    #[test]
    fn ensure_branch_is_main_renames_to_main() {
        let tmp = tempfile::tempdir().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(["commit", "--allow-empty", "-m", "i"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["branch", "-M", "main"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        ensure_branch_is_main(tmp.path()).unwrap();
        Command::new("git")
            .args(["branch", "-M", "master"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        ensure_branch_is_main(tmp.path()).unwrap();
        let branch = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&branch.stdout).trim(), "main");
    }

    #[test]
    fn install_git_lfs_succeeds_when_available() {
        let lfs_available = Command::new("git")
            .args(["lfs", "version"])
            .status()
            .is_ok_and(|s| s.success());
        if !lfs_available {
            eprintln!("test skipped: git-lfs not installed");
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        install_git_lfs(tmp.path()).unwrap();
    }
}
