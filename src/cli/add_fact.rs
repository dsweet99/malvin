use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::tag_list::limit_words;
use crate::desc_limit::MAX_DESC_WORDS;

const FIELDS: &str = "fact_id, description, fact, symptoms";
const ID_MAX: usize = 64;
const FACT_MAX: usize = 4000;
const SYMPTOM_MAX: usize = 500;
const SYMPTOMS_MAX: usize = 20;
const LOCK_WAIT: Duration = Duration::from_secs(2);

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
struct NewFact {
    fact_id: String,
    description: String,
    fact: String,
    symptoms: Vec<String>,
}

fn is_invisible(c: char) -> bool {
    c.is_control()
        || matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2060}'..='\u{2069}' | '\u{FEFF}')
}

fn clean_text(name: &str, raw: &str, max: usize) -> Result<String, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(format!("{name} must not be empty"));
    }
    if value.chars().count() > max {
        return Err(format!("{name} must be at most {max} characters"));
    }
    if value.chars().any(is_invisible) {
        return Err(format!(
            "{name} must be a single line without control or invisible characters"
        ));
    }
    Ok(value.to_string())
}

fn clean_id(raw: &str) -> Result<String, String> {
    let id = raw.trim();
    let mut chars = id.chars();
    let starts_ok = chars.next().is_some_and(|c| c.is_ascii_lowercase());
    let rest_ok =
        chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    if !starts_ok || !rest_ok || id.len() > ID_MAX {
        return Err(format!(
            "fact_id `{id}` is invalid: use lowercase letters, digits, `-` and `_`, starting with a letter, at most {ID_MAX} characters"
        ));
    }
    Ok(id.to_string())
}

fn clean_description(raw: &str) -> Result<String, String> {
    let value = clean_text("description", raw, FACT_MAX)?;
    let words = value.split_whitespace().count();
    if words > MAX_DESC_WORDS {
        return Err(format!(
            "description must be at most {MAX_DESC_WORDS} words (got {words})"
        ));
    }
    Ok(value)
}

fn parse_args(args: &[String]) -> Result<NewFact, String> {
    let (mut id, mut fact, mut description, mut symptoms) = (None, None, None, Vec::new());
    for arg in args {
        let (key, value) = arg.split_once('=').ok_or_else(|| {
            format!("argument `{arg}` is not field_name=value (fields: {FIELDS})")
        })?;
        let slot = match key.trim() {
            "fact_id" => &mut id,
            "fact" => &mut fact,
            "description" => &mut description,
            "symptoms" | "symptom" => {
                symptoms.push(clean_text("symptoms", value, SYMPTOM_MAX)?);
                continue;
            }
            other => return Err(format!("unknown field `{other}` (fields: {FIELDS})")),
        };
        if slot.replace(value).is_some() {
            return Err(format!("field `{}` given more than once", key.trim()));
        }
    }
    if symptoms.len() > SYMPTOMS_MAX {
        return Err(format!("at most {SYMPTOMS_MAX} symptoms are allowed"));
    }
    let id = id.ok_or("missing required field `fact_id`")?;
    let fact = fact.ok_or("missing required field `fact`")?;
    let fact = clean_text("fact", fact, FACT_MAX)?;
    let description = match description {
        Some(raw) => clean_description(raw)?,
        None => limit_words(&fact),
    };
    Ok(NewFact {
        fact_id: clean_id(id)?,
        description,
        fact,
        symptoms,
    })
}

struct LockGuard(PathBuf);

impl LockGuard {
    fn acquire(path: PathBuf) -> Result<Self, String> {
        let start = Instant::now();
        loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Ok(Self(path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if start.elapsed() > LOCK_WAIT {
                        return Err(format!(
                            "facts file is locked by {}; remove it if no other admin facts command is running",
                            path.display()
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => return Err(format!("cannot create {}: {e}", path.display())),
            }
        }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn id_exists(text: &str, id: &str) -> bool {
    text.lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .any(|v| v.get("fact_id").and_then(serde_json::Value::as_str) == Some(id))
}

fn read_existing(path: &Path) -> Result<String, String> {
    match std::fs::read_to_string(path) {
        Ok(t) => Ok(t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(format!("cannot read {}: {e}", path.display())),
    }
}

fn write_line(path: &Path, existing: &str, new: &NewFact) -> Result<(), String> {
    let value = serde_json::to_string(new).map_err(|e| format!("cannot encode fact: {e}"))?;
    let lead = if existing.is_empty() || existing.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    replace_atomically(path, &format!("{existing}{lead}{value}\n"))
}

fn replace_atomically(path: &Path, text: &str) -> Result<(), String> {
    let mut tmp_name = path.as_os_str().to_owned();
    tmp_name.push(".tmp");
    let tmp = PathBuf::from(tmp_name);
    let mut file = std::fs::File::create(&tmp)
        .map_err(|e| format!("cannot create {}: {e}", tmp.display()))?;
    file.write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("cannot replace {}: {e}", path.display()))
}

fn append_fact_at(path: &Path, new: &NewFact) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }
    let mut lock_name = path.as_os_str().to_owned();
    lock_name.push(".lock");
    let _lock = LockGuard::acquire(PathBuf::from(lock_name))?;
    let existing = read_existing(path)?;
    if id_exists(&existing, &new.fact_id) {
        return Err(format!(
            "fact_id `{}` already exists in {}",
            new.fact_id,
            path.display()
        ));
    }
    write_line(path, &existing, new)
}

pub(crate) fn add_fact(args: &[String]) -> Result<String, String> {
    let new = parse_args(args)?;
    let path = super::facts_view::facts_path();
    append_fact_at(&path, &new)?;
    Ok(format!(
        "added fact `{}` to {}",
        new.fact_id,
        path.display()
    ))
}

fn without_fact(text: &str, id: &str) -> Option<String> {
    let mut found = false;
    let kept: String = text
        .split_inclusive('\n')
        .filter(|line| {
            let matches = serde_json::from_str::<serde_json::Value>(line)
                .ok()
                .is_some_and(|v| v.get("fact_id").and_then(serde_json::Value::as_str) == Some(id));
            found |= matches;
            !matches
        })
        .collect();
    found.then_some(kept)
}

fn remove_fact_at(path: &Path, id: &str) -> Result<(), String> {
    let mut lock_name = path.as_os_str().to_owned();
    lock_name.push(".lock");
    if !path.exists() {
        return Err(format!("no facts file at {}", path.display()));
    }
    let _lock = LockGuard::acquire(PathBuf::from(lock_name))?;
    let existing = read_existing(path)?;
    let kept = without_fact(&existing, id)
        .ok_or_else(|| format!("fact_id `{id}` not found in {}", path.display()))?;
    replace_atomically(path, &kept)
}

pub(crate) fn remove_fact(fact_id: &str) -> Result<String, String> {
    let id = clean_id(fact_id)?;
    let path = super::facts_view::facts_path();
    remove_fact_at(&path, &id)?;
    Ok(format!("removed fact `{id}` from {}", path.display()))
}

#[cfg(test)]
#[path = "add_fact_tests.rs"]
mod tests;
