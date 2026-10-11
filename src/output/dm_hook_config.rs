use super::dm_hook_split::split_on_emit;

pub(crate) fn on_emit_tokens(text: &str) -> Result<Option<Vec<String>>, String> {
    let Some(spec) = on_emit_spec(text)? else {
        return Ok(None);
    };
    if spec.trim().is_empty() {
        return Ok(None);
    }
    let tokens = split_on_emit(&spec)?;
    if tokens.is_empty() {
        Ok(None)
    } else {
        Ok(Some(tokens))
    }
}

fn on_emit_spec(text: &str) -> Result<Option<String>, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|err| format!("could not parse dm.on_emit: {err}"))?;
    let Some(on_emit) = value.get("dm").and_then(|table| table.get("on_emit")) else {
        return Ok(None);
    };
    if on_emit.as_array().is_some_and(Vec::is_empty) {
        return Ok(None);
    }
    on_emit
        .as_str()
        .map(|spec| Some(spec.to_string()))
        .ok_or_else(|| "dm.on_emit must be a string".to_string())
}

pub(crate) fn load_home_tokens() -> Option<Vec<String>> {
    let path = crate::malvin_home_config_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return None;
    };
    match on_emit_tokens(&text) {
        Ok(tokens) => tokens,
        Err(msg) => {
            super::super::print_log_warning(&msg);
            None
        }
    }
}
