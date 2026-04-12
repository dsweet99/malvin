//! Prompt templates under `~/.malvin/prompts` with embedded defaults.

use std::collections::HashMap;
use std::path::PathBuf;

const REQUIRED_PROMPTS: &[&str] = &[
    "implement.md",
    "review_1.md",
    "review_2.md",
    "kpop.md",
    "concerns.md",
];

const DEFAULT_PROMPTS: &[&str] = &[
    "implement.md",
    "review_1.md",
    "review_2.md",
    "kpop.md",
    "mbc2.md",
    "concerns.md",
    "learn.md",
    "coding_rules.md",
];

pub(crate) fn default_file(name: &str) -> Option<&'static str> {
    match name {
        "implement.md" => Some(include_str!("../../default_prompts/implement.md")),
        "review_1.md" => Some(include_str!("../../default_prompts/review_1.md")),
        "review_2.md" => Some(include_str!("../../default_prompts/review_2.md")),
        "kpop.md" => Some(include_str!("../../default_prompts/kpop.md")),
        "mbc2.md" => Some(include_str!("../../default_prompts/mbc2.md")),
        "concerns.md" => Some(include_str!("../../default_prompts/concerns.md")),
        "learn.md" => Some(include_str!("../../default_prompts/learn.md")),
        "coding_rules.md" => Some(include_str!("../../default_prompts/coding_rules.md")),
        _ => None,
    }
}

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

    /// Ensure prompts needed for standalone `malvin kpop` exist (`kpop.md`, and `learn.md` when learning runs).
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
    /// The same expansion is applied to `coding_rules.md` before it is injected into the main template.
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
        let rules_raw = self.load_coding_rules();
        let rules_expanded = render_template(&rules_raw, &render_context);
        render_context.insert("coding_rules".to_string(), rules_expanded);
        Ok(render_template(&prompt_text, &render_context))
    }

    fn load_coding_rules(&self) -> String {
        let p = self.root.join("coding_rules.md");
        std::fs::read_to_string(p)
            .unwrap_or_default()
            .trim()
            .to_string()
    }
}

pub(crate) fn render_template(prompt_text: &str, context: &HashMap<String, String>) -> String {
    let mut translated = prompt_text.to_string();
    for key in context.keys() {
        let needle = format!("{{{{ {key} }}}}");
        let dollar = format!("${key}");
        translated = translated.replace(&needle, &dollar);
    }
    substitute_template(&translated, context)
}

/// `$identifier` replacement similar to `string.Template.safe_substitute` (no `${}` brace forms).
pub(crate) fn substitute_template(template: &str, context: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let chars: Vec<char> = template.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_') {
                end += 1;
            }
            if end > start {
                let key: String = chars[start..end].iter().collect();
                if let Some(val) = context.get(&key) {
                    out.push_str(val);
                    i = end;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests;
