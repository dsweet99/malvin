//! Prefixed model ids: `cursor:…`, `pi:…`.
pub const CURSOR_PREFIX: &str = "cursor:";
pub const PI_PREFIX: &str = "pi:";

/// Legacy prefixes — rejected with a rename hint.
pub const MINI_PREFIX: &str = "mini:";
pub const OPENROUTER_PREFIX: &str = "openrouter:";
pub const LOCAL_PREFIX: &str = "local:";
pub const PRIME_PREFIX: &str = "prime:";

pub const UNPREFIXED_MODEL_MESSAGE: &str =
    "model id must use a `cursor:` or `pi:` prefix (for example `cursor:auto` or `pi:openai/gpt-4o`)";

const LEGACY_MINI_HINT: &str =
    "legacy `mini:` prefix removed; use `pi:` (for example `pi:openrouter/<slug>`)";
const LEGACY_OPENROUTER_HINT: &str =
    "legacy `openrouter:` prefix removed; use `pi:openrouter/<slug>` (for example `pi:openrouter/anthropic/claude-3-haiku`)";
const LEGACY_LOCAL_HINT: &str = "legacy `local:` prefix removed; local GGUF models are no longer supported";
const LEGACY_PRIME_HINT: &str =
    "legacy `prime:` prefix removed; use `cursor:` or `pi:` (for example `cursor:auto` or `pi:openai/gpt-4o`)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelBackend {
    Cursor,
    Pi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedModel {
    pub backend: ModelBackend,
    /// Provider / transport slug without the malvin prefix.
    pub slug: String,
}

impl std::fmt::Display for ParsedModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.canonical())
    }
}

impl ParsedModel {
    #[must_use]
    pub fn canonical(&self) -> String {
        match self.backend {
            ModelBackend::Cursor => format!("{CURSOR_PREFIX}{}", self.slug),
            ModelBackend::Pi => format!("{PI_PREFIX}{}", self.slug),
        }
    }

    #[must_use]
    pub const fn is_pi(&self) -> bool {
        matches!(self.backend, ModelBackend::Pi)
    }

    /// Split `pi:<provider>/<model>` on the first `/` after the prefix.
    ///
    /// Model ids may themselves contain `/` (for example `openrouter/anthropic/claude-3-haiku`).
    #[must_use]
    pub fn pi_provider_and_model(&self) -> Option<(&str, &str)> {
        if !self.is_pi() {
            return None;
        }
        split_first_slash(&self.slug).filter(|(p, m)| !p.is_empty() && !m.is_empty())
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
    if let Some(rest) = raw.strip_prefix(PI_PREFIX) {
        return parse_pi(rest);
    }
    Err(legacy_or_unprefixed_error(raw))
}

fn legacy_or_unprefixed_error(raw: &str) -> String {
    if raw.starts_with(PRIME_PREFIX) {
        LEGACY_PRIME_HINT.to_string()
    } else if raw.starts_with(MINI_PREFIX) {
        LEGACY_MINI_HINT.to_string()
    } else if raw.starts_with(OPENROUTER_PREFIX) {
        LEGACY_OPENROUTER_HINT.to_string()
    } else if raw.starts_with(LOCAL_PREFIX) {
        LEGACY_LOCAL_HINT.to_string()
    } else {
        UNPREFIXED_MODEL_MESSAGE.to_string()
    }
}

fn parse_pi(rest: &str) -> Result<ParsedModel, String> {
    let rest = rest.trim();
    if rest.is_empty() {
        return Err(UNPREFIXED_MODEL_MESSAGE.to_string());
    }
    let err = || format!("pi model id must be `pi:<provider>/<model>` (got `pi:{rest}`)");
    let Some((provider, model)) = split_first_slash(rest) else {
        return Err(err());
    };
    if provider.is_empty() || model.is_empty() {
        return Err(err());
    }
    Ok(ParsedModel {
        backend: ModelBackend::Pi,
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
pub fn uses_pi_backend(raw: &str) -> bool {
    parse_model_id(raw).map(|p| p.is_pi()).unwrap_or(false)
}

pub fn require_prefixed_model(raw: &str) -> Result<String, String> {
    Ok(parse_model_id(raw)?.canonical())
}
#[cfg(test)]
#[path = "model_id_tests.rs"]
mod model_id_tests;
