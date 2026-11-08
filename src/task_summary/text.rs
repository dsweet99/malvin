use std::path::{Path, PathBuf};

pub(super) const MAX_CHARS: usize = 20;
const MAX_CHARS_WORD: &str = "twenty";
const ELLIPSIS: &str = "...";
const TASK_DIRECTIONS: &str = include_str!("../../assets/prompts/lite/task.md");
const STATUS_DIRECTIONS: &str = include_str!("../../assets/prompts/lite/status.md");

pub(super) fn directions(subject: &str) -> String {
    let prompt = if subject == "user request" {
        TASK_DIRECTIONS
    } else {
        debug_assert_eq!(subject, "agent's current status");
        STATUS_DIRECTIONS
    };
    prompt.to_string()
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

pub(super) fn local_task(request: &str) -> Option<String> {
    let request = cleaned(request)?;
    if request.split_whitespace().count() < 10 {
        Some(shortened(&request))
    } else {
        normalize(&request)
    }
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
    let Some((mut prefix, rest)) = cut(summary) else {
        return summary.to_string();
    };
    if !rest.starts_with(char::is_whitespace)
        && let Some((words, _)) = prefix.rsplit_once(' ')
    {
        prefix = words;
    }
    mark(prefix)
}

fn cut(summary: &str) -> Option<(&str, &str)> {
    if summary.chars().count() <= MAX_CHARS {
        return None;
    }
    let budget = MAX_CHARS - ELLIPSIS.chars().count();
    let (boundary, _) = summary.char_indices().nth(budget)?;
    Some(summary.split_at(boundary))
}

fn mark(prefix: &str) -> String {
    format!("{}{ELLIPSIS}", prefix.trim_end())
}

pub(super) fn publishable(text: &str) -> Option<String> {
    let summary = normalize(text)?;
    if summary.ends_with(ELLIPSIS) {
        return Some(summary);
    }
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
    let limit_digits = MAX_CHARS.to_string();
    let mut limit = false;
    let mut character = false;
    for word in text.split_whitespace() {
        for key in parts(word) {
            limit |= key == limit_digits || key == MAX_CHARS_WORD;
            character |= key == "character" || key == "characters";
        }
    }
    limit && character
}

#[cfg(test)]
#[path = "text_tests.rs"]
mod tests;
