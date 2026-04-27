//! Prompt templates sourced from embedded defaults, with optional custom root.

mod defaults;
mod template;

use std::collections::HashMap;
use std::path::PathBuf;

use defaults::{DEFAULT_PROMPTS, REQUIRED_PROMPTS};

pub use defaults::{DO_HEADER_MD, HEADER_MD};

pub(crate) use defaults::default_file;

pub fn enforce_no_unresolved_braces(text: &str) -> Result<(), PromptError> {
    if text.contains("{{") {
        Err(PromptError(
            "prompt still contains \"{{\" before ACP; resolve every {{ key }} placeholder"
                .to_string(),
        ))
    } else {
        Ok(())
    }
}

/// User-facing prompt configuration error.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct PromptError(pub String);

/// Inputs for [`PromptStore::validate_kpop_prompts`].
#[derive(Debug, Clone, Copy)]
pub struct KpopPromptValidation {
    pub run_learn: bool,
    pub require_mbc2: bool,
}

#[derive(Debug, Clone)]
pub struct PromptStore {
    root: Option<PathBuf>,
}

pub(crate) fn user_home_dir() -> PathBuf {
    if let Some(h) = std::env::var_os("HOME").filter(|s| !s.is_empty()) {
        return PathBuf::from(h);
    }
    if let Some(h) = std::env::var_os("USERPROFILE").filter(|s| !s.is_empty()) {
        return PathBuf::from(h);
    }
    PathBuf::from(".")
}

impl PromptStore {
    fn prompt_text(&self, filename: &str) -> Result<String, PromptError> {
        self.root.as_ref().map_or_else(
            || {
                default_file(filename)
                    .map(str::to_string)
                    .ok_or_else(|| PromptError(format!("Missing embedded prompt in binary: {filename}")))
            },
            |root| {
                std::fs::read_to_string(root.join(filename)).map_err(|_| {
                    PromptError(format!(
                        "Missing prompt file in {}: {filename}. Reinstall malvin or copy the missing file there.",
                        root.display()
                    ))
                })
            },
        )
    }

    fn prompt_source_desc(&self) -> String {
        self.root
            .as_ref()
            .map_or_else(|| "embedded prompts".to_string(), |root| root.display().to_string())
    }
}

impl PromptStore {
    #[must_use]
    pub const fn default_store() -> Self {
        Self { root: None }
    }

    #[must_use]
    pub const fn with_root(root: PathBuf) -> Self {
        Self { root: Some(root) }
    }

    pub fn ensure_defaults(&self) -> Result<(), PromptError> {
        let Some(root) = &self.root else {
            return Ok(());
        };
        std::fs::create_dir_all(root).map_err(|e| {
            PromptError(format!(
                "failed to create prompt directory {}: {e}",
                root.display()
            ))
        })?;
        for name in DEFAULT_PROMPTS {
            let target = root.join(name);
            if target.exists() {
                continue;
            }
            let body = default_file(name).ok_or_else(|| {
                PromptError(format!("internal: missing embedded default for {name}"))
            })?;
            std::fs::write(&target, body).map_err(|e| {
                PromptError(format!(
                    "failed to write default prompt {}: {e}",
                    target.display()
                ))
            })?;
        }
        Ok(())
    }

    pub fn validate_required(&self) -> Result<(), PromptError> {
        let has_file = |name: &str| -> bool {
            self.root
                .as_ref()
                .map_or_else(|| default_file(name).is_some(), |root| root.join(name).exists())
        };
        let missing: Vec<&str> = REQUIRED_PROMPTS
            .iter()
            .copied()
            .filter(|name| !has_file(name))
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        Err(PromptError(format!(
            "Missing required prompt files in {}: {}. Reinstall malvin or copy the missing files there.",
            self.prompt_source_desc(),
            missing.join(", ")
        )))
    }

    pub fn validate_kpop_prompts(
        &self,
        validation: KpopPromptValidation,
    ) -> Result<(), PromptError> {
        let mut missing: Vec<&str> = Vec::new();
        if self.prompt_text(HEADER_MD).is_err() {
            missing.push(HEADER_MD);
        }
        if self.prompt_text("kpop_common.md").is_err() {
            missing.push("kpop_common.md");
        }
        if self.prompt_text("kpop_block.md").is_err() {
            missing.push("kpop_block.md");
        }
        if validation.require_mbc2 && self.prompt_text("mbc2_pure.md").is_err() {
            missing.push("mbc2_pure.md");
        }
        if validation.run_learn && self.prompt_text("learn.md").is_err() {
            missing.push("learn.md");
        }
        if missing.is_empty() {
            return Ok(());
        }
        Err(PromptError(format!(
            "Missing required prompt files in {}: {}. Reinstall malvin or copy the missing files there.",
            self.prompt_source_desc(),
            missing.join(", ")
        )))
    }

    pub fn validate_exists(&self, filename: &str) -> Result<(), PromptError> {
        if self.prompt_text(filename).is_ok() {
            return Ok(());
        }
        Err(PromptError(format!(
            "Missing prompt file in {}: {filename}. Reinstall malvin or copy the missing file there.",
            self.prompt_source_desc()
        )))
    }

    pub fn render(
        &self,
        filename: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, PromptError> {
        let prompt_text = self.prompt_text(filename)?;
        let mut render_context: HashMap<String, String> = context.clone();
        render_context.insert(
            "coding_rules".to_string(),
            merged_coding_rules(self, context)?,
        );
        let out = render_template(&prompt_text, &render_context);
        enforce_no_unresolved_braces(&out)?;
        Ok(out)
    }

    pub fn render_prompt_only(
        &self,
        filename: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, PromptError> {
        let prompt_text = self.prompt_text(filename)?;
        let out = render_template(&prompt_text, context);
        enforce_no_unresolved_braces(&out)?;
        Ok(out)
    }

    pub(crate) fn load_coding_rules(&self) -> String {
        self.prompt_text("coding_rules.md")
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    pub(crate) fn load_header(&self) -> String {
        self.prompt_text(HEADER_MD)
            .unwrap_or_default()
            .trim()
            .to_string()
    }
}

pub use template::{
    merge_header_and_coding_rules, merged_coding_rules, render_mbc2_for_scheduled_kpop_block,
    render_template, substitute_template,
};

#[cfg(test)]
#[allow(unsafe_code)]
mod tests;
#[cfg(test)]
mod embedded_defaults_tests;
#[cfg(test)]
mod check_sync_tests;
