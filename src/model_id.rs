//! Prefixed model ids: `cursor:…` (ACP), `openrouter:…` / `local:…` (malvin-mini).

use crate::support_paths::MINI_DEFAULT_MODEL;

pub const CURSOR_PREFIX: &str = "cursor:";
pub const OPENROUTER_PREFIX: &str = "openrouter:";
pub const LOCAL_PREFIX: &str = "local:";

pub const UNPREFIXED_MODEL_MESSAGE: &str =
    "model id must use a `cursor:`, `openrouter:`, or `local:` prefix (for example `cursor:auto`, `openrouter:nvidia/nemotron-3-ultra-550b-a55b:free`, or `local:qwen35_9b_q4`)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelBackend {
    Cursor,
    OpenRouter,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedModel {
    pub backend: ModelBackend,
    /// Provider slug without the malvin prefix (`auto`, `gpt-5.1-low`, `org/model`, …).
    pub slug: String,
}

impl ParsedModel {
    #[must_use]
    pub fn canonical(&self) -> String {
        match self.backend {
            ModelBackend::Cursor => format!("{CURSOR_PREFIX}{}", self.slug),
            ModelBackend::OpenRouter => format!("{OPENROUTER_PREFIX}{}", self.slug),
            ModelBackend::Local => format!("{LOCAL_PREFIX}{}", self.slug),
        }
    }

    #[must_use]
    pub const fn is_openrouter(&self) -> bool {
        matches!(self.backend, ModelBackend::OpenRouter)
    }

    #[must_use]
    pub const fn is_local(&self) -> bool {
        matches!(self.backend, ModelBackend::Local)
    }

    /// True when the model uses the malvin-mini agent loop (`openrouter:` HTTP or `local:` in-process).
    #[must_use]
    pub const fn uses_mini_http(&self) -> bool {
        matches!(self.backend, ModelBackend::OpenRouter | ModelBackend::Local)
    }
}

/// Parse a canonical or raw model id. Bare (unprefixed) ids are rejected.
///
/// # Errors
///
/// Returns an error when the id lacks a known prefix or the slug is empty.
pub fn parse_model_id(raw: &str) -> Result<ParsedModel, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(UNPREFIXED_MODEL_MESSAGE.to_string());
    }
    if let Some(slug) = raw.strip_prefix(CURSOR_PREFIX) {
        return parsed(ModelBackend::Cursor, slug);
    }
    if let Some(slug) = raw.strip_prefix(OPENROUTER_PREFIX) {
        return parsed(ModelBackend::OpenRouter, slug);
    }
    if let Some(slug) = raw.strip_prefix(LOCAL_PREFIX) {
        return parsed(ModelBackend::Local, slug);
    }
    Err(UNPREFIXED_MODEL_MESSAGE.to_string())
}

fn parsed(backend: ModelBackend, slug: &str) -> Result<ParsedModel, String> {
    let slug = slug.trim();
    if slug.is_empty() {
        return Err(UNPREFIXED_MODEL_MESSAGE.to_string());
    }
    Ok(ParsedModel {
        backend,
        slug: slug.to_string(),
    })
}

/// Validate a config `model` value. Empty becomes the default; bare ids are rejected.
///
/// # Errors
///
/// Returns [`UNPREFIXED_MODEL_MESSAGE`] when the id is bare or invalid.
pub fn require_config_model(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(crate::support_paths::DEFAULT_CLI_MODEL.to_string());
    }
    require_prefixed_model(raw)
}

/// Resolve the provider slug passed to Cursor ACP, `OpenRouter`, or local llama.cpp.
#[must_use]
pub fn provider_slug(raw: &str) -> String {
    match parse_model_id(raw) {
        Ok(parsed) if parsed.is_openrouter() => resolve_openrouter_slug(&parsed.slug),
        Ok(parsed) => parsed.slug,
        Err(_) => raw.to_string(),
    }
}

#[must_use]
pub fn resolve_openrouter_slug(slug: &str) -> String {
    if slug == "auto" {
        MINI_DEFAULT_MODEL.to_string()
    } else {
        slug.to_string()
    }
}

#[must_use]
pub fn uses_openrouter_backend(raw: &str) -> bool {
    parse_model_id(raw)
        .map(|p| p.is_openrouter())
        .unwrap_or(false)
}

#[must_use]
pub fn uses_local_backend(raw: &str) -> bool {
    parse_model_id(raw).map(|p| p.is_local()).unwrap_or(false)
}

/// True when the model runs through the malvin-mini agent loop (`openrouter:` or `local:`).
#[must_use]
pub fn uses_mini_backend(raw: &str) -> bool {
    parse_model_id(raw)
        .map(|p| p.uses_mini_http())
        .unwrap_or(false)
}

/// Validate a CLI `--model` value (must already be prefixed).
///
/// # Errors
///
/// Returns [`UNPREFIXED_MODEL_MESSAGE`] when the id is bare or invalid.
pub fn require_prefixed_model(raw: &str) -> Result<String, String> {
    Ok(parse_model_id(raw)?.canonical())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cursor_openrouter_and_local() {
        let c = parse_model_id("cursor:auto").expect("cursor");
        assert_eq!(c.backend, ModelBackend::Cursor);
        assert_eq!(c.slug, "auto");
        assert_eq!(c.canonical(), "cursor:auto");

        let o = parse_model_id("openrouter:org/model").expect("or");
        assert!(o.is_openrouter());
        assert_eq!(o.slug, "org/model");

        let local = parse_model_id("local:qwen35_9b_q4").expect("local");
        assert!(local.is_local());
        assert!(local.uses_mini_http());
        assert_eq!(local.canonical(), "local:qwen35_9b_q4");
    }

    #[test]
    fn reject_bare_and_empty_slug() {
        assert!(parse_model_id("auto").is_err());
        assert!(parse_model_id("cursor:").is_err());
        assert!(parse_model_id("openrouter:").is_err());
        assert!(parse_model_id("local:").is_err());
    }

    #[test]
    fn require_config_model_rejects_bare() {
        assert!(require_config_model("auto").is_err());
        assert!(require_config_model("gpt-5.1-low").is_err());
        assert_eq!(
            require_config_model("openrouter:x/y").expect("ok"),
            "openrouter:x/y"
        );
        assert_eq!(
            require_config_model("local:qwen35_9b_q4").expect("ok"),
            "local:qwen35_9b_q4"
        );
        assert_eq!(
            require_config_model("").expect("default"),
            crate::support_paths::DEFAULT_CLI_MODEL
        );
    }

    #[test]
    fn provider_slug_resolves_openrouter_auto() {
        assert_eq!(provider_slug("cursor:auto"), "auto");
        assert_eq!(provider_slug("openrouter:auto"), MINI_DEFAULT_MODEL);
        assert_eq!(
            provider_slug("openrouter:openai/gpt-4o"),
            "openai/gpt-4o"
        );
        assert_eq!(provider_slug("local:qwen35_9b_q4"), "qwen35_9b_q4");
    }

    #[test]
    fn uses_mini_backend_covers_openrouter_and_local() {
        assert!(uses_mini_backend("openrouter:x"));
        assert!(uses_mini_backend("local:qwen35_9b_q4"));
        assert!(!uses_mini_backend("cursor:auto"));
        assert!(uses_local_backend("local:nemotron3_nano_4b"));
        assert!(!uses_local_backend("openrouter:x"));
        assert!(uses_openrouter_backend("openrouter:org/m"));
        assert!(!uses_openrouter_backend("local:qwen35_9b_q4"));
        let parsed: ParsedModel = parse_model_id("cursor:auto").expect("cursor");
        assert_eq!(parsed.backend, ModelBackend::Cursor);
    }
}
