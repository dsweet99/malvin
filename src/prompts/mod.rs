//! Prompt templates under `~/.malvin/prompts` with embedded defaults.

mod defaults;
mod template;

use std::collections::HashMap;
use std::path::PathBuf;

use defaults::{DEFAULT_PROMPTS, REQUIRED_PROMPTS};
use template::merge_header_and_coding_rules;

pub(crate) use defaults::default_file;

/// User-facing prompt configuration error.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct PromptError(pub String);

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

    /// Ensure prompts needed for standalone `malvin kpop` exist (`header.md`, `kpop.md`, and `learn.md` when learning runs).
    ///
    /// # Errors
    ///
    /// Returns [`PromptError`] listing any missing files.
    pub fn validate_kpop_prompts(
        &self,
        run_learn: bool,
        p_creative: f64,
    ) -> Result<(), PromptError> {
        let mut missing: Vec<&str> = Vec::new();
        if !self.root.join("header.md").exists() {
            missing.push("header.md");
        }
        if !self.root.join("kpop.md").exists() {
            missing.push("kpop.md");
        }
        if crate::kpop_acp_prompt::kpop_creative_enabled(p_creative)
            && !self.root.join("mbc2.md").exists()
        {
            missing.push("mbc2.md");
        }
        if run_learn && !self.root.join("learn.md").exists() {
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
        let header_raw = self.load_header();
        let header_expanded = render_template(&header_raw, &render_context);
        let rules_raw = self.load_coding_rules();
        let rules_expanded = render_template(&rules_raw, &render_context);
        let merged = merge_header_and_coding_rules(&header_expanded, &rules_expanded);
        render_context.insert("coding_rules".to_string(), merged);
        Ok(render_template(&prompt_text, &render_context))
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
        Ok(render_template(&prompt_text, context))
    }

    fn load_coding_rules(&self) -> String {
        let p = self.root.join("coding_rules.md");
        std::fs::read_to_string(p)
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    fn load_header(&self) -> String {
        let p = self.root.join("header.md");
        std::fs::read_to_string(p)
            .unwrap_or_default()
            .trim()
            .to_string()
    }
}

#[allow(unused_imports)]
// `substitute_template`: tests / coverage only (not used in this module body).
pub(crate) use template::{render_template, substitute_template};

#[cfg(test)]
#[allow(unsafe_code)]
mod tests;
