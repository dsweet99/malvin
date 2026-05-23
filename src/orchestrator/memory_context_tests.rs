use crate::prompts::PromptStore;
use std::collections::HashSet;
use std::path::Path;

use super::{
    build_memories_value, collect_memory_records, emit_if_complete, format_memories,
    parse_memories, process_memory_line, sample_memories, MemoryRecord, MemoryState,
    MAX_MEMORIES_PER_RUN,
};

#[test]
fn collect_memory_records_empty_when_no_dot_memory_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let records = collect_memory_records(tmp.path());
    assert!(records.is_empty());
}

#[test]
fn process_memory_line_and_emit_complete_one_record() {
    let mut state = MemoryState::default();
    let mut out = Vec::new();
    for line in ["TRIGGER: a", "ADVICE: b", "CONFIDENCE: 2"] {
        process_memory_line(line, &mut state, &mut out);
    }
    emit_if_complete(&mut state, &mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].trigger, "a");
    assert_eq!(out[0].advice, "b");
    assert_eq!(out[0].confidence, 2);
}

#[test]
fn parse_memories_skips_incomplete_triples() {
    let out = parse_memories("TRIGGER: one\nADVICE: do thing");
    assert!(out.is_empty());
}

#[test]
fn parse_memories_requires_valid_confidence_number() {
    let out = parse_memories("TRIGGER: one\nADVICE: do thing\nCONFIDENCE: bad");
    assert!(out.is_empty());
}

#[test]
fn parse_memories_collects_multiple_triples() {
    let out = parse_memories(
        "TRIGGER: one\nADVICE: do thing\nCONFIDENCE: 2\n\nTRIGGER: two\nADVICE: do more\nCONFIDENCE: 0",
    );
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].trigger, "one");
    assert_eq!(out[1].advice, "do more");
}

#[test]
fn sample_memories_respects_maximum_cap_and_uniqueness() {
    let mut records: Vec<MemoryRecord> = (0..110)
        .map(|i| MemoryRecord {
            trigger: format!("t{i}"),
            advice: format!("a{i}"),
            confidence: 0,
        })
        .collect();
    let sampled = sample_memories(&mut records, 100, 1);
    assert_eq!(sampled.len(), 100);

    let mut uniq = HashSet::new();
    for item in &sampled {
        assert!(uniq.insert((item.trigger.as_str(), item.advice.as_str())));
    }
}

#[test]
fn workflow_context_includes_rendered_memories() {
    let tmp = tempfile::tempdir().unwrap();
    let memory_dir = tmp.path().join(".malvin_memory");
    std::fs::create_dir(&memory_dir).unwrap();
    std::fs::write(
        memory_dir.join("index.md"),
        "TRIGGER: one\nADVICE: do thing\nCONFIDENCE: 2\n",
    )
    .unwrap();

    let rendered = build_memories_value(tmp.path());
    assert!(rendered.contains("TRIGGER: one"));
}

#[test]
fn build_memories_value_uses_sampling_limit() {
    let tmp = tempfile::tempdir().unwrap();
    let memory_dir = tmp.path().join(".malvin_memory");
    std::fs::create_dir(&memory_dir).unwrap();
    for i in 0..110 {
        std::fs::write(
            memory_dir.join(format!("{i}.md")),
            format!("TRIGGER: t{i}\nADVICE: a{i}\nCONFIDENCE: {i}\n"),
        )
        .unwrap();
    }
    let rendered = build_memories_value(tmp.path());
    assert!(rendered.matches("TRIGGER:").count() <= MAX_MEMORIES_PER_RUN);
}

#[test]
fn format_memories_escapes_template_tokens() {
    let output = format_memories(&[MemoryRecord {
        trigger: "TRIGGER: {{danger}}".to_string(),
        advice: "Use {{ plan_path }} carefully".to_string(),
        confidence: 3,
    }]);
    assert!(
        output.contains("{ {danger} }") && output.contains("{ { plan_path } }"),
        "memories should escape template tokens: {output}"
    );
}

pub(crate) fn write_memory_with_template_token(root: &Path) {
    let memory_dir = root.join(".malvin_memory");
    std::fs::create_dir(&memory_dir).unwrap();
    std::fs::write(
        memory_dir.join("index.md"),
        "TRIGGER: kpop\nADVICE: write ABORT to `{{result_path}}`\nCONFIDENCE: 3\n",
    )
    .unwrap();
}

pub(crate) fn prompt_store_with_memory_header(root: &Path) -> PromptStore {
    let prompts = root.join("prompts");
    std::fs::create_dir(&prompts).unwrap();
    std::fs::write(prompts.join("header.md"), "{{ memories }}").unwrap();
    std::fs::write(prompts.join("coding_rules.md"), "").unwrap();
    std::fs::write(prompts.join("implement.md"), "{{ coding_rules }}").unwrap();
    PromptStore::with_root(prompts)
}

pub(crate) fn context_with_memories(root: &Path) -> std::collections::HashMap<String, String> {
    let mut context = std::collections::HashMap::new();
    context.insert("memories".to_string(), build_memories_value(root));
    context
}
