//! Prompt templates under `~/.malvin/prompts` with embedded defaults.

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

/// Prompt files on disk (`~/.malvin/prompts` by default).
#[derive(Debug, Clone)]
pub struct PromptStore {
    root: PathBuf,
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
    #[must_use]
    pub fn default_store() -> Self {
        Self {
            root: user_home_dir().join(".malvin").join("prompts"),
        }
    }

    #[must_use]
    pub const fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    /// Create `root` and copy any missing default prompts from embedded templates.
    ///
    /// # Errors
    ///
    /// Returns [`PromptError`] if a default file is missing from the binary or I/O fails.
    pub fn ensure_defaults(&self) -> Result<(), PromptError> {
        std::fs::create_dir_all(&self.root).map_err(|e| {
            PromptError(format!(
                "failed to create prompt directory {}: {e}",
                self.root.display()
            ))
        })?;
        for name in DEFAULT_PROMPTS {
            let target = self.root.join(name);
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

    /// Ensure all workflow prompts exist.
    ///
    /// # Errors
    ///
    /// Returns [`PromptError`] listing any missing required files.
    pub fn validate_required(&self) -> Result<(), PromptError> {
        let missing: Vec<&str> = REQUIRED_PROMPTS
            .iter()
            .copied()
            .filter(|n| !self.root.join(n).exists())
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        Err(PromptError(format!(
            "Missing required prompt files in {}: {}. Reinstall malvin or copy the missing files there.",
            self.root.display(),
            missing.join(", ")
        )))
    }

    /// Ensure prompts needed for standalone `malvin kpop` exist (`header.md`, `kpop_common.md`, `kpop_block.md`, and `learn.md` when learning runs).
    ///
    /// # Errors
    ///
    /// Returns [`PromptError`] listing any missing files.
    pub fn validate_kpop_prompts(
        &self,
        validation: KpopPromptValidation,
    ) -> Result<(), PromptError> {
        let mut missing: Vec<&str> = Vec::new();
        if !self.root.join(HEADER_MD).exists() {
            missing.push(HEADER_MD);
        }
        if !self.root.join("kpop_common.md").exists() {
            missing.push("kpop_common.md");
        }
        if !self.root.join("kpop_block.md").exists() {
            missing.push("kpop_block.md");
        }
        if validation.require_mbc2 && !self.root.join("mbc2_pure.md").exists() {
            missing.push("mbc2_pure.md");
        }
        if validation.run_learn && !self.root.join("learn.md").exists() {
            missing.push("learn.md");
        }
        if missing.is_empty() {
            return Ok(());
        }
        Err(PromptError(format!(
            "Missing required prompt files in {}: {}. Reinstall malvin or copy the missing files there.",
            self.root.display(),
            missing.join(", ")
        )))
    }

    /// Ensure a single file exists (e.g. `learn.md`).
    ///
    /// # Errors
    ///
    /// Returns [`PromptError`] if the file is missing.
    pub fn validate_exists(&self, filename: &str) -> Result<(), PromptError> {
        if self.root.join(filename).exists() {
            return Ok(());
        }
        Err(PromptError(format!(
            "Missing prompt file in {}: {filename}. Reinstall malvin or copy the missing file there.",
            self.root.display()
        )))
    }

    /// Load `filename`, substitute `{{ key }}` → `$key`, then substitute.
    /// The same expansion is applied to `header.md` and `coding_rules.md`; the results are concatenated
    /// (header first) and injected as `coding_rules` into the main template.
    ///
    /// # Errors
    ///
    /// Returns [`PromptError`] if the file cannot be read.
    pub fn render(
        &self,
        filename: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, PromptError> {
        let path = self.root.join(filename);
        let prompt_text = std::fs::read_to_string(&path).map_err(|_| {
            PromptError(format!(
                "Missing prompt file in {}: {filename}. Reinstall malvin or copy the missing file there.",
                self.root.display()
            ))
        })?;
        let mut render_context: HashMap<String, String> = context.clone();
        render_context.insert(
            "coding_rules".to_string(),
            merged_coding_rules(self, context)?,
        );
        let out = render_template(&prompt_text, &render_context);
        enforce_no_unresolved_braces(&out)?;
        Ok(out)
    }

    /// Expand a single prompt file with `context` (`{{ key }}` / `$key`) without injecting `coding_rules`.
    ///
    /// # Errors
    ///
    /// Returns [`PromptError`] if the file cannot be read.
    pub fn render_prompt_only(
        &self,
        filename: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, PromptError> {
        let path = self.root.join(filename);
        let prompt_text = std::fs::read_to_string(&path).map_err(|_| {
            PromptError(format!(
                "Missing prompt file in {}: {filename}. Reinstall malvin or copy the missing file there.",
                self.root.display()
            ))
        })?;
        let out = render_template(&prompt_text, context);
        enforce_no_unresolved_braces(&out)?;
        Ok(out)
    }

    pub(crate) fn load_coding_rules(&self) -> String {
        let p = self.root.join("coding_rules.md");
        std::fs::read_to_string(p)
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    pub(crate) fn load_header(&self) -> String {
        let p = self.root.join(HEADER_MD);
        std::fs::read_to_string(p)
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
