use std::path::{Path, PathBuf};

pub(super) const MAX_CHARS: usize = 30;

pub(super) fn directions(subject: &str) -> String {
    format!(
        "Summarize the {subject} between the markers in {MAX_CHARS} characters or fewer. \
Return only that summary, on one line. \
The summary must describe the {subject}, not these directions.\n\n---\n"
    )
}

pub(super) fn task_path(run_dir: &Path) -> PathBuf {
    run_dir.join("request_summary.txt")
}

pub(super) fn status_path(run_dir: &Path) -> PathBuf {
    run_dir.join("status_summary.txt")
}

pub(super) fn normalize(text: &str) -> Option<String> {
    cleaned(text).filter(|summary| summary.chars().count() <= MAX_CHARS)
}

fn cleaned(text: &str) -> Option<String> {
    let text = text.trim().trim_matches(['"', '\'']);
    if text.chars().any(|c| c.is_control() && !c.is_whitespace()) {
        return None;
    }
    let words: Vec<_> = text.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }
    Some(words.join(" "))
}

pub(super) fn accept(source: &str, text: &str, directions: &str) -> Result<String, &'static str> {
    let Some(summary) = cleaned(text) else {
        return Err("summary is empty or contains control characters");
    };
    if only_directions(&summary, directions) {
        return Err("summary echoes the directions");
    }
    if instruction_echo(&summary) && !instruction_echo(source) {
        return Err("summary echoes the directions");
    }
    let summary = shortened(&summary);
    publishable(&summary).ok_or("summary echoes the directions")
}

fn shortened(summary: &str) -> String {
    let Some((boundary, _)) = summary.char_indices().nth(MAX_CHARS) else {
        return summary.to_string();
    };
    let (mut prefix, rest) = summary.split_at(boundary);
    if !rest.starts_with(char::is_whitespace)
        && let Some((words, _)) = prefix.rsplit_once(' ')
    {
        prefix = words;
    }
    prefix.trim_end().to_string()
}

pub(super) fn publishable(text: &str) -> Option<String> {
    let summary = normalize(text)?;
    let task = directions("user request");
    let status = directions("agent's current status");
    if only_directions(&summary, &task) || only_directions(&summary, &status) {
        None
    } else {
        Some(summary)
    }
}

pub(super) fn status_excerpt(text: &str) -> Option<String> {
    const MAX: usize = 4_000;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let count = trimmed.chars().count();
    if count <= MAX {
        return Some(trimmed.to_string());
    }
    Some(trimmed.chars().skip(count - MAX).collect())
}

fn only_directions(text: &str, directions: &str) -> bool {
    let mut any = false;
    for word in text.split_whitespace() {
        for key in parts(word) {
            any = true;
            if !direction_word(&key, directions) {
                return false;
            }
        }
    }
    any
}

fn parts(word: &str) -> impl Iterator<Item = String> + '_ {
    word.split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(str::to_ascii_lowercase)
}

fn direction_word(key: &str, directions: &str) -> bool {
    directions.split_whitespace().any(|word| {
        let mut keys = parts(word);
        keys.next().is_some_and(|first| first == key) && keys.next().is_none()
    })
}

fn instruction_echo(text: &str) -> bool {
    let mut limit = false;
    let mut character = false;
    for word in text.split_whitespace() {
        for key in parts(word) {
            limit |= key == "30" || key == "thirty";
            character |= key == "character" || key == "characters";
        }
    }
    limit && character
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_is_at_most_thirty_characters() {
        assert_eq!(
            normalize("\"Fix the parser\""),
            Some("Fix the parser".into())
        );
        assert_eq!(
            normalize("Add\ncheap request summaries"),
            Some("Add cheap request summaries".into())
        );
        assert_eq!(
            normalize("123456789012345678901234567890"),
            Some("123456789012345678901234567890".into())
        );
        assert_eq!(normalize("1234567890123456789012345678901"), None);
        assert_eq!(normalize("one two \x1b[31mthree"), None);
        assert_eq!(normalize(" \n"), None);
    }

    #[test]
    fn summary_must_describe_the_source_rather_than_the_directions() {
        let directions = directions("user request");
        assert!(accept("Thanks", "Return only that summary", &directions).is_err());
        assert!(accept("Thanks", "Use 30 characters now", &directions).is_err());
        assert_eq!(
            accept("Thanks", "\"User expresses thanks\"", &directions).unwrap(),
            "User expresses thanks"
        );
        assert_eq!(
            accept(
                "Show the 30-character limit",
                "Show the 30-character limit",
                &directions
            )
            .unwrap(),
            "Show the 30-character limit"
        );
    }

    #[test]
    fn stored_direction_text_is_not_publishable() {
        assert_eq!(publishable("Return only that summary"), None);
        assert_eq!(
            publishable("Add cheap request summaries"),
            Some("Add cheap request summaries".into())
        );
    }

    #[test]
    fn model_summaries_fit_at_word_and_unicode_boundaries() {
        let directions = directions("agent's current status");
        assert_eq!(
            accept(
                "Inspecting code",
                "Investigating missing Herdr status messages",
                &directions
            ),
            Ok("Investigating missing Herdr".into())
        );
        assert_eq!(
            accept("Inspecting code", &"é".repeat(31), &directions),
            Ok("é".repeat(30))
        );
        assert!(
            accept(
                "Inspecting code",
                "Return only that summary on one line in 30 characters",
                &directions
            )
            .is_err()
        );
        assert!(accept("Inspecting code", " \n", &directions).is_err());
        assert!(accept("Inspecting code", "Editing \x1b[31mthe parser", &directions).is_err());
    }

    #[test]
    fn status_excerpt_keeps_the_recent_tail() {
        assert_eq!(status_excerpt("  \n"), None);
        assert_eq!(status_excerpt(" Reading "), Some("Reading".into()));
        let long = format!("{}TAIL", "x".repeat(4_010));
        let excerpt = status_excerpt(&long).unwrap();
        assert_eq!(excerpt.chars().count(), 4_000);
        assert!(excerpt.ends_with("TAIL"));
    }
}
