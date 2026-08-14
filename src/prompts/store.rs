use std::collections::HashMap;
use std::path::PathBuf;

use super::PromptError;
use super::defaults::{DEFAULT_PROMPTS, HEADER_MD, REQUIRED_PROMPTS, default_file};
use super::enforce_template_placeholders_resolved_in;
use super::render_template;

#[derive(Debug, Clone, Copy)]
pub struct KpopPromptValidation {
    pub require_mbc2: bool,
}

#[derive(Debug, Clone)]
pub struct PromptStore {
    root: Option<PathBuf>,
}

impl PromptStore {
    pub(crate) fn prompt_text(&self, filename: &str) -> Result<String, PromptError> {
        self.root.as_ref().map_or_else(
            || {
                default_file(filename)
                    .map(str::to_string)
                    .ok_or_else(|| {
                        PromptError(format!(
                            "Missing embedded prompt in binary: {filename}"
                        ))
                    })
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
        self.root.as_ref().map_or_else(
            || "embedded prompts".to_string(),
            |root| root.display().to_string(),
        )
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
            self.root.as_ref().map_or_else(
                || default_file(name).is_some(),
                |root| root.join(name).is_file(),
            )
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
        if validation.require_mbc2 && self.prompt_text("mbc2.md").is_err() {
            missing.push("mbc2.md");
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
        render_context.insert("coding_rules".to_string(), String::new());
        enforce_template_placeholders_resolved_in(&prompt_text, &render_context, Some(filename))?;
        Ok(render_template(&prompt_text, &render_context))
    }

    pub fn render_prompt_only(
        &self,
        filename: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, PromptError> {
        let prompt_text = self.prompt_text(filename)?;
        let render_context: HashMap<String, String> = context.clone();
        enforce_template_placeholders_resolved_in(&prompt_text, &render_context, Some(filename))?;
        Ok(render_template(&prompt_text, &render_context))
    }

    pub(crate) fn load_header(&self) -> String {
        self.prompt_text(HEADER_MD)
            .unwrap_or_default()
            .trim()
            .to_string()
    }
}

pub fn render_header(
    store: &PromptStore,
    context: &std::collections::HashMap<String, String>,
) -> Result<String, PromptError> {
    let header_raw = store.load_header();
    enforce_template_placeholders_resolved_in(&header_raw, context, Some(HEADER_MD))?;
    Ok(render_template(&header_raw, context))
}

pub fn render_mbc2_for_scheduled_kpop_block(
    store: &PromptStore,
    context: &std::collections::HashMap<String, String>,
) -> Result<String, PromptError> {
    let mut ctx = context.clone();
    ctx.insert("coding_rules".to_string(), String::new());
    store.render_prompt_only("mbc2.md", &ctx)
}

