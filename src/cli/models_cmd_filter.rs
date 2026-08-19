use crate::model_id::PI_PREFIX;

pub(crate) fn models_list_prefix(words: &[String]) -> Result<Option<String>, String> {
    if words.is_empty() {
        return Ok(None);
    }
    if words[0].eq_ignore_ascii_case("download") {
        return Err(
            "`malvin models` no longer downloads; local GGUF models are no longer supported".into(),
        );
    }
    Ok(Some(join_models_prefix_words(words)))
}

pub(crate) fn join_models_prefix_words(words: &[String]) -> String {
    let mut out = String::new();
    for word in words {
        if out.is_empty() {
            out.push_str(word);
            continue;
        }
        if needs_models_filter_slash(&out) && !word.starts_with('/') {
            out.push('/');
        }
        out.push_str(word);
    }
    out
}

fn needs_models_filter_slash(prefix: &str) -> bool {
    if prefix.ends_with(':') || prefix.ends_with('/') {
        return false;
    }
    prefix.starts_with(PI_PREFIX)
}

pub(crate) fn section_may_match(filter: Option<&str>, section_head: &str) -> bool {
    match filter {
        None => true,
        Some("") => true,
        Some(f) => f.starts_with(section_head) || section_head.starts_with(f),
    }
}

pub(crate) fn line_matches_prefix(line: &str, filter: Option<&str>) -> bool {
    let Some(f) = filter else {
        return true;
    };
    if f.is_empty() {
        return true;
    }
    let id = line.split('\t').next().unwrap_or(line).trim();
    id.starts_with(f)
}
