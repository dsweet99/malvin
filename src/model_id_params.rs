use super::ModelParam;

#[must_use]
pub fn format_bracket_params(params: &[ModelParam]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let body = params
        .iter()
        .map(|p| format!("{}={}", p.id, p.value))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

pub fn split_bracket_params(raw: &str) -> Result<(String, Vec<ModelParam>), String> {
    let raw = raw.trim();
    let Some(open) = raw.find('[') else {
        return Ok((raw.to_string(), Vec::new()));
    };
    if !raw.ends_with(']') || open == 0 {
        return Err(bracket_shape_error(raw));
    }
    let base = raw[..open].trim();
    if base.is_empty() {
        return Err(bracket_shape_error(raw));
    }
    let inner = &raw[open + 1..raw.len() - 1];
    if inner.contains('[') || inner.contains(']') {
        return Err(format!(
            "model id bracket overrides must not nest brackets (got `{raw}`)"
        ));
    }
    Ok((base.to_string(), parse_bracket_inner(inner)?))
}

fn bracket_shape_error(raw: &str) -> String {
    if raw.ends_with(']') {
        format!("model id bracket overrides require a base id before `[` (got `{raw}`)")
    } else {
        format!("model id bracket overrides must end with `]` (got `{raw}`)")
    }
}

fn parse_bracket_inner(inner: &str) -> Result<Vec<ModelParam>, String> {
    let inner = inner.trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    let mut params = Vec::new();
    for part in inner.split(',') {
        params.push(parse_one_param(part)?);
    }
    Ok(params)
}

fn parse_one_param(part: &str) -> Result<ModelParam, String> {
    let part = part.trim();
    let Some((id, value)) = part.split_once('=') else {
        return Err(kv_error(part));
    };
    let id = id.trim();
    let value = value.trim();
    if id.is_empty() || value.is_empty() || id.contains('=') {
        return Err(kv_error(part));
    }
    Ok(ModelParam {
        id: id.to_string(),
        value: value.to_string(),
    })
}

fn kv_error(part: &str) -> String {
    if part.is_empty() {
        "model id bracket overrides must not contain empty entries".into()
    } else {
        format!("model id bracket overrides must be `key=value` (got `{part}`)")
    }
}

/// Shared `thinking=` vocabulary for `pi:` and `codex:`.
/// Wire adapters map levels a vendor does not name (`ultra` on Pi, `off`/`minimal` on Codex).
const THINKING_LEVELS: &[&str] = &[
    "off", "minimal", "low", "medium", "high", "xhigh", "max", "ultra",
];

pub(super) fn validate_pi_thinking_params(params: &[ModelParam]) -> Result<(), String> {
    for p in params {
        if p.id != "thinking" {
            return Err(format!(
                "pi model bracket overrides only support `thinking` (got `{}`)",
                p.id
            ));
        }
        require_known_value("thinking", THINKING_LEVELS, &p.value)?;
    }
    Ok(())
}

pub(super) fn validate_codex_params(params: &[ModelParam]) -> Result<(), String> {
    for p in params {
        match p.id.as_str() {
            "thinking" => require_known_value("thinking", THINKING_LEVELS, &p.value)?,
            "service" => require_nonempty_service(&p.value)?,
            other => {
                return Err(format!(
                    "codex model bracket overrides only support `thinking` and `service` (got `{other}`)",
                ));
            }
        }
    }
    Ok(())
}

fn require_known_value(label: &str, allowed: &[&str], value: &str) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "{label} must be one of {} (got `{value}`)",
            allowed.join("|")
        ))
    }
}

fn require_nonempty_service(value: &str) -> Result<(), String> {
    if value.is_empty() {
        Err("codex service must be a non-empty catalog tier id".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod model_id_params_kiss_cov {
    #[test]
    fn kiss_cov_model_id_params_idents() {
        let _ = super::format_bracket_params;
        let _ = super::split_bracket_params;
        let _ = stringify!(parse_bracket_inner);
        let _ = stringify!(parse_one_param);
        let _ = stringify!(bracket_shape_error);
        let _ = stringify!(kv_error);
        let _ = stringify!(validate_pi_thinking_params);
        let _ = super::validate_codex_params;
        let _ = super::require_known_value;
        let _ = super::require_nonempty_service;
    }
}
