//! Prefixed model ids: `cursor:…`, `prime:…`, `mini:openrouter/…`, `mini:local/…`.
use crate::support_paths::MINI_DEFAULT_MODEL;

pub const CURSOR_PREFIX: &str = "cursor:";
pub const PRIME_PREFIX: &str = "prime:";
pub const MINI_PREFIX: &str = "mini:";

/// Legacy prefixes — rejected with a rename hint.
pub const OPENROUTER_PREFIX: &str = "openrouter:";
pub const LOCAL_PREFIX: &str = "local:";

pub const UNPREFIXED_MODEL_MESSAGE: &str =
    "model id must use a `cursor:`, `prime:`, or `mini:` prefix (for example `cursor:auto`, `prime:openai/gpt-5.5`, `mini:openrouter/anthropic/claude-3-haiku`, or `mini:local/qwen35_9b_q4`)";

const LEGACY_OPENROUTER_HINT: &str =
    "legacy `openrouter:` prefix removed; use `mini:openrouter/<slug>` (for example `mini:openrouter/anthropic/claude-3-haiku`)";
const LEGACY_LOCAL_HINT: &str =
    "legacy `local:` prefix removed; use `mini:local/<slug>` (for example `mini:local/qwen35_9b_q4`)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiniTransport {
    OpenRouter,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelBackend {
    Cursor,
    Mini(MiniTransport),
    Prime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedModel {
    pub backend: ModelBackend,
    /// Provider / transport slug without the malvin prefix.
    pub slug: String,
}

impl ParsedModel {
    #[must_use]
    pub fn canonical(&self) -> String {
        match self.backend {
            ModelBackend::Cursor => format!("{CURSOR_PREFIX}{}", self.slug),
            ModelBackend::Prime => format!("{PRIME_PREFIX}{}", self.slug),
            ModelBackend::Mini(MiniTransport::OpenRouter) => {
                format!("{MINI_PREFIX}openrouter/{}", self.slug)
            }
            ModelBackend::Mini(MiniTransport::Local) => {
                format!("{MINI_PREFIX}local/{}", self.slug)
            }
        }
    }

    #[must_use]
    pub const fn is_openrouter(&self) -> bool {
        matches!(self.backend, ModelBackend::Mini(MiniTransport::OpenRouter))
    }

    #[must_use]
    pub const fn is_local(&self) -> bool {
        matches!(self.backend, ModelBackend::Mini(MiniTransport::Local))
    }

    /// `prime:local/local/<slug>` (malvin GGUF via Prime sidecar).
    #[must_use]
    pub fn is_prime_local(&self) -> bool {
        matches!(self.backend, ModelBackend::Prime)
            && self.slug.starts_with("local/local/")
            && self.slug.len() > "local/local/".len()
    }

    #[must_use]
    pub const fn is_prime(&self) -> bool {
        matches!(self.backend, ModelBackend::Prime)
    }

    #[must_use]
    pub const fn uses_mini_http(&self) -> bool {
        matches!(self.backend, ModelBackend::Mini(_))
    }

    /// Catalog slug for mini/prime local GGUF ids.
    #[must_use]
    pub fn local_catalog_slug(&self) -> Option<&str> {
        if self.is_local() {
            Some(self.slug.as_str())
        } else if self.is_prime_local() {
            self.slug.strip_prefix("local/local/")
        } else {
            None
        }
    }
}

/// Parse a canonical or raw model id. Bare (unprefixed) ids are rejected.
pub fn parse_model_id(raw: &str) -> Result<ParsedModel, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(UNPREFIXED_MODEL_MESSAGE.to_string());
    }
    if let Some(rest) = raw.strip_prefix(CURSOR_PREFIX) {
        return parsed(ModelBackend::Cursor, rest);
    }
    if let Some(rest) = raw.strip_prefix(PRIME_PREFIX) {
        return parse_prime(rest);
    }
    if let Some(rest) = raw.strip_prefix(MINI_PREFIX) {
        return parse_mini(rest);
    }
    Err(legacy_or_unprefixed_error(raw))
}

fn legacy_or_unprefixed_error(raw: &str) -> String {
    if raw.starts_with(OPENROUTER_PREFIX) {
        LEGACY_OPENROUTER_HINT.to_string()
    } else if raw.starts_with(LOCAL_PREFIX) {
        LEGACY_LOCAL_HINT.to_string()
    } else {
        UNPREFIXED_MODEL_MESSAGE.to_string()
    }
}

fn parse_prime(rest: &str) -> Result<ParsedModel, String> {
    let rest = rest.trim();
    if rest.is_empty() {
        return Err(UNPREFIXED_MODEL_MESSAGE.to_string());
    }
    let Some((provider, model)) = split_first_slash(rest) else {
        return Err(format!(
            "prime model id must be `prime:<provider>/<model>` (got `prime:{rest}`)"
        ));
    };
    if provider.is_empty() || model.is_empty() {
        return Err(format!(
            "prime model id must be `prime:<provider>/<model>` (got `prime:{rest}`)"
        ));
    }
    Ok(ParsedModel {
        backend: ModelBackend::Prime,
        slug: rest.to_string(),
    })
}

fn parse_mini(rest: &str) -> Result<ParsedModel, String> {
    let rest = rest.trim();
    let Some((transport, slug)) = split_first_slash(rest) else {
        return Err(format!(
            "mini model id must be `mini:openrouter/<slug>` or `mini:local/<slug>` (got `mini:{rest}`)"
        ));
    };
    if transport.is_empty() || slug.is_empty() {
        return Err(format!(
            "mini model id must be `mini:openrouter/<slug>` or `mini:local/<slug>` (got `mini:{rest}`)"
        ));
    }
    let backend = match transport {
        "openrouter" => ModelBackend::Mini(MiniTransport::OpenRouter),
        "local" => ModelBackend::Mini(MiniTransport::Local),
        other => {
            return Err(format!(
                "unknown mini transport `{other}`; use `openrouter` or `local` (for example `mini:openrouter/…` or `mini:local/…`)"
            ));
        }
    };
    Ok(ParsedModel {
        backend,
        slug: slug.to_string(),
    })
}

fn split_first_slash(s: &str) -> Option<(&str, &str)> {
    s.split_once('/')
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

pub fn require_config_model(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(crate::support_paths::DEFAULT_CLI_MODEL.to_string());
    }
    require_prefixed_model(raw)
}

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
    parse_model_id(raw)
        .map(|p| p.is_local() || p.is_prime_local())
        .unwrap_or(false)
}

/// True for `prime:local/local/<slug>`.
#[must_use]
pub fn uses_prime_local_backend(raw: &str) -> bool {
    parse_model_id(raw).map(|p| p.is_prime_local()).unwrap_or(false)
}

#[must_use]
pub fn uses_mini_backend(raw: &str) -> bool {
    parse_model_id(raw)
        .map(|p| p.uses_mini_http())
        .unwrap_or(false)
}

#[must_use]
pub fn uses_prime_backend(raw: &str) -> bool {
    parse_model_id(raw).map(|p| p.is_prime()).unwrap_or(false)
}

pub fn require_prefixed_model(raw: &str) -> Result<String, String> {
    Ok(parse_model_id(raw)?.canonical())
}
#[cfg(test)]
#[path = "model_id_tests.rs"]
mod model_id_tests;
