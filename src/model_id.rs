#[path = "model_id_params.rs"]
mod model_id_params;
pub use model_id_params::{format_bracket_params, split_bracket_params};
#[path = "model_id_legacy.rs"]
mod model_id_legacy;
#[path = "model_id_spec.rs"]
mod model_id_spec;
pub use model_id_spec::BackendSpec;

pub const CURSOR_PREFIX: &str = "cursor:";
pub const PI_PREFIX: &str = "pi:";
pub const CODEX_PREFIX: &str = "codex:";

pub const MINI_PREFIX: &str = "mini:";
pub const RPI_PREFIX: &str = "rpi:";
pub const OPENROUTER_PREFIX: &str = "openrouter:";
pub const LOCAL_PREFIX: &str = "local:";
pub const PRIME_PREFIX: &str = "prime:";

pub const UNPREFIXED_MODEL_MESSAGE: &str = "model id must use a `cursor:`, `pi:`, or `codex:` prefix (for example `cursor:auto`, `pi:openai/gpt-4o`, or `codex:gpt-5.6`)";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelParam {
    pub id: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelBackend {
    Cursor,
    Pi,
    Codex,
}

impl ModelBackend {
    pub const ALL: [Self; 3] = [Self::Cursor, Self::Pi, Self::Codex];

    #[must_use]
    pub const fn spec(self) -> &'static dyn BackendSpec {
        match self {
            Self::Cursor => &model_id_spec::CursorSpec,
            Self::Pi => &model_id_spec::PiSpec,
            Self::Codex => &model_id_spec::CodexSpec,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        self.spec().label()
    }

    #[must_use]
    pub fn drain_idle_prefix(self) -> &'static str {
        self.spec().drain_idle_prefix()
    }

    #[must_use]
    pub fn wire_label(self) -> &'static str {
        self.spec().wire_label()
    }

    #[must_use]
    pub fn bridge_wire_model(self, model: &ParsedModel) -> String {
        debug_assert_eq!(self, model.backend);
        self.spec().wire_model(model)
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
        let base = format!("{}{}", self.backend.spec().prefix(), self.slug);
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
    pub const fn is_codex(&self) -> bool {
        matches!(self.backend, ModelBackend::Codex)
    }

    #[must_use]
    pub fn pi_provider_and_model(&self) -> Option<(&str, &str)> {
        if !matches!(self.backend, ModelBackend::Pi) {
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
    for backend in ModelBackend::ALL {
        let spec = backend.spec();
        if let Some(rest) = raw.strip_prefix(spec.prefix()) {
            return spec.parse_slug(rest);
        }
    }
    Err(model_id_legacy::legacy_or_unprefixed_error(raw))
}

fn split_first_slash(s: &str) -> Option<(&str, &str)> {
    s.split_once('/')
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
