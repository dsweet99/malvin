use super::model_id_params::{validate_codex_params, validate_pi_thinking_params};
use super::{
    CODEX_PREFIX, CURSOR_PREFIX, ModelBackend, PI_PREFIX, ParsedModel, UNPREFIXED_MODEL_MESSAGE,
    split_bracket_params,
};

pub trait BackendSpec: Sync {
    fn prefix(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn drain_idle_prefix(&self) -> &'static str;
    fn wire_model(&self, model: &ParsedModel) -> String;
    fn parse_slug(&self, rest: &str) -> Result<ParsedModel, String>;
}

pub(super) struct CursorSpec;
pub(super) struct PiSpec;
pub(super) struct CodexSpec;

impl BackendSpec for CursorSpec {
    fn prefix(&self) -> &'static str {
        CURSOR_PREFIX
    }

    fn label(&self) -> &'static str {
        "cursor"
    }

    fn drain_idle_prefix(&self) -> &'static str {
        "bridge timed out"
    }

    fn wire_model(&self, model: &ParsedModel) -> String {
        model.cursor_bridge_model()
    }

    fn parse_slug(&self, rest: &str) -> Result<ParsedModel, String> {
        parsed(ModelBackend::Cursor, rest)
    }
}

impl BackendSpec for PiSpec {
    fn prefix(&self) -> &'static str {
        PI_PREFIX
    }

    fn label(&self) -> &'static str {
        "pi"
    }

    fn drain_idle_prefix(&self) -> &'static str {
        "npm pi rpc timed out"
    }

    fn wire_model(&self, model: &ParsedModel) -> String {
        model.slug.clone()
    }

    fn parse_slug(&self, rest: &str) -> Result<ParsedModel, String> {
        parse_provider_slash_model(rest, ModelBackend::Pi, self.label())
    }
}

impl BackendSpec for CodexSpec {
    fn prefix(&self) -> &'static str {
        CODEX_PREFIX
    }

    fn label(&self) -> &'static str {
        "codex"
    }

    fn drain_idle_prefix(&self) -> &'static str {
        "codex timed out"
    }

    fn wire_model(&self, model: &ParsedModel) -> String {
        model.slug.clone()
    }

    fn parse_slug(&self, rest: &str) -> Result<ParsedModel, String> {
        let model = parsed(ModelBackend::Codex, rest)?;
        validate_codex_params(&model.params)?;
        Ok(model)
    }
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
    let Some((provider, model)) = slug.split_once('/') else {
        return Err(err());
    };
    if provider.is_empty() || model.is_empty() {
        return Err(err());
    }
    validate_pi_thinking_params(&params)?;
    Ok(ParsedModel {
        backend,
        slug,
        params,
    })
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
