//! Repo signal mining for init checks discovery (pre-commit, Makefile, dedupe).

use std::path::Path;

use super::{KISS_CHECK_COMMAND, builtin_gate_command_lines};

/// Mine hook `entry` lines from `.pre-commit-config.yaml` (first signal priority).
pub fn precommit_hook_entries(root: &Path) -> Vec<String> {
    let path = root.join(".pre-commit-config.yaml");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("entry:") else {
            continue;
        };
        let cmd = parse_yaml_scalar(rest.trim());
        if !cmd.is_empty() {
            out.push(cmd);
        }
    }
    out
}

pub(super) fn parse_yaml_scalar(raw: &str) -> String {
    let s = raw.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len().saturating_sub(1)].trim().to_string()
    } else {
        s.to_string()
    }
}

/// `lint` and `test` recipe commands from a root `Makefile` (second priority).
pub fn makefile_gate_targets(root: &Path) -> Vec<String> {
    for name in ["Makefile", "makefile", "GNUmakefile"] {
        let path = root.join(name);
        if path.is_file() {
            return parse_makefile_targets(&path);
        }
    }
    Vec::new()
}

pub(super) fn parse_makefile_targets(path: &Path) -> Vec<String> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut lines = raw.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let target = trimmed.strip_suffix(':').unwrap_or(trimmed);
        if !matches!(target.trim(), "lint" | "test") {
            continue;
        }
        if let Some(recipe) = next_makefile_recipe(&mut lines) {
            out.push(recipe);
        }
    }
    out
}

pub(crate) fn next_makefile_recipe<'a, I>(lines: &mut std::iter::Peekable<I>) -> Option<String>
where
    I: Iterator<Item = &'a str>,
{
    while let Some(&next) = lines.peek() {
        if next.trim().is_empty() {
            lines.next();
            continue;
        }
        if !next.starts_with('\t') {
            break;
        }
        let recipe = next.trim().to_string();
        lines.next();
        if recipe.is_empty() || recipe.starts_with('#') {
            return None;
        }
        return Some(recipe);
    }
    None
}

pub(super) fn canonical_tool(line: &str) -> String {
    line.split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// Deduplicate by first command token; preserve first occurrence order.
pub fn dedupe_check_lines(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let tool = canonical_tool(trimmed);
        if seen.insert(tool) {
            out.push(trimmed.to_string());
        }
    }
    out
}

/// Build ordered check lines from repo signals, then malvin builtins for gaps.
#[must_use]
pub fn discover_init_check_commands(root: &Path) -> Vec<String> {
    let mut signal_lines = precommit_hook_entries(root);
    if signal_lines.is_empty() {
        signal_lines = makefile_gate_targets(root);
    }
    let mut merged = dedupe_check_lines(&signal_lines);
    for fallback in builtin_gate_command_lines(root) {
        let tool = canonical_tool(&fallback);
        if merged.iter().any(|l| canonical_tool(l) == tool) {
            continue;
        }
        merged.push(fallback);
    }
    ensure_kiss_check_first(&mut merged);
    merged
}

pub(super) fn ensure_kiss_check_first(lines: &mut Vec<String>) {
    let kiss = KISS_CHECK_COMMAND.to_string();
    lines.retain(|l| l.trim() != KISS_CHECK_COMMAND);
    lines.insert(0, kiss);
}

#[cfg(test)]
mod local_tests {
    use super::*;

    #[test]
    fn next_makefile_recipe_breaks_when_recipe_not_indented() {
        let src = "lint:\nhelp:\n\techo x\n";
        let mut lines = src.lines().peekable();
        lines.next();
        assert!(next_makefile_recipe(&mut lines).is_none());
    }
}
