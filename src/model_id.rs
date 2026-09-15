#[path = "model_id_params.rs"]
mod model_id_params;
pub use model_id_params::{format_bracket_params, split_bracket_params};
#[path = "model_id_legacy.rs"]
mod model_id_legacy;

pub const CURSOR_PREFIX: &str = "cursor:";
pub const PI_PREFIX: &str = "pi:";
pub const RPI_PREFIX: &str = "rpi:";
pub const CODEX_PREFIX: &str = "codex:";

pub const MINI_PREFIX: &str = "mini:";
pub const OPENROUTER_PREFIX: &str = "openrouter:";
pub const LOCAL_PREFIX: &str = "local:";
pub const PRIME_PREFIX: &str = "prime:";

pub const UNPREFIXED_MODEL_MESSAGE: &str = "model id must use a `cursor:`, `pi:`, `rpi:`, or `codex:` prefix (for example `cursor:auto`, `pi:openai/gpt-4o`, or `codex:gpt-5.6`)";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelParam {
    pub id: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelBackend {
    Cursor,
    NpmPi,
    Pi,
    Codex,
}

impl ModelBackend {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cursor => "cursor",
            Self::NpmPi => "pi",
            Self::Pi => "rpi",
            Self::Codex => "codex",
        }
    }

    #[must_use]
    pub const fn drain_idle_prefix(self) -> &'static str {
        match self {
            Self::Cursor => "bridge timed out",
            Self::NpmPi => "npm pi rpc timed out",
            Self::Pi => "pi rpc timed out",
            Self::Codex => "codex timed out",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedModel {
    pub backend: ModelBackend,
    pub slug: String,
    pub params: Vec<ModelParam>,
}

impl std::fmt::Display for ParsedModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.canonical())
    }
}

impl ParsedModel {
    #[must_use]
    pub fn canonical(&self) -> String {
        let base = match self.backend {
            ModelBackend::Cursor => format!("{CURSOR_PREFIX}{}", self.slug),
            ModelBackend::NpmPi => format!("{PI_PREFIX}{}", self.slug),
            ModelBackend::Pi => format!("{RPI_PREFIX}{}", self.slug),
            ModelBackend::Codex => format!("{CODEX_PREFIX}{}", self.slug),
        };
        if self.params.is_empty() {
            base
        } else {
            format!("{base}{}", format_bracket_params(&self.params))
        }
    }

    #[must_use]
    pub const fn is_pi(&self) -> bool {
        matches!(self.backend, ModelBackend::Pi)
    }

    #[must_use]
    pub const fn is_npm_pi(&self) -> bool {
        matches!(self.backend, ModelBackend::NpmPi)
    }

    #[must_use]
    pub const fn is_codex(&self) -> bool {
        matches!(self.backend, ModelBackend::Codex)
    }

    #[must_use]
    pub fn pi_provider_and_model(&self) -> Option<(&str, &str)> {
        if !matches!(self.backend, ModelBackend::Pi | ModelBackend::NpmPi) {
            return None;
        }
        let (provider, model) =
            split_first_slash(&self.slug).filter(|(p, m)| !p.is_empty() && !m.is_empty())?;
        if provider.eq_ignore_ascii_case("local") {
            return split_first_slash(model)
                .filter(|(p, m)| !p.is_empty() && !m.is_empty())
                .or(Some(("ollama", model)));
        }
        Some((provider, model))
    }

    #[must_use]
    pub fn cursor_bridge_model(&self) -> String {
        if self.params.is_empty() {
            self.slug.clone()
        } else {
            format!("{}{}", self.slug, format_bracket_params(&self.params))
        }
    }

    #[must_use]
    pub fn thinking_param(&self) -> Option<&str> {
        self.named_param("thinking")
    }

    #[must_use]
    pub fn service_param(&self) -> Option<&str> {
        self.named_param("service")
    }

    fn named_param(&self, id: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.value.as_str())
    }
}

pub fn parse_model_id(raw: &str) -> Result<ParsedModel, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(UNPREFIXED_MODEL_MESSAGE.to_string());
    }
    if let Some(rest) = raw.strip_prefix(CURSOR_PREFIX) {
        return parsed(ModelBackend::Cursor, rest);
    }
    if let Some(rest) = raw.strip_prefix(PI_PREFIX) {
        return parse_provider_slash_model(rest, ModelBackend::NpmPi, "pi");
    }
    if let Some(rest) = raw.strip_prefix(RPI_PREFIX) {
        return parse_provider_slash_model(rest, ModelBackend::Pi, "rpi");
    }
    if let Some(rest) = raw.strip_prefix(CODEX_PREFIX) {
        return parse_codex(rest);
    }
    Err(model_id_legacy::legacy_or_unprefixed_error(raw))
}

fn parse_provider_slash_model(
    rest: &str,
    backend: ModelBackend,
    prefix_label: &str,
) -> Result<ParsedModel, String> {
    let rest = rest.trim();
    if rest.is_empty() {
        return Err(UNPREFIXED_MODEL_MESSAGE.to_string());
    }
    let (slug, params) = split_bracket_params(rest)?;
    let err = || {
        format!(
            "{prefix_label} model id must be `{prefix_label}:<provider>/<model>` (got `{prefix_label}:{rest}`)"
        )
    };
    let Some((provider, model)) = split_first_slash(&slug) else {
        return Err(err());
    };
    if provider.is_empty() || model.is_empty() {
        return Err(err());
    }
    model_id_params::validate_pi_thinking_params(&params)?;
    Ok(ParsedModel {
        backend,
        slug,
        params,
    })
}

fn split_first_slash(s: &str) -> Option<(&str, &str)> {
    s.split_once('/')
}

fn parse_codex(rest: &str) -> Result<ParsedModel, String> {
    let model = parsed(ModelBackend::Codex, rest)?;
    model_id_params::validate_codex_params(&model.params)?;
    Ok(model)
}

fn parsed(backend: ModelBackend, slug: &str) -> Result<ParsedModel, String> {
    let (slug, params) = split_bracket_params(slug)?;
    if slug.is_empty() {
        return Err(UNPREFIXED_MODEL_MESSAGE.to_string());
    }
    Ok(ParsedModel {
        backend,
        slug,
        params,
    })
}

pub fn require_config_model(raw: &str) -> Result<ParsedModel, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return parse_model_id(crate::support_paths::DEFAULT_CLI_MODEL);
    }
    parse_model_id(raw)
}

pub fn require_prefixed_model(raw: &str) -> Result<String, String> {
    Ok(parse_model_id(raw)?.canonical())
}

#[cfg(test)]
#[path = "model_id_tests.rs"]
mod model_id_tests;
