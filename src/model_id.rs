//! Prefixed model ids: `cursor:…`, `prime:…`.
pub const CURSOR_PREFIX: &str = "cursor:";
pub const PRIME_PREFIX: &str = "prime:";

/// Legacy prefixes — rejected with a rename hint.
pub const MINI_PREFIX: &str = "mini:";
pub const OPENROUTER_PREFIX: &str = "openrouter:";
pub const LOCAL_PREFIX: &str = "local:";

pub const UNPREFIXED_MODEL_MESSAGE: &str =
    "model id must use a `cursor:` or `prime:` prefix (for example `cursor:auto`, `prime:openai/gpt-5.5`, or `prime:local/qwen35_9b_q4`)";

const LEGACY_MINI_HINT: &str =
    "legacy `mini:` prefix removed; use `prime:` (for example `prime:openrouter/<slug>` or `prime:local/<slug>`)";
const LEGACY_OPENROUTER_HINT: &str =
    "legacy `openrouter:` prefix removed; use `prime:openrouter/<slug>` (for example `prime:openrouter/anthropic/claude-3-haiku`)";
const LEGACY_LOCAL_HINT: &str =
    "legacy `local:` prefix removed; use `prime:local/<slug>` (for example `prime:local/qwen35_9b_q4`)";
const LEGACY_PRIME_LOCAL_HINT: &str =
    "use `prime:local/<slug>` instead of `prime:local/local/<slug>` (for example `prime:local/qwen35_9b_q4`)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelBackend {
    Cursor,
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
        }
    }

    /// `prime:local/<slug>` (malvin GGUF via Prime sidecar).
    #[must_use]
    pub fn is_prime_local(&self) -> bool {
        matches!(self.backend, ModelBackend::Prime)
            && self.slug.starts_with("local/")
            && self.slug.len() > "local/".len()
    }

    #[must_use]
    pub const fn is_prime(&self) -> bool {
        matches!(self.backend, ModelBackend::Prime)
    }

    /// Catalog slug for `prime:local/<slug>` GGUF ids.
    #[must_use]
    pub fn local_catalog_slug(&self) -> Option<&str> {
        if self.is_prime_local() {
            self.slug.strip_prefix("local/")
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
    Err(legacy_or_unprefixed_error(raw))
}

fn legacy_or_unprefixed_error(raw: &str) -> String {
    if raw.starts_with(MINI_PREFIX) {
        LEGACY_MINI_HINT.to_string()
    } else if raw.starts_with(OPENROUTER_PREFIX) {
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
    if rest.starts_with("local/local/") {
        return Err(LEGACY_PRIME_LOCAL_HINT.to_string());
    }
    let err = || format!("prime model id must be `prime:<provider>/<model>` (got `prime:{rest}`)");
    let Some((provider, model)) = split_first_slash(rest) else {
        return Err(err());
    };
    if provider.is_empty() || model.is_empty() {
        return Err(err());
    }
    Ok(ParsedModel {
        backend: ModelBackend::Prime,
        slug: rest.to_string(),
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
        Ok(parsed) => parsed.slug,
        Err(_) => raw.to_string(),
    }
}

#[must_use]
pub fn uses_local_backend(raw: &str) -> bool {
    parse_model_id(raw)
        .map(|p| p.is_prime_local())
        .unwrap_or(false)
}

#[must_use]
pub fn uses_prime_local_backend(raw: &str) -> bool {
    parse_model_id(raw).map(|p| p.is_prime_local()).unwrap_or(false)
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
