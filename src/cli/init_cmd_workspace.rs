use std::path::Path;

use super::Language;
use super::init_cmd_mid_core::write_text_file;
use super::TPL_ADVICE;
use super::TPL_CONFIG;

pub(super) fn ensure_malvin_workspace_layout(
    root: &Path,
    force: bool,
    languages: &[Language],
) -> Result<(), String> {
    std::fs::create_dir_all(crate::malvin_logs_root(root))
        .map_err(|e| format!("init: mkdir {}: {e}", crate::MALVIN_LOGS_REL))?;
    if languages.contains(&Language::Rust) {
        ensure_cargo_toml(root, force)?;
    }
    crate::repo_gates::ensure_default_malvin_checks_file(root)?;
    write_text_file(&crate::malvin_advice_path(root), TPL_ADVICE, force)?;
    write_text_file(&crate::malvin_config_path(root), TPL_CONFIG, force)?;
    Ok(())
}

fn ensure_cargo_toml(root: &Path, force: bool) -> Result<(), String> {
    let path = root.join("Cargo.toml");
    if path.is_file() && !force {
        return Ok(());
    }
    let name = cargo_package_name_from_root(root);
    let contents = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[profile.dev]
incremental = true
"#
    );
    write_text_file(&path, &contents, true)
}

fn cargo_package_name_from_root(root: &Path) -> String {
    let raw = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    let mut out = String::new();
    let mut prev_underscore = false;
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_underscore = false;
        } else if !prev_underscore {
            out.push('_');
            prev_underscore = true;
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "project".to_string()
    } else if trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("p_{trimmed}")
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_cargo_toml_respects_existing_file_without_force() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("Cargo.toml");
        std::fs::write(&p, "keep\n").unwrap();
        ensure_cargo_toml(tmp.path(), false).unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "keep\n");
    }

    #[test]
    fn ensure_cargo_toml_force_rewrites_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("Cargo.toml");
        std::fs::write(&p, "old\n").unwrap();
        ensure_cargo_toml(tmp.path(), true).unwrap();
        let toml = std::fs::read_to_string(&p).unwrap();
        assert!(toml.contains("[package]"));
        assert!(toml.contains("incremental = true"));
    }

    #[test]
    fn cargo_package_name_from_root_edge_cases() {
        assert_eq!(
            cargo_package_name_from_root(Path::new("/tmp/123")),
            "p_123"
        );
        assert_eq!(
            cargo_package_name_from_root(Path::new("/tmp/---")),
            "project"
        );
    }
}
