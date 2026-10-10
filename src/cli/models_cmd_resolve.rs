use std::io::Write;

use malvin::config::malvin_config_file::parse_model_cli_arg;
use malvin::config::model_id::ParsedModel;

#[must_use]
pub(crate) fn resolved_model_json(model: &ParsedModel) -> String {
    let provider = model.pi_provider_and_model().map(|(provider, _)| provider);
    let local = provider.is_some_and(malvin::local_llm::provider_is_keyless_local);
    serde_json::json!({
        "canonical": model.canonical(),
        "backend": model.backend.label(),
        "provider": provider,
        "local": local,
    })
    .to_string()
}

pub(crate) fn write_resolved_model(raw: &str, out: &mut impl Write) -> Result<(), String> {
    let model = parse_model_cli_arg(raw)?;
    writeln!(out, "{}", resolved_model_json(&model)).map_err(|e| format!("stdout: {e}"))
}

#[cfg(test)]
#[path = "models_cmd_resolve_tests.rs"]
mod models_cmd_resolve_tests;
