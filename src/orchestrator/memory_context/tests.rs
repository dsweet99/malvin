use crate::orchestrator::memory_context::{
    build_memories_value, format_memories, parse_memories, sample_memories, sample_seed,
    MemoryRecord, MAX_MEMORIES_PER_RUN,
};
use crate::prompts::PromptStore;
use std::collections::HashSet;
use std::path::Path;

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

fn write_memory_with_template_token(root: &Path) {
    let memory_dir = root.join(".malvin_memory");
    std::fs::create_dir(&memory_dir).unwrap();
    std::fs::write(
        memory_dir.join("index.md"),
        "TRIGGER: kpop\nADVICE: write ABORT to `{{result_path}}`\nCONFIDENCE: 3\n",
    )
    .unwrap();
}

fn prompt_store_with_memory_header(root: &Path) -> PromptStore {
    let prompts = root.join("prompts");
    std::fs::create_dir(&prompts).unwrap();
    std::fs::write(prompts.join("header.md"), "{{ memories }}").unwrap();
    std::fs::write(prompts.join("coding_rules.md"), "").unwrap();
    std::fs::write(prompts.join("implement.md"), "{{ coding_rules }}").unwrap();
    PromptStore::with_root(prompts)
}

fn context_with_memories(root: &Path) -> std::collections::HashMap<String, String> {
    let mut context = std::collections::HashMap::new();
    context.insert("memories".to_string(), build_memories_value(root));
    context
}

#[test]
fn rendered_memories_with_template_tokens_pass_prompt_guard() {
    let tmp = tempfile::tempdir().unwrap();
    write_memory_with_template_token(tmp.path());
    let store = prompt_store_with_memory_header(tmp.path());
    let context = context_with_memories(tmp.path());
    let prompt = store.render("implement.md", &context).unwrap();
    assert!(
        !prompt.contains("{{") && !prompt.contains("}}"),
        "rendered prompt must not trip ACP placeholder guard: {prompt}"
    );
}

#[test]
fn collect_memory_records_ignores_case_variant_markdown_extensions() {
    let tmp = tempfile::tempdir().unwrap();
    let memory_dir = tmp.path().join(".malvin_memory");
    std::fs::create_dir(&memory_dir).unwrap();
    std::fs::write(
        memory_dir.join("note.MD"),
        "TRIGGER: one\nADVICE: keep notes\nCONFIDENCE: 4\n",
    )
    .unwrap();

    let rendered = build_memories_value(tmp.path());
    assert!(rendered.contains("TRIGGER: one"));
    assert!(rendered.contains("ADVICE: keep notes"));
}

#[test]
fn format_memories_escapes_dollar_template_tokens() {
    let output = format_memories(&[MemoryRecord {
        trigger: "TRIGGER: one".to_string(),
        advice: "use $plan_path".to_string(),
        confidence: 2,
    }]);
    assert!(
        output.contains("use $$plan_path"),
        "memory advice should escape dollar tokens before prompt substitution: {output}"
    );
}

#[test]
fn sample_memories_is_deterministic_for_same_inputs() {
    let tmp = tempfile::tempdir().unwrap();
    let records_dir = tmp.path().join(".malvin_memory");
    std::fs::create_dir_all(&records_dir).unwrap();
    std::fs::write(
            records_dir.join("notes.md"),
            "TRIGGER: one\nADVICE: keep notes\nCONFIDENCE: 4\nTRIGGER: two\nADVICE: do next\nCONFIDENCE: 2\n",
        )
        .unwrap();
    let first = build_memories_value(tmp.path());
    let second = build_memories_value(tmp.path());
    assert_eq!(first, second);
}

#[test]
fn sample_memories_uses_deterministic_seed() {
    let records = vec![
        MemoryRecord {
            trigger: "t".into(),
            advice: "a".into(),
            confidence: 3,
        },
        MemoryRecord {
            trigger: "u".into(),
            advice: "b".into(),
            confidence: 1,
        },
    ];
    let seed_a = sample_seed(std::path::Path::new("/workspace/a"), &records);
    let seed_b = sample_seed(std::path::Path::new("/workspace/a"), &records);
    assert_eq!(seed_a, seed_b);
    let mut first_records = records.clone();
    let mut second_records = records;
    let first = sample_memories(&mut first_records, 1, seed_a);
    let second = sample_memories(&mut second_records, 1, seed_b);
    assert_eq!(first, second);
}
