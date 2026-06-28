//! `malvin init` — install templates and bootstrap local tooling.

pub(crate) const TPL_GITIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/gitignore"
));
pub(crate) const TPL_KISSIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/kissignore"
));
pub(crate) const TPL_ADVICE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/advice.md"
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
#[path = "init_cmd_mid_core.rs"]
mod init_cmd_mid_core;

#[path = "init_cmd_bootstrap.rs"]
mod init_cmd_bootstrap;

#[path = "init_cmd_workspace.rs"]
mod init_cmd_workspace;

#[cfg(test)]
#[path = "init_cmd_mid_tests.rs"]
mod init_cmd_mid_tests;

use std::path::PathBuf;

use clap::Args;
use init_cmd_mid_core::{bootstrap_repo_tooling, resolve_init_root, write_init_templates};
use init_cmd_workspace::ensure_malvin_workspace_layout;
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
    let checks_existed_before = crate::malvin_checks_path(&root).is_file();
    let artifacts = init_cmd_mid_core::emit_init_startup(&root, req.opts.tee_startup_stdout)?;
    crate::cli::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    let discovery_decision = crate::repo_gates::init_discovery::init_discovery_decision(
        &root,
        crate::repo_gates::init_discovery::InitDiscoveryRequest {
            checks_existed_before_bootstrap: checks_existed_before,
            force_overwrite: req.opts.overwrite_templates,
        },
    );
    let r = async {
        write_init_templates(&root, req.opts.overwrite_templates, &languages)?;
        init_cmd_bootstrap::ensure_git_repo(&root)?;
        ensure_malvin_workspace_layout(&root, req.opts.overwrite_templates, &languages)?;
        bootstrap_repo_tooling(&root)?;
        if discovery_decision.run && req.opts.overwrite_templates && checks_existed_before {
            crate::repo_gates::refresh_provisional_malvin_checks_file(&root)?;
        }
        if discovery_decision.run {
            crate::cli::init_discovery_flow::run_init_discovery_kpop(req.shared, &artifacts)
                .await
                .map(|_| ())
        } else {
            crate::cli::init_discovery_flow::emit_init_discovery_skip(discovery_decision);
            Ok(())
        }
    }
    .await;
    if r.is_ok() {
        crate::cli::error_run_log::clear_command_error_run_dir();
    }
    r
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

#[cfg(test)]
mod run_init_tests {
    use super::*;
    use crate::cli::SharedOpts;

    fn test_shared_opts() -> SharedOpts {
        SharedOpts::test_defaults()
    }

    #[test]
    fn run_init_options_expose_force_and_tee_flags() {
        let opts = RunInitOptions {
            overwrite_templates: true,
            tee_startup_stdout: false,
        };
        assert!(opts.overwrite_templates);
        assert!(!opts.tee_startup_stdout);
    }

    #[tokio::test]
    async fn run_init_rejects_empty_languages() {
        let shared = test_shared_opts();
        let languages: Vec<String> = vec![];
        let err = run_init(RunInitRequest {
            path: None,
            languages: &languages,
            shared: &shared,
            opts: RunInitOptions {
                overwrite_templates: false,
                tee_startup_stdout: false,
            },
        })
        .await
        .unwrap_err();
        assert!(err.contains("At least one language"));
    }
}
