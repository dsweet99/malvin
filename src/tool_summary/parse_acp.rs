use serde_json::Value;

use super::parse::{json_number, LineRange};

pub(crate) fn acp_content_diff_paths(update: &Value) -> Option<Vec<String>> {
    let arr = update.get("content")?.as_array()?;
    let mut paths = Vec::new();
    for item in arr {
        if item.get("type").and_then(Value::as_str) != Some("diff") {
            continue;
        }
        if let Some(path) = acp_path_value(item) {
            paths.push(path);
        }
    }
    (!paths.is_empty()).then_some(paths)
}

pub(crate) fn acp_path_value(v: &Value) -> Option<String> {
    v.get("path")
        .or_else(|| v.get("filePath"))
        .or_else(|| v.get("uri"))
        .and_then(Value::as_str)
        .map(acp_normalize_path)
}

pub(crate) fn acp_normalize_path(s: &str) -> String {
    let Some(rest) = s.strip_prefix("file://") else {
        return s.to_string();
    };
    if rest.starts_with('/') {
        return rest.to_string();
    }
    rest.find('/')
        .map_or_else(|| rest.to_string(), |slash| rest[slash..].to_string())
}

pub(crate) fn merge_content_diff_paths(
    raw_output: Option<&Value>,
    content_diff_paths: Option<&[String]>,
) -> Option<Value> {
    match (raw_output, content_diff_paths) {
        (None, None) => None,
        (Some(raw), None) => Some(raw.clone()),
        (None, Some(paths)) => Some(content_diff_paths_to_raw_output(paths)),
        (Some(raw), Some(paths)) => {
            let mut merged = raw.clone();
            if !raw_output_has_edit_paths(raw) {
                merge_paths_into_raw(&mut merged, paths);
            }
            Some(merged)
        }
    }
}

pub(crate) fn content_diff_paths_to_raw_output(paths: &[String]) -> Value {
    if paths.len() == 1 {
        Value::Object(serde_json::Map::from_iter([(
            "path".to_string(),
            Value::String(paths[0].clone()),
        )]))
    } else {
        Value::Object(serde_json::Map::from_iter([(
            "paths".to_string(),
            Value::Array(paths.iter().cloned().map(Value::String).collect()),
        )]))
    }
}

pub(crate) fn raw_output_has_edit_paths(raw: &Value) -> bool {
    acp_path_value(raw).is_some()
        || raw
            .get("paths")
            .and_then(Value::as_array)
            .is_some_and(|paths| !paths.is_empty())
}

pub(crate) fn merge_paths_into_raw(raw: &mut Value, paths: &[String]) {
    let Some(obj) = raw.as_object_mut() else {
        return;
    };
    if paths.len() == 1 {
        obj.insert("path".to_string(), Value::String(paths[0].clone()));
        return;
    }
    obj.insert(
        "paths".to_string(),
        Value::Array(paths.iter().cloned().map(Value::String).collect()),
    );
}

pub(crate) fn acp_path_field(raw_input: Option<&Value>) -> Option<String> {
    raw_input.and_then(acp_path_value)
}

pub(crate) fn acp_search_query_field(raw_input: Option<&Value>) -> Option<String> {
    let v = raw_input?;
    v.get("query")
        .or_else(|| v.get("pattern"))
        .or_else(|| v.get("searchQuery"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

pub(crate) fn acp_line_range_field(raw: Option<&Value>) -> Option<LineRange> {
    let v = raw?;
    let line_number = v
        .get("lineNumber")
        .or_else(|| v.get("startLine"))
        .and_then(json_number);
    let end_line = v.get("endLine").and_then(json_number);
    if let Some(start) = line_number {
        let end = end_line.filter(|end| *end > start);
        return Some(LineRange { start, end });
    }
    let offset = v.get("offset").and_then(json_number);
    let limit = v.get("limit").and_then(json_number);
    let (Some(offset), Some(limit)) = (offset, limit) else {
        return None;
    };
    if limit == 0 {
        return None;
    }
    let start = offset.saturating_add(1);
    let end = if limit > 1 {
        Some(start.saturating_add(limit.saturating_sub(1)))
    } else {
        None
    };
    Some(LineRange { start, end })
}
