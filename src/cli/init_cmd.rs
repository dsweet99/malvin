//! `malvin init` — install templates and bootstrap local tooling.

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Args;

use malvin::env_path::{lookup_bin_on_path, require_kiss_for_malvin};

const TPL_GITIGNORE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default_repo/gitignore"));
const TPL_KISSIGNORE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default_repo/kissignore"));
const TPL_GROUNDING: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default_repo/grounding.md"));
const ADMIN_CHECK_UNTRACKED: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default_repo/admin/check_untracked.sh"));
const PRE_COMMIT_HEADER: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default_repo/hooks/header.yaml"));
const HOOK_RUFF: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default_repo/hooks/ruff.yaml"));
const HOOK_CLIPPY: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default_repo/hooks/clippy.yaml"));
const HOOK_KISS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default_repo/hooks/kiss.yaml"));
const HOOK_UNTRACKED: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default_repo/hooks/untracked.yaml"));

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

    const fn display_name(self) -> &'static str {
        match self {
            Self::Python => "Python",
            Self::Rust => "Rust",
        }
    }
}

/// Arguments for [`run_init`].
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

/// `--force` overwrites files installed from `default_repo/` and refreshes `admin/check_untracked.sh`.
pub fn run_init(
    path: Option<PathBuf>,
    force: bool,
    language_args: &[String],
) -> Result<(), String> {
    let languages = parse_languages(language_args)?;
    let root = resolve_init_root(path)?;
    write_init_templates(&root, force, &languages)?;
    bootstrap_repo_tooling(&root)
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

fn format_languages_for_grounding(languages: &[Language]) -> String {
    match languages.len() {
        0 => String::new(),
        1 => format!("in {}", languages[0].display_name()),
        _ => {
            let names: Vec<&str> = languages.iter().map(|l| l.display_name()).collect();
            format!("in {} and {}", names[..names.len() - 1].join(", "), names.last().unwrap())
        }
    }
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
    write_text_file(&root.join(".pre-commit-config.yaml"), &pre_commit_config, force)?;
    let grounding = TPL_GROUNDING.replace("{{languages}}", &format_languages_for_grounding(languages));
    write_text_file(&root.join("grounding.md"), &grounding, force)?;
    let admin_dir = root.join("admin");
    std::fs::create_dir_all(&admin_dir).map_err(|e| format!("init: mkdir admin: {e}"))?;
    write_shell_script(&admin_dir.join("check_untracked.sh"), ADMIN_CHECK_UNTRACKED, force)
}

fn bootstrap_repo_tooling(root: &Path) -> Result<(), String> {
    require_on_path("pre-commit", "`pre-commit` is not installed; run `pip install pre-commit`.")?;
    run_command_expect_success(Command::new("pre-commit").arg("install").current_dir(root), "`pre-commit install` failed.")?;
    require_kiss_for_malvin("init")?;
    run_command_expect_success(Command::new("kiss").arg("init").current_dir(root), "`kiss init` failed.")?;
    install_git_lfs(root)
}

fn require_on_path(bin: &str, err: &str) -> Result<(), String> {
    if lookup_bin_on_path(bin).is_none() { return Err(err.to_string()); }
    Ok(())
}

fn install_git_lfs(root: &Path) -> Result<(), String> {
    let err = "`git lfs` is not available. Install Git LFS so `git lfs version` succeeds.";
    let status = Command::new("git").args(["lfs", "version"]).current_dir(root).status().map_err(|_| err.to_string())?;
    if !status.success() { return Err(err.to_string()); }
    run_command_expect_success(Command::new("git").args(["lfs", "install"]).current_dir(root), "`git lfs install` failed.")
}

fn run_command_expect_success(cmd: &mut Command, err: &str) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("{err} ({e})"))?;
    if status.success() { Ok(()) } else { Err(err.to_string()) }
}

fn write_text_file(path: &Path, contents: &str, force: bool) -> Result<(), String> {
    if path.exists() && !force { return Ok(()); }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("init: mkdir {}: {e}", parent.display()))?;
    }
    std::fs::write(path, contents).map_err(|e| format!("init: write {}: {e}", path.display()))
}

fn write_shell_script(path: &Path, contents: &str, force: bool) -> Result<(), String> {
    if path.exists() && !force { return Ok(()); }
    write_text_file(path, contents, force)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path).map_err(|e| format!("init: stat {}: {e}", path.display()))?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).map_err(|e| format!("init: chmod {}: {e}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_templates_are_non_empty() {
        assert!(!TPL_GITIGNORE.trim().is_empty());
        assert!(ADMIN_CHECK_UNTRACKED.contains("check_untracked"));
    }

    #[test]
    fn parse_languages_valid() {
        assert_eq!(parse_languages(&["python".to_string()]).unwrap(), vec![Language::Python]);
        assert_eq!(parse_languages(&["RUST".to_string()]).unwrap(), vec![Language::Rust]);
        assert_eq!(parse_languages(&["Python".to_string(), "rust".to_string()]).unwrap(), vec![Language::Python, Language::Rust]);
    }

    #[test]
    fn parse_languages_deduplicates() {
        assert_eq!(parse_languages(&["python".to_string(), "PYTHON".to_string()]).unwrap(), vec![Language::Python]);
    }

    #[test]
    fn parse_languages_rejects_unknown() {
        assert!(parse_languages(&["javascript".to_string()]).unwrap_err().contains("Unknown language"));
    }

    #[test]
    fn parse_languages_rejects_empty() {
        assert!(parse_languages(&[]).unwrap_err().contains("At least one language"));
    }

    #[test]
    fn format_languages_single() {
        assert_eq!(format_languages_for_grounding(&[Language::Python]), "in Python");
        assert_eq!(format_languages_for_grounding(&[Language::Rust]), "in Rust");
    }

    #[test]
    fn format_languages_multiple() {
        assert_eq!(format_languages_for_grounding(&[Language::Python, Language::Rust]), "in Python and Rust");
        assert_eq!(format_languages_for_grounding(&[Language::Rust, Language::Python]), "in Rust and Python");
    }

    #[test]
    fn pre_commit_config_python_only() {
        let config = build_pre_commit_config(&[Language::Python]);
        assert!(config.contains("ruff") && !config.contains("clippy") && config.contains("kiss") && config.contains("check-untracked"));
    }

    #[test]
    fn pre_commit_config_rust_only() {
        let config = build_pre_commit_config(&[Language::Rust]);
        assert!(!config.contains("ruff") && config.contains("clippy") && config.contains("kiss") && config.contains("check-untracked"));
    }

    #[test]
    fn pre_commit_config_both_languages() {
        let config = build_pre_commit_config(&[Language::Python, Language::Rust]);
        assert!(config.contains("ruff") && config.contains("clippy") && config.contains("kiss") && config.contains("check-untracked"));
    }

    #[test]
    fn kiss_stringify_init_cmd() {
        let _ = (stringify!(InitArgs), stringify!(Language), stringify!(run_init), stringify!(parse_languages));
        let _ = (stringify!(format_languages_for_grounding), stringify!(build_pre_commit_config), stringify!(resolve_init_root));
        let _ = (stringify!(write_init_templates), stringify!(bootstrap_repo_tooling), stringify!(require_on_path));
        let _ = (stringify!(install_git_lfs), stringify!(run_command_expect_success), stringify!(write_text_file), stringify!(write_shell_script));
    }
}
