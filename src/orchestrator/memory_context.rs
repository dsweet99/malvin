use std::path::Path;

use rand::SeedableRng;

pub(super) const MAX_MEMORIES_PER_RUN: usize = 100;
const MEMORY_FILE_EXTENSION: &str = "md";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MemoryRecord {
    pub(super) trigger: String,
    pub(super) advice: String,
    pub(super) confidence: u64,
}

#[derive(Default)]
pub(super) struct MemoryState {
    trigger: Option<String>,
    advice: Option<String>,
    confidence: Option<u64>,
}

pub(super) fn emit_if_complete(state: &mut MemoryState, out: &mut Vec<MemoryRecord>) {
    if let (Some(trigger), Some(advice), Some(confidence)) = (
        state.trigger.take(),
        state.advice.take(),
        state.confidence.take(),
    ) {
        if !trigger.is_empty() && !advice.is_empty() {
            out.push(MemoryRecord {
                trigger,
                advice,
                confidence,
            });
        }
    }
}

pub(super) fn process_memory_line(line: &str, state: &mut MemoryState, out: &mut Vec<MemoryRecord>) {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("TRIGGER:") {
        emit_if_complete(state, out);
        state.trigger = Some(rest.trim().to_string());
        return;
    }
    if let Some(rest) = trimmed.strip_prefix("ADVICE:") {
        state.advice = Some(rest.trim().to_string());
        return;
    }
    if let Some(rest) = trimmed.strip_prefix("CONFIDENCE:") {
        state.confidence = rest.trim().parse::<u64>().ok();
    }
}

pub(super) fn parse_memories(contents: &str) -> Vec<MemoryRecord> {
    let mut out = Vec::new();
    let mut state = MemoryState::default();
    for line in contents.lines() {
        process_memory_line(line, &mut state, &mut out);
    }
    emit_if_complete(&mut state, &mut out);
    out
}

pub(super) fn collect_memory_records(work_dir: &Path) -> Vec<MemoryRecord> {
    let memory_dir = work_dir.join(".malvin_memory");
    if !memory_dir.is_dir() {
        return Vec::new();
    }

    let mut entries: Vec<_> = match std::fs::read_dir(&memory_dir) {
        Ok(entries) => entries.filter_map(Result::ok).collect(),
        Err(_) => return Vec::new(),
    };
    entries.sort_by_key(std::fs::DirEntry::path);

    let mut records = Vec::new();
    for entry in entries {
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_none_or(|ext| !ext.eq_ignore_ascii_case(MEMORY_FILE_EXTENSION))
        {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut parsed = parse_memories(&text);
        records.append(&mut parsed);
    }

    records
}

pub(super) fn format_memories(records: &[MemoryRecord]) -> String {
    let escape_template = |value: &str| {
        value
            .replace("{{", "{ {")
            .replace("}}", "} }")
            .replace('$', "$$")
    };
    records
        .iter()
        .map(|record| {
            format!(
                "TRIGGER: {}\nADVICE: {}\nCONFIDENCE: {}",
                escape_template(&record.trigger),
                escape_template(&record.advice),
                record.confidence
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub(super) fn sample_seed(path: &Path, records: &[MemoryRecord]) -> u64 {
    let mut seed = 1u64;
    for b in path.as_os_str().to_string_lossy().as_bytes() {
        seed ^= u64::from(*b);
        seed = seed.wrapping_mul(0x0100_0000_01b3);
    }
    for record in records {
        for b in record.trigger.as_bytes() {
            seed ^= u64::from(*b);
            seed = seed.rotate_left(5).wrapping_mul(0x85eb_ca6b);
        }
        for b in record.advice.as_bytes() {
            seed ^= u64::from(*b);
            seed = seed.rotate_left(7).wrapping_mul(0xc2b2_ae3d);
        }
        seed ^= record.confidence;
    }
    seed
}

pub(super) fn sample_memories(
    records: &mut Vec<MemoryRecord>,
    max: usize,
    seed: u64,
) -> Vec<MemoryRecord> {
    if records.len() <= max {
        return records.clone();
    }

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut out = Vec::with_capacity(max);

    while out.len() < max && !records.is_empty() {
        let mut total: u64 = 0;
        for record in records.iter() {
            total = total.saturating_add(record.confidence.saturating_add(1));
        }
        if total == 0 {
            break;
        }
        let mut cursor = rand::Rng::gen_range(&mut rng, 0..total);
        let mut picked = None;
        for (i, record) in records.iter().enumerate() {
            let weight = record.confidence.saturating_add(1);
            if cursor < weight {
                picked = Some(i);
                break;
            }
            cursor -= weight;
        }
        if let Some(i) = picked {
            out.push(records.remove(i));
            continue;
        }
        break;
    }

    out
}

pub(super) fn build_memories_value(work_dir: &Path) -> String {
    let mut records = collect_memory_records(work_dir);
    let seed = sample_seed(work_dir, &records);
    let sampled = sample_memories(&mut records, MAX_MEMORIES_PER_RUN, seed);
    format_memories(&sampled)
}

#[cfg(test)]
include!("memory_context/tests.rs");

