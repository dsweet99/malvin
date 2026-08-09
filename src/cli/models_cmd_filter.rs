//! Prefix-filter helpers for `malvin models`.

use crate::model_id::PRIME_PREFIX;

/// Resolve optional listing prefix from trailing words.
///
/// Rejects legacy `download …` action words. Words are joined so path-shaped catalogs keep `/`
/// boundaries: `malvin models prime: open` → `prime:open`, and
/// `malvin models prime:local qwen` → `prime:local/qwen`.
pub(crate) fn models_list_prefix(words: &[String]) -> Result<Option<String>, String> {
    if words.is_empty() {
        return Ok(None);
    }
    if words[0].eq_ignore_ascii_case("download") {
        return Err(
            "`malvin models` no longer downloads; `prime:local/…` models fetch automatically on first use (omit `--no-download`)"
                .into(),
        );
    }
    Ok(Some(join_models_prefix_words(words)))
}

/// Join filter words, inserting `/` between path segments for `prime:` ids when the left side
/// does not already end with `:` or `/`.
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
    prefix.starts_with(PRIME_PREFIX)
}

/// Whether a catalog section whose ids start with `section_head` can produce rows for `filter`.
pub(crate) fn section_may_match(filter: Option<&str>, section_head: &str) -> bool {
    match filter {
        None => true,
        Some("") => true,
        Some(f) => f.starts_with(section_head) || section_head.starts_with(f),
    }
}

/// Whether a printed model row matches an optional id prefix filter.
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
